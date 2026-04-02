use crate::config::get_hud_plugin_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_POINTS: usize = 20;
const SAMPLE_INTERVAL_MS: u64 = 5_000; // 5초마다 샘플링
const SPARK_CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

#[derive(Serialize, Deserialize)]
struct SparkPoint {
    ts: u64,
    pct: u32,
}

#[derive(Serialize, Deserialize, Default)]
struct SparkCache {
    session: String,
    points: Vec<SparkPoint>,
}

fn cache_path() -> std::path::PathBuf {
    get_hud_plugin_dir().join(".sparkline.json")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn record_and_render(session_id: &str, context_pct: u32) -> Option<String> {
    let path = cache_path();
    let mut cache = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<SparkCache>(&s).ok())
        .unwrap_or_default();

    // 세션이 바뀌면 리셋
    if cache.session != session_id {
        cache = SparkCache {
            session: session_id.to_string(),
            points: Vec::new(),
        };
    }

    let now = now_ms();
    let should_sample = cache
        .points
        .last()
        .map(|p| now.saturating_sub(p.ts) >= SAMPLE_INTERVAL_MS)
        .unwrap_or(true);

    if should_sample {
        cache.points.push(SparkPoint {
            ts: now,
            pct: context_pct,
        });
        if cache.points.len() > MAX_POINTS {
            cache.points.drain(..cache.points.len() - MAX_POINTS);
        }

        // 쓰기 실패는 무시
        let _ = fs::create_dir_all(path.parent().unwrap_or(&path));
        let _ = fs::write(&path, serde_json::to_string(&cache).unwrap_or_default());
    }

    if cache.points.len() < 2 {
        return None;
    }

    let spark: String = cache
        .points
        .iter()
        .map(|p| {
            let idx = ((p.pct as f64 / 100.0) * (SPARK_CHARS.len() - 1) as f64).round() as usize;
            SPARK_CHARS[idx.min(SPARK_CHARS.len() - 1)]
        })
        .collect();

    Some(spark)
}
