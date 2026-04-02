use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(3);
const MAX_BUFFER: usize = 10 * 1024; // 10KB
const MAX_LABEL_LEN: usize = 50;

/// --extra-cmd 인자를 파싱한다.
/// `--extra-cmd "command"` 또는 `--extra-cmd=command` 형식 지원.
pub fn parse_extra_cmd_arg() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    for (i, arg) in args.iter().enumerate() {
        if let Some(val) = arg.strip_prefix("--extra-cmd=") {
            if val.is_empty() {
                return None;
            }
            return Some(val.to_string());
        }
        if arg == "--extra-cmd" {
            return args.get(i + 1).filter(|v| !v.is_empty()).cloned();
        }
    }
    None
}

/// 셸 명령을 실행하고 JSON `{ "label": "..." }` 출력에서 label을 추출한다.
pub fn run_extra_cmd(cmd: &str) -> Option<String> {
    let mut child = Command::new("sh")
        .args(["-c", cmd])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                break;
            }
            Ok(None) => {
                if start.elapsed() > TIMEOUT {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return None,
        }
    }

    let mut buf = vec![0u8; MAX_BUFFER];
    let n = child.stdout.as_mut()?.read(&mut buf).ok()?;
    let output = std::str::from_utf8(&buf[..n]).ok()?.trim();

    let val: serde_json::Value = serde_json::from_str(output).ok()?;
    let label = val.get("label")?.as_str()?;

    let sanitized = sanitize(label);
    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

/// ANSI 이스케이프, 제어문자, bidi 문자를 제거하고 길이를 제한한다.
fn sanitize(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut visible = 0;

    while let Some(c) = chars.next() {
        if visible >= MAX_LABEL_LEN {
            result.push('…');
            break;
        }
        // ANSI escape
        if c == '\x1b' {
            while let Some(&nc) = chars.peek() {
                chars.next();
                if nc.is_ascii_alphabetic() || nc == 'm' {
                    break;
                }
            }
            continue;
        }
        // 제어 문자 (탭/개행 포함)
        if c.is_control() {
            continue;
        }
        // bidi 제어 문자
        if matches!(c,
            '\u{061C}' | '\u{200E}' | '\u{200F}' |
            '\u{202A}'..='\u{202E}' |
            '\u{2066}'..='\u{2069}' |
            '\u{206A}'..='\u{206F}'
        ) {
            continue;
        }
        result.push(c);
        visible += 1;
    }

    result
}
