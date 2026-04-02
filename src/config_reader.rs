use crate::config::get_claude_config_dir;
use crate::types::ConfigCounts;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn count_configs(cwd: Option<&str>) -> ConfigCounts {
    let mut counts = ConfigCounts::default();
    let claude_dir = get_claude_config_dir();

    let mut user_mcp_servers: HashSet<String> = HashSet::new();
    let mut project_mcp_servers: HashSet<String> = HashSet::new();

    // === USER SCOPE ===

    if claude_dir.join("CLAUDE.md").exists() {
        counts.claude_md_count += 1;
    }

    counts.rules_count += count_rules_in_dir(&claude_dir.join("rules"));

    let user_settings = claude_dir.join("settings.json");
    for name in get_mcp_server_names(&user_settings) {
        user_mcp_servers.insert(name);
    }
    counts.hooks_count += count_hooks_in_file(&user_settings);

    let user_claude_json = format!("{}.json", claude_dir.display());
    let user_claude_json = Path::new(&user_claude_json);
    for name in get_mcp_server_names(user_claude_json) {
        user_mcp_servers.insert(name);
    }

    let disabled = get_disabled_mcp_servers(user_claude_json, "disabledMcpServers");
    for name in disabled {
        user_mcp_servers.remove(&name);
    }

    // === PROJECT SCOPE ===

    if let Some(cwd) = cwd {
        let cwd_path = Path::new(cwd);
        let project_claude_dir = cwd_path.join(".claude");
        let overlaps = paths_refer_to_same_location(&project_claude_dir, &claude_dir);

        if cwd_path.join("CLAUDE.md").exists() {
            counts.claude_md_count += 1;
        }
        if cwd_path.join("CLAUDE.local.md").exists() {
            counts.claude_md_count += 1;
        }
        if !overlaps && project_claude_dir.join("CLAUDE.md").exists() {
            counts.claude_md_count += 1;
        }
        if project_claude_dir.join("CLAUDE.local.md").exists() {
            counts.claude_md_count += 1;
        }

        if !overlaps {
            counts.rules_count += count_rules_in_dir(&project_claude_dir.join("rules"));
        }

        let mut mcp_json_servers = get_mcp_server_names(&cwd_path.join(".mcp.json"));

        let project_settings = project_claude_dir.join("settings.json");
        if !overlaps {
            for name in get_mcp_server_names(&project_settings) {
                project_mcp_servers.insert(name);
            }
            counts.hooks_count += count_hooks_in_file(&project_settings);
        }

        let local_settings = project_claude_dir.join("settings.local.json");
        for name in get_mcp_server_names(&local_settings) {
            project_mcp_servers.insert(name);
        }
        counts.hooks_count += count_hooks_in_file(&local_settings);

        let disabled_mcp_json = get_disabled_mcp_servers(&local_settings, "disabledMcpjsonServers");
        for name in disabled_mcp_json {
            mcp_json_servers.remove(&name);
        }

        for name in mcp_json_servers {
            project_mcp_servers.insert(name);
        }
    }

    counts.mcp_count = (user_mcp_servers.len() + project_mcp_servers.len()) as u32;
    counts
}

fn get_mcp_server_names(path: &Path) -> HashSet<String> {
    let mut names = HashSet::new();
    if !path.exists() {
        return names;
    }
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(val) = serde_json::from_str::<Value>(&content) {
            if let Some(obj) = val.get("mcpServers").and_then(|v| v.as_object()) {
                for key in obj.keys() {
                    names.insert(key.clone());
                }
            }
        }
    }
    names
}

fn get_disabled_mcp_servers(path: &Path, key: &str) -> HashSet<String> {
    let mut disabled = HashSet::new();
    if !path.exists() {
        return disabled;
    }
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(val) = serde_json::from_str::<Value>(&content) {
            if let Some(arr) = val.get(key).and_then(|v| v.as_array()) {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        disabled.insert(s.to_string());
                    }
                }
            }
        }
    }
    disabled
}

fn count_hooks_in_file(path: &Path) -> u32 {
    if !path.exists() {
        return 0;
    }
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(val) = serde_json::from_str::<Value>(&content) {
            if let Some(obj) = val.get("hooks").and_then(|v| v.as_object()) {
                return obj.len() as u32;
            }
        }
    }
    0
}

fn count_rules_in_dir(dir: &Path) -> u32 {
    if !dir.exists() {
        return 0;
    }
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_rules_in_dir(&path);
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "md" {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

fn paths_refer_to_same_location(a: &Path, b: &Path) -> bool {
    if let (Ok(a_canon), Ok(b_canon)) = (fs::canonicalize(a), fs::canonicalize(b)) {
        return a_canon == b_canon;
    }
    a == b
}
