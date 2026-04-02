use crate::types::*;
use serde_json::Value;
use std::path::PathBuf;

pub fn get_claude_config_dir() -> PathBuf {
    let home = dirs_home();
    match std::env::var("CLAUDE_CONFIG_DIR") {
        Ok(d) if !d.trim().is_empty() => {
            let d = d.trim();
            if d == "~" {
                return home;
            }
            if d.starts_with("~/") || d.starts_with("~\\") {
                return home.join(&d[2..]);
            }
            PathBuf::from(d)
        }
        _ => home.join(".claude"),
    }
}

pub fn get_hud_plugin_dir() -> PathBuf {
    get_claude_config_dir().join("plugins").join("claude-pulse")
}

pub fn get_config_path() -> PathBuf {
    get_hud_plugin_dir().join("config.json")
}

pub fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

pub fn default_config() -> HudConfig {
    HudConfig {
        line_layout: LineLayout::Expanded,
        show_separators: false,
        path_levels: 1,
        element_order: default_element_order(),
        git_status: GitConfig {
            enabled: true,
            show_dirty: true,
            show_ahead_behind: false,
            show_file_stats: false,
        },
        display: DisplayConfig {
            show_model: true,
            show_project: true,
            show_context_bar: true,
            context_value: ContextValueMode::Percent,
            show_config_counts: false,
            show_duration: false,
            show_speed: false,
            show_token_breakdown: true,
            show_usage: true,
            usage_bar_enabled: true,
            show_tools: false,
            show_agents: false,
            show_todos: false,
            show_session_name: false,
            show_claude_code_version: false,
            show_memory_usage: false,
            autocompact_buffer: AutocompactBuffer::Enabled,
            usage_threshold: 0,
            seven_day_threshold: 80,
            environment_threshold: 0,
            custom_line: String::new(),
        },
        colors: default_colors(),
        bar: BarConfig::default(),
        icons: IconConfig::default(),
        labels: LabelsConfig::default(),
        template: None,
    }
}

pub fn default_element_order() -> Vec<HudElement> {
    vec![
        HudElement::Project,
        HudElement::Context,
        HudElement::Usage,
        HudElement::Memory,
        HudElement::Environment,
        HudElement::Tools,
        HudElement::Agents,
        HudElement::Todos,
    ]
}

pub fn default_colors() -> ColorOverrides {
    ColorOverrides {
        context: ColorValue::Named(NamedColor::Green),
        usage: ColorValue::Named(NamedColor::BrightBlue),
        warning: ColorValue::Named(NamedColor::Yellow),
        usage_warning: ColorValue::Named(NamedColor::BrightMagenta),
        critical: ColorValue::Named(NamedColor::Red),
        model: ColorValue::Named(NamedColor::Cyan),
        project: ColorValue::Named(NamedColor::Yellow),
        git: ColorValue::Named(NamedColor::Magenta),
        git_branch: ColorValue::Named(NamedColor::Cyan),
        seven_day: ColorValue::Named(NamedColor::Magenta),
        label: ColorValue::Named(NamedColor::Dim),
        custom: ColorValue::Ansi256(208),
    }
}

pub fn load_config() -> HudConfig {
    let config_path = get_config_path();
    if !config_path.exists() {
        return default_config();
    }

    match std::fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(val) => merge_config(&val),
            Err(e) => {
                eprintln!(
                    "[claude-pulse] config parse error: {} ({})",
                    e,
                    config_path.display()
                );
                default_config()
            }
        },
        Err(_) => default_config(),
    }
}

