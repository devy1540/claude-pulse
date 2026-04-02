use crate::types::StdinData;

/// 모델별 토큰 가격 (USD per 1M tokens)
struct Pricing {
    input: f64,
    output: f64,
    cache_read: f64,
}

fn get_pricing(model_name: &str) -> Pricing {
    let lower = model_name.to_lowercase();

    if lower.contains("opus") {
        // Opus 3 / 3.5 / 4 / 4.6 — 동일 가격
        Pricing { input: 15.0, output: 75.0, cache_read: 1.50 }
    } else if lower.contains("sonnet") {
        // Sonnet 3.5 / 4 / 4.6 — 동일 가격
        Pricing { input: 3.0, output: 15.0, cache_read: 0.30 }
    } else if lower.contains("haiku") {
        // Claude 3 Haiku: $0.25/$1.25, Claude 3.5+ Haiku: $0.80/$4.00
        if lower.contains("-3-haiku") || lower.contains("haiku 3 ") {
            Pricing { input: 0.25, output: 1.25, cache_read: 0.03 }
        } else {
            Pricing { input: 0.80, output: 4.00, cache_read: 0.08 }
        }
    } else {
        // 알 수 없는 모델은 Sonnet 가격 기준
        Pricing { input: 3.0, output: 15.0, cache_read: 0.30 }
    }
}

pub fn estimate_cost(stdin: &StdinData) -> Option<String> {
    let model_name = stdin.model.as_ref()?.display_name.as_deref()
        .or(stdin.model.as_ref()?.id.as_deref())?;
    let usage = stdin.context_window.as_ref()?.current_usage.as_ref()?;

    let pricing = get_pricing(model_name);

    let input_tokens = usage.input_tokens.unwrap_or(0) as f64;
    let output_tokens = usage.output_tokens.unwrap_or(0) as f64;
    let cache_create = usage.cache_creation_input_tokens.unwrap_or(0) as f64;
    let cache_read = usage.cache_read_input_tokens.unwrap_or(0) as f64;

    // cache creation은 input 가격의 1.25배
    let cost = (input_tokens * pricing.input
        + output_tokens * pricing.output
        + cache_create * pricing.input * 1.25
        + cache_read * pricing.cache_read)
        / 1_000_000.0;

    if cost < 0.01 {
        Some(format!("~${:.2}", cost))
    } else if cost < 1.0 {
        Some(format!("~${:.2}", cost))
    } else {
        Some(format!("~${:.1}", cost))
    }
}
