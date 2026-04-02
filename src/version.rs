use crate::config::get_claude_config_dir;
use std::process::Command;

pub fn get_claude_code_version() -> Option<String> {
    // Try reading from version file first
    let version_file = get_claude_config_dir().join("version");
    if let Ok(content) = std::fs::read_to_string(&version_file) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // Fallback: try `claude --version`
    let output = Command::new("claude")
        .arg("--version")
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Extract version number from output like "Claude Code v1.2.3"
        if let Some(v) = text.split_whitespace().last() {
            let v = v.trim_start_matches('v');
            if v.contains('.') {
                return Some(v.to_string());
            }
        }
    }

    None
}
