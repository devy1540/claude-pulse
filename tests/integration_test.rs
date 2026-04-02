use std::process::Command;

/// 테스트별 격리된 config 디렉토리 생성 (병렬 실행 안전)
fn make_test_config_dir() -> String {
    let dir = std::env::temp_dir().join(format!(
        "claude-pulse-test-{:?}-{}",
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_string_lossy().to_string()
}

/// config 없이 바이너리 실행
fn run_with_stdin(json: &str) -> String {
    run_with_config(json, None)
}

/// 선택적 config와 함께 바이너리 실행 (격리된 디렉토리 사용)
fn run_with_config(json: &str, config_json: Option<&str>) -> String {
    let config_dir = make_test_config_dir();

    if let Some(cfg) = config_json {
        let plugin_dir = format!("{}/plugins/claude-pulse", config_dir);
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(format!("{}/config.json", plugin_dir), cfg).unwrap();
    }

    let output = Command::new("cargo")
        .args(["run", "--release", "--quiet", "--"])
        .env("CLAUDE_CONFIG_DIR", &config_dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(json.as_bytes()).ok();
            }
            child.wait_with_output()
        })
        .expect("failed to run");

    let _ = std::fs::remove_dir_all(&config_dir);
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc.is_ascii_alphabetic() || nc == 'm' {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

const BASE_JSON: &str = r#"{"model":{"display_name":"Opus"},"context_window":{"context_window_size":200000,"current_usage":{"input_tokens":50000},"used_percentage":25},"cwd":"/tmp","transcript_path":""}"#;

// ── 기본 출력 ───────────────────────────────────────────────

#[test]
fn test_basic_output_has_model_and_context() {
    let out = strip_ansi(&run_with_stdin(BASE_JSON));
    assert!(out.contains("[Opus]"), "모델명: {out}");
    assert!(out.contains("25%"), "컨텍스트 퍼센트: {out}");
    assert!(out.contains("ctx"), "ctx 라벨: {out}");
}

#[test]
fn test_empty_stdin_shows_init_message() {
    let out = run_with_stdin("");
    assert!(out.contains("[claude-pulse]"), "초기화 메시지: {out}");
}

#[test]
fn test_usage_placeholder_when_no_rate_limits() {
    let out = strip_ansi(&run_with_stdin(BASE_JSON));
    assert!(out.contains("--%"), "rate_limits 없을 때 --% 표시: {out}");
}

#[test]
fn test_usage_shows_percentage() {
    let json = r#"{"model":{"display_name":"Opus"},"context_window":{"context_window_size":200000,"current_usage":{"input_tokens":10000},"used_percentage":5},"cwd":"/tmp","transcript_path":"","rate_limits":{"five_hour":{"used_percentage":42}}}"#;
    let out = strip_ansi(&run_with_stdin(json));
    assert!(out.contains("42%"), "사용량 표시: {out}");
}

// ── 모델 파싱 ───────────────────────────────────────────────

#[test]
fn test_bedrock_model_id() {
    let json = r#"{"model":{"id":"anthropic.claude-3-5-sonnet-20240620-v1:0"},"context_window":{"context_window_size":200000,"current_usage":{"input_tokens":10000},"used_percentage":5},"cwd":"/tmp","transcript_path":""}"#;
    let out = strip_ansi(&run_with_stdin(json));
    assert!(out.contains("Sonnet"), "Bedrock 모델 ID 파싱: {out}");
}

// ── 임계값 ──────────────────────────────────────────────────

#[test]
fn test_high_context_shows_token_breakdown() {
    let json = r#"{"model":{"display_name":"Opus"},"context_window":{"context_window_size":200000,"current_usage":{"input_tokens":170000,"cache_creation_input_tokens":10000,"cache_read_input_tokens":5000},"used_percentage":92},"cwd":"/tmp","transcript_path":""}"#;
    let out = strip_ansi(&run_with_stdin(json));
    assert!(
        out.contains("in:") && out.contains("cache:"),
        "토큰 브레이크다운: {out}"
    );
}

#[test]
fn test_limit_reached_shows_warning() {
    let json = r#"{"model":{"display_name":"Opus"},"context_window":{"context_window_size":200000,"current_usage":{"input_tokens":10000},"used_percentage":5},"cwd":"/tmp","transcript_path":"","rate_limits":{"five_hour":{"used_percentage":100}}}"#;
    let out = strip_ansi(&run_with_stdin(json));
    assert!(out.contains("Limit reached"), "100% 경고: {out}");
}

// ── 템플릿 + 비용 ──────────────────────────────────────────

#[test]
fn test_cost_estimation_in_template() {
    let json = r#"{"model":{"display_name":"Opus"},"context_window":{"context_window_size":200000,"current_usage":{"input_tokens":100000,"output_tokens":5000,"cache_creation_input_tokens":2000,"cache_read_input_tokens":1000},"used_percentage":50},"cwd":"/tmp","transcript_path":""}"#;
    let out = strip_ansi(&run_with_config(
        json,
        Some(r#"{"lines": ["{model} | {cost}"]}"#),
    ));
    assert!(out.contains("~$"), "비용 추정 표시: {out}");
}

// ── 커스텀 바 ───────────────────────────────────────────────

#[test]
fn test_custom_bar_style() {
    let out = strip_ansi(&run_with_config(
        BASE_JSON,
        Some(r##"{"bar": {"filled": "#", "empty": "-", "width": 5}}"##),
    ));
    assert!(
        out.contains('#') && out.contains('-'),
        "커스텀 바: {out}"
    );
}
