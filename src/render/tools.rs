use crate::render::colors::*;
use crate::types::{RenderContext, ToolStatus};
use std::collections::HashMap;

pub fn render_tools_line(ctx: &RenderContext) -> Option<String> {
    let tools = &ctx.transcript.tools;
    let colors = &ctx.config.colors;

    if tools.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    let running: Vec<_> = tools.iter().filter(|t| t.status == ToolStatus::Running).collect();
    let completed: Vec<_> = tools
        .iter()
        .filter(|t| t.status == ToolStatus::Completed || t.status == ToolStatus::Error)
        .collect();

    for tool in running.iter().rev().take(2).rev() {
        let target = tool.target.as_deref().map(|t| truncate_path(t, 20)).unwrap_or_default();
        if target.is_empty() {
            parts.push(format!("{} {}", yellow("◐"), cyan(&tool.name)));
        } else {
            parts.push(format!(
                "{} {}{}",
                yellow("◐"),
                cyan(&tool.name),
                label_color(&format!(": {target}"), colors)
            ));
        }
    }

    let mut tool_counts: HashMap<&str, u32> = HashMap::new();
    for tool in &completed {
        *tool_counts.entry(&tool.name).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = tool_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    for (name, count) in sorted.into_iter().take(4) {
        parts.push(format!(
            "{} {} {}",
            green("✓"),
            name,
            label_color(&format!("×{count}"), colors)
        ));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" | "))
    }
}

fn truncate_path(path: &str, max_len: usize) -> String {
    let normalized = path.replace('\\', "/");
    if normalized.len() <= max_len {
        return normalized;
    }

    let parts: Vec<&str> = normalized.split('/').collect();
    let filename = parts.last().unwrap_or(&path);

    if filename.len() >= max_len {
        return format!("{}...", &filename[..max_len.saturating_sub(3)]);
    }

    format!(".../{filename}")
}
