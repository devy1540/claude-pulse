pub mod agents;
pub mod colors;
pub mod template;
pub mod todos;
pub mod tools;

use crate::types::*;
use colors::RESET;
use unicode_width::UnicodeWidthStr;

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
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
            result.push(c);
        }
    }
    result
}

fn visual_length(s: &str) -> usize {
    UnicodeWidthStr::width(strip_ansi(s).as_str())
}

fn slice_visible(s: &str, max_visible: usize) -> String {
    if max_visible == 0 {
        return String::new();
    }
    let mut result = String::new();
    let mut visible_width = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            result.push(c);
            while let Some(&nc) = chars.peek() {
                result.push(*chars.peek().unwrap());
                chars.next();
                if nc.is_ascii_alphabetic() || nc == 'm' {
                    break;
                }
            }
            continue;
        }
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if visible_width + w > max_visible {
            break;
        }
        result.push(c);
        visible_width += w;
    }
    result
}

fn truncate_to_width(s: &str, max_width: usize) -> String {
    if max_width == 0 || visual_length(s) <= max_width {
        return s.to_string();
    }
    let suffix = if max_width >= 3 { "..." } else { &".".repeat(max_width) };
    let keep = max_width.saturating_sub(suffix.len());
    format!("{}{suffix}{RESET}", slice_visible(s, keep))
}

fn wrap_line_to_width(line: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 || visual_length(line) <= max_width {
        return vec![line.to_string()];
    }

    // │ 또는 | 로 분할 시도
    let parts: Vec<&str> = line.split(" │ ").collect();
    if parts.len() <= 1 {
        let parts2: Vec<&str> = line.split(" | ").collect();
        if parts2.len() <= 1 {
            return vec![truncate_to_width(line, max_width)];
        }
        return wrap_parts(&parts2, " | ", max_width);
    }
    wrap_parts(&parts, " │ ", max_width)
}

fn wrap_parts(parts: &[&str], sep: &str, max_width: usize) -> Vec<String> {
    let mut wrapped: Vec<String> = Vec::new();
    let mut current = parts[0].to_string();

    for part in &parts[1..] {
        let candidate = format!("{current}{sep}{part}");
        if visual_length(&candidate) <= max_width {
            current = candidate;
        } else {
            wrapped.push(truncate_to_width(&current, max_width));
            current = part.to_string();
        }
    }
    if !current.is_empty() {
        wrapped.push(truncate_to_width(&current, max_width));
    }
    wrapped
}

pub fn render(ctx: &RenderContext) {
    let terminal_width = ctx.terminal_width.map(|w| w as usize);

    // 항상 템플릿 렌더러 사용 (lines 없으면 디폴트 라인 자동 생성)
    let lines = template::render_template(ctx);

    // 물리적 줄 분리 + 터미널 너비 래핑
    let physical: Vec<String> = lines
        .into_iter()
        .flat_map(|l| l.split('\n').map(String::from).collect::<Vec<_>>())
        .collect();

    let visible: Vec<String> = match terminal_width {
        Some(tw) => physical
            .into_iter()
            .flat_map(|l| wrap_line_to_width(&l, tw))
            .collect(),
        None => physical,
    };

    for line in visible {
        println!("{RESET}{line}");
    }
}
