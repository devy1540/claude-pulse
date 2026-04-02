use crate::types::{StdinData, UsageData};
use std::io::Read;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const AUTOCOMPACT_BUFFER_PERCENT: f64 = 0.165;

pub fn read_stdin() -> Option<StdinData> {
    if atty::is(atty::Stream::Stdin) {
        return None;
    }

    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).ok()?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

// atty equivalent without the crate - check if stdin is a TTY
mod atty {
    pub enum Stream {
        Stdin,
    }

    pub fn is(stream: Stream) -> bool {
        match stream {
            Stream::Stdin => unsafe { libc_isatty(0) != 0 },
        }
    }

    extern "C" {
        #[link_name = "isatty"]
        fn libc_isatty(fd: i32) -> i32;
    }
}

pub fn get_total_tokens(stdin: &StdinData) -> u64 {
    let usage = match &stdin.context_window {
        Some(cw) => cw.current_usage.as_ref(),
        None => None,
    };
    match usage {
        Some(u) => {
            u.input_tokens.unwrap_or(0)
                + u.cache_creation_input_tokens.unwrap_or(0)
                + u.cache_read_input_tokens.unwrap_or(0)
        }
        None => 0,
    }
}

fn get_native_percent(stdin: &StdinData) -> Option<u32> {
    let p = stdin.context_window.as_ref()?.used_percentage?;
    if p.is_nan() {
        return None;
    }
    Some(p.round().clamp(0.0, 100.0) as u32)
}

pub fn get_context_percent(stdin: &StdinData) -> u32 {
    if let Some(native) = get_native_percent(stdin) {
        return native;
    }

    let size = match &stdin.context_window {
        Some(cw) => cw.context_window_size.unwrap_or(0),
        None => 0,
    };
    if size == 0 {
        return 0;
    }

    let total = get_total_tokens(stdin);
    ((total as f64 / size as f64) * 100.0).round().min(100.0) as u32
}

pub fn get_buffered_percent(stdin: &StdinData) -> u32 {
    if let Some(native) = get_native_percent(stdin) {
        return native;
    }

    let size = match &stdin.context_window {
        Some(cw) => cw.context_window_size.unwrap_or(0) as f64,
        None => 0.0,
    };
    if size <= 0.0 {
        return 0;
    }

    let total = get_total_tokens(stdin) as f64;
    let raw_ratio = total / size;
    let scale = ((raw_ratio - 0.05) / (0.50 - 0.05)).clamp(0.0, 1.0);
    let buffer = size * AUTOCOMPACT_BUFFER_PERCENT * scale;

    (((total + buffer) / size) * 100.0).round().min(100.0) as u32
}

pub fn get_model_name(stdin: &StdinData) -> String {
    if let Some(m) = &stdin.model {
        if let Some(name) = &m.display_name {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Some(id) = &m.id {
            let trimmed = id.trim();
            if !trimmed.is_empty() {
                return normalize_bedrock_model_label(trimmed)
                    .unwrap_or_else(|| trimmed.to_string());
            }
        }
    }
    "Unknown".to_string()
}

pub fn is_bedrock_model_id(model_id: Option<&str>) -> bool {
    match model_id {
        Some(id) => id.to_lowercase().contains("anthropic.claude-"),
        None => false,
    }
}

pub fn get_provider_label(stdin: &StdinData) -> Option<&'static str> {
    let id = stdin.model.as_ref()?.id.as_deref();
    if is_bedrock_model_id(id) {
        Some("Bedrock")
    } else {
        None
    }
}

fn parse_rate_limit_percent(value: Option<f64>) -> Option<u32> {
    let v = value?;
    if !v.is_finite() {
        return None;
    }
    Some(v.round().clamp(0.0, 100.0) as u32)
}

fn parse_rate_limit_reset_at(value: Option<f64>) -> Option<SystemTime> {
    let v = value?;
    if !v.is_finite() || v <= 0.0 {
        return None;
    }
    Some(UNIX_EPOCH + Duration::from_secs_f64(v))
}

pub fn get_usage_from_stdin(stdin: &StdinData) -> Option<UsageData> {
    let rate_limits = stdin.rate_limits.as_ref()?;

    let five_hour = parse_rate_limit_percent(
        rate_limits.five_hour.as_ref().and_then(|w| w.used_percentage),
    );
    let seven_day = parse_rate_limit_percent(
        rate_limits.seven_day.as_ref().and_then(|w| w.used_percentage),
    );

    if five_hour.is_none() && seven_day.is_none() {
        return None;
    }

    Some(UsageData {
        five_hour,
        seven_day,
        five_hour_reset_at: parse_rate_limit_reset_at(
            rate_limits.five_hour.as_ref().and_then(|w| w.resets_at),
        ),
        seven_day_reset_at: parse_rate_limit_reset_at(
            rate_limits.seven_day.as_ref().and_then(|w| w.resets_at),
        ),
    })
}

fn normalize_bedrock_model_label(model_id: &str) -> Option<String> {
    let lower = model_id.to_lowercase();
    let prefix = "anthropic.claude-";
    let idx = lower.find(prefix)?;

    let mut suffix = &lower[idx + prefix.len()..];

    // Strip version suffix like -v1:0
    if let Some(i) = suffix.rfind("-v") {
        if suffix[i + 2..].contains(':') {
            suffix = &suffix[..i];
        }
    }
    // Strip date suffix like -20240301
    let parts_check: Vec<&str> = suffix.rsplitn(2, '-').collect();
    if parts_check.len() == 2 && parts_check[0].len() == 8 && parts_check[0].chars().all(|c| c.is_ascii_digit()) {
        suffix = parts_check[1];
    }

    let tokens: Vec<&str> = suffix.split('-').filter(|t| !t.is_empty()).collect();
    if tokens.is_empty() {
        return None;
    }

    let family_idx = tokens.iter().position(|t| *t == "haiku" || *t == "sonnet" || *t == "opus")?;
    let family = tokens[family_idx];

    let before_version = read_numeric_version(&tokens, family_idx as isize - 1, -1);
    let after_version = read_numeric_version(&tokens, family_idx as isize + 1, 1);

    let version_parts = if before_version.len() >= after_version.len() {
        let mut v = before_version;
        v.reverse();
        v
    } else {
        after_version
    };

    let family_label = format!("{}{}", family[..1].to_uppercase(), &family[1..]);
    if version_parts.is_empty() {
        Some(format!("Claude {family_label}"))
    } else {
        Some(format!("Claude {family_label} {}", version_parts.join(".")))
    }
}

fn read_numeric_version(tokens: &[&str], start: isize, step: isize) -> Vec<String> {
    let mut parts = Vec::new();
    let mut i = start;
    while i >= 0 && (i as usize) < tokens.len() {
        if tokens[i as usize].chars().all(|c| c.is_ascii_digit()) && !tokens[i as usize].is_empty()
        {
            parts.push(tokens[i as usize].to_string());
            if parts.len() == 2 {
                break;
            }
        } else {
            break;
        }
        i += step;
    }
    parts
}
