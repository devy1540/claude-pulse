use crate::config::get_hud_plugin_dir;
use crate::types::StdinData;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const SPEED_WINDOW_MS: u64 = 5_000;

#[derive(Serialize, Deserialize)]
struct SpeedCache {
    output_tokens: u64,
    timestamp_ms: u64,
    last_speed: Option<f64>,
}

fn cache_path() -> std::path::PathBuf {
    get_hud_plugin_dir().join(".speed-cache.json")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn get_output_speed(stdin: &StdinData) -> Option<f64> {
    let output_tokens = stdin
        .context_window
        .as_ref()?
        .current_usage
        .as_ref()?
        .output_tokens?;

    let now = now_ms();
    let path = cache_path();

    let previous = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<SpeedCache>(&s).ok());

    let (speed, last_speed) = match &previous {
        Some(prev) if output_tokens >= prev.output_tokens => {
            let delta_tokens = output_tokens - prev.output_tokens;
            let delta_ms = now.saturating_sub(prev.timestamp_ms);
            if delta_tokens > 0 && delta_ms > 0 && delta_ms <= SPEED_WINDOW_MS {
                let s = delta_tokens as f64 / (delta_ms as f64 / 1000.0);
                (Some(s), Some(s))
            } else {
                // 측정 불가 → 마지막 속도 유지
                (prev.last_speed, prev.last_speed)
            }
        }
        Some(_prev) => {
            // output_tokens 감소 (세션 변경) → 리셋
            (None, None)
        }
        None => (None, None),
    };

    let cache = SpeedCache {
        output_tokens,
        timestamp_ms: now,
        last_speed,
    };
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    let _ = std::fs::write(&path, serde_json::to_string(&cache).unwrap_or_default());

    speed
}