fn merge_config(user: &Value) -> HudConfig {
    let def = default_config();

    let line_layout = match user.get("lineLayout").and_then(|v| v.as_str()) {
        Some("compact") => LineLayout::Compact,
        Some("expanded") => LineLayout::Expanded,
        _ => def.line_layout,
    };

    let show_separators = user
        .get("showSeparators")
        .and_then(|v| v.as_bool())
        .unwrap_or(def.show_separators);

    let path_levels = match user.get("pathLevels").and_then(|v| v.as_u64()) {
        Some(1) => 1,
        Some(2) => 2,
        Some(3) => 3,
        _ => def.path_levels,
    };

    let element_order = parse_element_order(user.get("elementOrder"));

    let gs = user.get("gitStatus");
    let git_status = GitConfig {
        enabled: gs.and_then(|g| g.get("enabled")).and_then(|v| v.as_bool()).unwrap_or(def.git_status.enabled),
        show_dirty: gs.and_then(|g| g.get("showDirty")).and_then(|v| v.as_bool()).unwrap_or(def.git_status.show_dirty),
        show_ahead_behind: gs.and_then(|g| g.get("showAheadBehind")).and_then(|v| v.as_bool()).unwrap_or(def.git_status.show_ahead_behind),
        show_file_stats: gs.and_then(|g| g.get("showFileStats")).and_then(|v| v.as_bool()).unwrap_or(def.git_status.show_file_stats),
    };

    let ds = user.get("display");
    let display = DisplayConfig {
        show_model: bool_field(ds, "showModel", def.display.show_model),
        show_project: bool_field(ds, "showProject", def.display.show_project),
        show_context_bar: bool_field(ds, "showContextBar", def.display.show_context_bar),
        context_value: match ds.and_then(|d| d.get("contextValue")).and_then(|v| v.as_str()) {
            Some("percent") => ContextValueMode::Percent,
            Some("tokens") => ContextValueMode::Tokens,
            Some("remaining") => ContextValueMode::Remaining,
            Some("both") => ContextValueMode::Both,
            _ => def.display.context_value,
        },
        show_config_counts: bool_field(ds, "showConfigCounts", def.display.show_config_counts),
        show_duration: bool_field(ds, "showDuration", def.display.show_duration),
        show_speed: bool_field(ds, "showSpeed", def.display.show_speed),
        show_token_breakdown: bool_field(ds, "showTokenBreakdown", def.display.show_token_breakdown),
        show_usage: bool_field(ds, "showUsage", def.display.show_usage),
        usage_bar_enabled: bool_field(ds, "usageBarEnabled", def.display.usage_bar_enabled),
        show_tools: bool_field(ds, "showTools", def.display.show_tools),
        show_agents: bool_field(ds, "showAgents", def.display.show_agents),
        show_todos: bool_field(ds, "showTodos", def.display.show_todos),
        show_session_name: bool_field(ds, "showSessionName", def.display.show_session_name),
        show_claude_code_version: bool_field(ds, "showClaudeCodeVersion", def.display.show_claude_code_version),
        show_memory_usage: bool_field(ds, "showMemoryUsage", def.display.show_memory_usage),
        autocompact_buffer: match ds.and_then(|d| d.get("autocompactBuffer")).and_then(|v| v.as_str()) {
            Some("disabled") => AutocompactBuffer::Disabled,
            Some("enabled") => AutocompactBuffer::Enabled,
            _ => def.display.autocompact_buffer,
        },
        usage_threshold: ds.and_then(|d| d.get("usageThreshold")).and_then(|v| v.as_u64()).map(|v| v.min(100) as u32).unwrap_or(def.display.usage_threshold),
        seven_day_threshold: ds.and_then(|d| d.get("sevenDayThreshold")).and_then(|v| v.as_u64()).map(|v| v.min(100) as u32).unwrap_or(def.display.seven_day_threshold),
        environment_threshold: ds.and_then(|d| d.get("environmentThreshold")).and_then(|v| v.as_u64()).map(|v| v.min(100) as u32).unwrap_or(def.display.environment_threshold),
        custom_line: ds.and_then(|d| d.get("customLine")).and_then(|v| v.as_str()).map(|s| s.chars().take(80).collect()).unwrap_or(def.display.custom_line),
    };

    let cs = user.get("colors");
    let colors = ColorOverrides {
        context: parse_color(cs, "context", def.colors.context),
        usage: parse_color(cs, "usage", def.colors.usage),
        warning: parse_color(cs, "warning", def.colors.warning),
        usage_warning: parse_color(cs, "usageWarning", def.colors.usage_warning),
        critical: parse_color(cs, "critical", def.colors.critical),
        model: parse_color(cs, "model", def.colors.model),
        project: parse_color(cs, "project", def.colors.project),
        git: parse_color(cs, "git", def.colors.git),
        git_branch: parse_color(cs, "gitBranch", def.colors.git_branch),
        seven_day: parse_color(cs, "sevenDay", def.colors.seven_day),
        label: parse_color(cs, "label", def.colors.label),
        custom: parse_color(cs, "custom", def.colors.custom),
    };

    let (bar, icons) = parse_bar_and_icons(user);
    let labels = parse_labels(user);
    let template = parse_template_config(user);

    HudConfig {
        line_layout,
        show_separators,
        path_levels,
        element_order,
        git_status,
        display,
        colors,
        bar,
        icons,
        labels,
        template,
    }
}

fn parse_bar_and_icons(user: &Value) -> (BarConfig, IconConfig) {
    let bar_val = user.get("bar");
    let bar = BarConfig {
        filled: bar_val
            .and_then(|b| b.get("filled"))
            .and_then(|v| v.as_str())
            .unwrap_or("█")
            .to_string(),
        empty: bar_val
            .and_then(|b| b.get("empty"))
            .and_then(|v| v.as_str())
            .unwrap_or("░")
            .to_string(),
        width: bar_val
            .and_then(|b| b.get("width"))
            .and_then(|v| v.as_u64())
            .unwrap_or(10)
            .min(30) as u32,
    };

    let icon_val = user.get("icons");
    let def_icons = IconConfig::default();
    let icons = IconConfig {
        running: str_field(icon_val, "running", &def_icons.running),
        completed: str_field(icon_val, "completed", &def_icons.completed),
        error: str_field(icon_val, "error", &def_icons.error),
        todo_active: str_field(icon_val, "todoActive", &def_icons.todo_active),
        todo_done: str_field(icon_val, "todoDone", &def_icons.todo_done),
        dirty: str_field(icon_val, "dirty", &def_icons.dirty),
        ahead: str_field(icon_val, "ahead", &def_icons.ahead),
        behind: str_field(icon_val, "behind", &def_icons.behind),
        timer: str_field(icon_val, "timer", &def_icons.timer),
        warning: str_field(icon_val, "warning", &def_icons.warning),
    };

    (bar, icons)
}

