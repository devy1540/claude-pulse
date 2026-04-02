use crate::render::colors::*;
use crate::types::{AgentEntry, AgentStatus, RenderContext};
use std::time::SystemTime;

pub fn render_agents_line(ctx: &RenderContext) -> Option<String> {
    let agents = &ctx.transcript.agents;
    let colors = &ctx.config.colors;

    let running: Vec<&AgentEntry> = agents.iter().filter(|a| a.status == AgentStatus::Running).collect();
    let recent_completed: Vec<&AgentEntry> = agents
        .iter()
        .filter(|a| a.status == AgentStatus::Completed)
        .rev()
        .take(2)
        .collect();

    let mut to_show: Vec<&AgentEntry> = Vec::new();
    to_show.extend(running.iter());
    to_show.extend(recent_completed.iter().rev());
    to_show.truncate(3);

    if to_show.is_empty() {
        return None;
    }

    let lines: Vec<String> = to_show.iter().map(|agent| format_agent(agent, colors)).collect();
    Some(lines.join("\n"))
}

fn format_agent(agent: &AgentEntry, colors: &crate::types::ColorOverrides) -> String {
    let status_icon = if agent.status == AgentStatus::Running {
        yellow("◐")
    } else {
        green("✓")
    };
    let agent_type = magenta(&agent.agent_type);
    let model_part = agent
        .model
        .as_deref()
        .map(|m| format!(" {}", label_color(&format!("[{m}]"), colors)))
        .unwrap_or_default();
    let desc_part = agent
        .description
        .as_deref()
        .map(|d| label_color(&format!(": {}", truncate_desc(d, 40)), colors))
        .unwrap_or_default();
    let elapsed = format_elapsed(agent);

    format!(
        "{status_icon} {agent_type}{model_part}{desc_part} {}",
        label_color(&format!("({elapsed})"), colors)
    )
}

fn truncate_desc(desc: &str, max_len: usize) -> String {
    if desc.len() <= max_len {
        desc.to_string()
    } else {
        format!("{}...", &desc[..max_len.saturating_sub(3)])
    }
}

fn format_elapsed(agent: &AgentEntry) -> String {
    let now = SystemTime::now();
    let end = agent.end_time.unwrap_or(now);
    let ms = end
        .duration_since(agent.start_time)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    if ms < 1000 {
        "<1s".to_string()
    } else if ms < 60_000 {
        format!("{}s", (ms + 500) / 1000)
    } else {
        let mins = ms / 60_000;
        let secs = ((ms % 60_000) + 500) / 1000;
        format!("{mins}m {secs}s")
    }
}