fn parse_labels(user: &Value) -> LabelsConfig {
    let def = LabelsConfig::default();
    let ls = match user.get("labels") {
        Some(v) => v,
        None => return def,
    };
    LabelsConfig {
        context: ls.get("context").and_then(|v| v.as_str()).unwrap_or(&def.context).to_string(),
        usage: ls.get("usage").and_then(|v| v.as_str()).unwrap_or(&def.usage).to_string(),
        seven_day: ls.get("sevenDay").and_then(|v| v.as_str()).unwrap_or(&def.seven_day).to_string(),
        memory: ls.get("memory").and_then(|v| v.as_str()).unwrap_or(&def.memory).to_string(),
    }
}

fn parse_template_config(user: &Value) -> Option<TemplateConfig> {
    let lines_val = user.get("lines").and_then(|v| v.as_array());
    let lines: Vec<String> = match lines_val {
        Some(arr) if !arr.is_empty() => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => return None,
    };

    let rules = user
        .get("rules")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(parse_display_rule)
                .collect()
        })
        .unwrap_or_default();

    Some(TemplateConfig {
        lines,
        rules,
    })
}

fn parse_display_rule(rule: &Value) -> Option<DisplayRule> {
    // { "show": "token_breakdown", "when": "context_pct >= 85" }
    let when_str = rule.get("when").and_then(|v| v.as_str())?;
    let target = rule.get("show").and_then(|v| v.as_str())?.to_string();

    // Parse "variable op value"
    let parts: Vec<&str> = when_str.split_whitespace().collect();
    if parts.len() != 3 {
        return None;
    }

    let op = match parts[1] {
        ">" => RuleOp::Gt,
        ">=" => RuleOp::Gte,
        "<" => RuleOp::Lt,
        "<=" => RuleOp::Lte,
        "==" => RuleOp::Eq,
        "!=" => RuleOp::Neq,
        _ => return None,
    };

    let value: f64 = parts[2].parse().ok()?;

    Some(DisplayRule { target, op, value })
}

fn str_field(parent: Option<&Value>, key: &str, default: &str) -> String {
    parent
        .and_then(|p| p.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

fn bool_field(parent: Option<&Value>, key: &str, default: bool) -> bool {
    parent
        .and_then(|p| p.get(key))
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

fn parse_color(colors: Option<&Value>, key: &str, default: ColorValue) -> ColorValue {
    let val = match colors.and_then(|c| c.get(key)) {
        Some(v) => v,
        None => return default,
    };

    if let Some(n) = val.as_u64() {
        if n <= 255 {
            return ColorValue::Ansi256(n as u8);
        }
    }

    if let Some(s) = val.as_str() {
        if s.starts_with('#') && s.len() == 7 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&s[1..3], 16),
                u8::from_str_radix(&s[3..5], 16),
                u8::from_str_radix(&s[5..7], 16),
            ) {
                return ColorValue::Hex(r, g, b);
            }
        }

        match s {
            "dim" => return ColorValue::Named(NamedColor::Dim),
            "red" => return ColorValue::Named(NamedColor::Red),
            "green" => return ColorValue::Named(NamedColor::Green),
            "yellow" => return ColorValue::Named(NamedColor::Yellow),
            "magenta" => return ColorValue::Named(NamedColor::Magenta),
            "cyan" => return ColorValue::Named(NamedColor::Cyan),
            "brightBlue" => return ColorValue::Named(NamedColor::BrightBlue),
            "brightMagenta" => return ColorValue::Named(NamedColor::BrightMagenta),
            _ => {}
        }
    }

    default
}

fn parse_element_order(val: Option<&Value>) -> Vec<HudElement> {
    let arr = match val.and_then(|v| v.as_array()) {
        Some(a) if !a.is_empty() => a,
        _ => return default_element_order(),
    };

    let mut seen = std::collections::HashSet::new();
    let mut order = Vec::new();

    for item in arr {
        if let Some(s) = item.as_str() {
            let elem = match s {
                "project" => HudElement::Project,
                "context" => HudElement::Context,
                "usage" => HudElement::Usage,
                "memory" => HudElement::Memory,
                "environment" => HudElement::Environment,
                "tools" => HudElement::Tools,
                "agents" => HudElement::Agents,
                "todos" => HudElement::Todos,
                _ => continue,
            };
            if seen.insert(elem) {
                order.push(elem);
            }
        }
    }

    if order.is_empty() {
        default_element_order()
    } else {
        order
    }
}
