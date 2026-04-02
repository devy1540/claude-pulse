use crate::cost;
use crate::memory::format_bytes;
use crate::render::agents::render_agents_line;
use crate::render::colors::*;
use crate::render::todos::render_todos_line;
use crate::render::tools::render_tools_line;
use crate::sparkline;
use crate::stdin::*;
use crate::types::*;
use std::time::SystemTime;

// ── 퍼블릭 API ─────────────────────────────────────────────

/// 메인 렌더 함수. 항상 이 함수를 통해 출력한다.
pub fn render_template(ctx: &RenderContext) -> Vec<String> {
    let lines = match &ctx.config.template {
        Some(t) => t.lines.clone(),
        None => default_lines(&ctx.config),
    };

    let rules = ctx
        .config
        .template
        .as_ref()
        .map(|t| t.rules.as_slice())
        .unwrap_or(&[]);

    let rule_vars = collect_rule_vars(ctx);
    let mut output = Vec::new();

    for line_template in &lines {
        if let Some(rendered) = render_line(line_template, ctx, rules, &rule_vars) {
            // 멀티라인 플레이스홀더 (agents 등) 처리
            for sub in rendered.split('\n') {
                let trimmed = sub.trim();
                if !trimmed.is_empty() {
                    output.push(trimmed.to_string());
                }
            }
        }
    }

    output
}

// ── 디폴트 라인 ─────────────────────────────────────────────

fn default_lines(config: &HudConfig) -> Vec<String> {
    let d = &config.display;

    if config.line_layout == LineLayout::Compact {
        return default_compact_lines(config);
    }

    let mut lines = Vec::new();

    // Line 1: 모델 + 프로젝트 + git + 기타
    let mut parts1 = Vec::new();
    if d.show_model {
        parts1.push("{model}".to_string());
    }
    {
        let mut project_git = Vec::new();
        if d.show_project {
            project_git.push("{project}");
        }
        if config.git_status.enabled {
            project_git.push("{git}");
        }
        if !project_git.is_empty() {
            parts1.push(project_git.join(" "));
        }
    }
    if d.show_session_name {
        parts1.push("{session_name}".to_string());
    }
    if d.show_claude_code_version {
        parts1.push("{version}".to_string());
    }
    if d.show_speed {
        parts1.push("{speed}".to_string());
    }
    if d.show_duration {
        parts1.push("{duration}".to_string());
    }
    if !d.custom_line.is_empty() {
        parts1.push("{custom}".to_string());
    }
    if !parts1.is_empty() {
        lines.push(parts1.join(" │ "));
    }

    // Line 2: 컨텍스트 + 사용량
    let mut parts2 = Vec::new();
    if d.show_context_bar {
        if d.show_token_breakdown {
            parts2.push("{context} {token_breakdown}".to_string());
        } else {
            parts2.push("{context}".to_string());
        }
    }
    if d.show_usage {
        parts2.push("{usage}".to_string());
    }
    if !parts2.is_empty() {
        lines.push(parts2.join(" │ "));
    }

    // 선택적 라인
    if d.show_config_counts {
        lines.push("{env}".to_string());
    }
    if d.show_memory_usage {
        lines.push("{memory}".to_string());
    }
    if d.show_tools {
        lines.push("{tools}".to_string());
    }
    if d.show_agents {
        lines.push("{agents}".to_string());
    }
    if d.show_todos {
        lines.push("{todos}".to_string());
    }

    lines
}

fn default_compact_lines(config: &HudConfig) -> Vec<String> {
    let d = &config.display;
    let mut parts = Vec::new();

    if d.show_model && d.show_context_bar {
        parts.push("{model} {context_bar} {context_pct}".to_string());
    } else if d.show_model {
        parts.push("{model} {context_pct}".to_string());
    } else {
        parts.push("{context_bar} {context_pct}".to_string());
    }

    if d.show_project {
        if config.git_status.enabled {
            parts.push("{project} {git}".to_string());
        } else {
            parts.push("{project}".to_string());
        }
    }

    if d.show_usage {
        parts.push("{usage_bar} {usage_pct}".to_string());
    }

    if d.show_config_counts {
        parts.push("{env}".to_string());
    }
    if d.show_speed {
        parts.push("{speed}".to_string());
    }
    if d.show_duration {
        parts.push("{duration}".to_string());
    }
    if !d.custom_line.is_empty() {
        parts.push("{custom}".to_string());
    }

    let mut lines = vec![parts.join(" │ ")];

    if d.show_tools {
        lines.push("{tools}".to_string());
    }
    if d.show_agents {
        lines.push("{agents}".to_string());
    }
    if d.show_todos {
        lines.push("{todos}".to_string());
    }

    lines
}

// ── 규칙 평가 ───────────────────────────────────────────────

struct RuleVars {
    context_pct: f64,
    usage_pct: f64,
    seven_day_pct: f64,
    tools_count: f64,
    agents_count: f64,
    todos_count: f64,
    memory_pct: f64,
    speed: f64,
}

fn collect_rule_vars(ctx: &RenderContext) -> RuleVars {
    RuleVars {
        context_pct: get_context_percent(&ctx.stdin) as f64,
        usage_pct: ctx.usage_data.as_ref().and_then(|u| u.five_hour).unwrap_or(0) as f64,
        seven_day_pct: ctx.usage_data.as_ref().and_then(|u| u.seven_day).unwrap_or(0) as f64,
        tools_count: ctx.transcript.tools.len() as f64,
        agents_count: ctx.transcript.agents.len() as f64,
        todos_count: ctx.transcript.todos.len() as f64,
        memory_pct: ctx.memory_usage.as_ref().map(|m| m.used_percent as f64).unwrap_or(0.0),
        speed: ctx.speed.unwrap_or(0.0),
    }
}

fn check_rules_for(placeholder: &str, rules: &[DisplayRule], vars: &RuleVars) -> bool {
    let matching: Vec<&DisplayRule> = rules.iter().filter(|r| r.target == placeholder).collect();
    if matching.is_empty() {
        return true;
    }
    matching.iter().all(|rule| {
        let var_value = auto_var_for_target(&rule.target, vars);
        eval_condition(var_value, rule.op, rule.value)
    })
}

fn auto_var_for_target(target: &str, vars: &RuleVars) -> f64 {
    match target {
        "token_breakdown" | "context_bar" | "context_pct" | "context" | "sparkline" | "predict" => {
            vars.context_pct
        }
        "usage_bar" | "usage_pct" | "usage" | "cost" => vars.usage_pct,
        "seven_day_bar" | "seven_day_pct" | "seven_day" => vars.seven_day_pct,
        "tools" => vars.tools_count,
        "agents" => vars.agents_count,
        "todos" | "todo_bar" => vars.todos_count,
        "memory_bar" | "memory_pct" | "memory" => vars.memory_pct,
        "speed" => vars.speed,
        _ => 0.0,
    }
}

fn eval_condition(actual: f64, op: RuleOp, expected: f64) -> bool {
    match op {
        RuleOp::Gt => actual > expected,
        RuleOp::Gte => actual >= expected,
        RuleOp::Lt => actual < expected,
        RuleOp::Lte => actual <= expected,
        RuleOp::Eq => (actual - expected).abs() < f64::EPSILON,
        RuleOp::Neq => (actual - expected).abs() >= f64::EPSILON,
    }
}

// ── 템플릿 파싱 ─────────────────────────────────────────────

enum Segment<'a> {
    Literal(&'a str),
    Placeholder(&'a str),
}

fn parse_segments(template: &str) -> Vec<Segment<'_>> {
    let mut segments = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        if open > 0 {
            segments.push(Segment::Literal(&rest[..open]));
        }
        let after = &rest[open + 1..];
        if let Some(close) = after.find('}') {
            segments.push(Segment::Placeholder(&after[..close]));
            rest = &after[close + 1..];
        } else {
            segments.push(Segment::Literal(&rest[open..]));
            rest = "";
            break;
        }
    }
    if !rest.is_empty() {
        segments.push(Segment::Literal(rest));
    }
    segments
}

fn render_line(
    template: &str,
    ctx: &RenderContext,
    rules: &[DisplayRule],
    rule_vars: &RuleVars,
) -> Option<String> {
    let segments = parse_segments(template);
    let mut result = String::new();
    let mut has_content = false;

    for seg in &segments {
        match seg {
            Segment::Literal(text) => result.push_str(text),
            Segment::Placeholder(name) => {
                if !check_rules_for(name, rules, rule_vars) {
                    continue;
                }
                if let Some(value) = resolve(name, ctx) {
                    if !strip_ansi_simple(&value).trim().is_empty() {
                        result.push_str(&value);
                        has_content = true;
                    }
                }
            }
        }
    }

    if has_content {
        Some(collapse_separators(&result))
    } else {
        None
    }
}

/// 연속된 구분자(│, |)를 하나로 합치고, 줄 앞뒤 구분자를 제거
fn collapse_separators(s: &str) -> String {
    let mut result = s.to_string();

    // 연속 구분자 축소: " │ │ " → " │ ", " | | " → " | "
    loop {
        let prev = result.clone();
        result = result.replace(" │ │ ", " │ ");
        result = result.replace(" | | ", " | ");
        if result == prev {
            break;
        }
    }

    // 줄 끝 구분자 제거
    let trimmed = strip_ansi_simple(&result);
    let trimmed_end = trimmed.trim_end();
    if trimmed_end.ends_with('│') || trimmed_end.ends_with('|') {
        // visible 텍스트에서 마지막 구분자 위치를 찾아 원본에서 제거
        if let Some(pos) = result.rfind('│') {
            let after = &result[pos + '│'.len_utf8()..];
            if strip_ansi_simple(after).trim().is_empty() {
                result = result[..pos].to_string() + after;
            }
        } else if let Some(pos) = result.rfind('|') {
            let after = &result[pos + 1..];
            if strip_ansi_simple(after).trim().is_empty() {
                result = result[..pos].to_string() + after;
            }
        }
    }

    // 줄 시작 구분자 제거
    let trimmed_start = strip_ansi_simple(&result);
    let trimmed_start = trimmed_start.trim_start();
    if trimmed_start.starts_with('│') || trimmed_start.starts_with('|') {
        if let Some(pos) = result.find('│') {
            let before = &result[..pos];
            if strip_ansi_simple(before).trim().is_empty() {
                result = before.to_string() + &result[pos + '│'.len_utf8()..];
            }
        } else if let Some(pos) = result.find('|') {
            let before = &result[..pos];
            if strip_ansi_simple(before).trim().is_empty() {
                result = before.to_string() + &result[pos + 1..];
            }
        }
    }

    result.trim().to_string()
}

fn strip_ansi_simple(s: &str) -> String {
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

// ── 플레이스홀더 해석 ───────────────────────────────────────

fn resolve(name: &str, ctx: &RenderContext) -> Option<String> {
    let colors = &ctx.config.colors;
    let bar = &ctx.config.bar;
    let icons = &ctx.config.icons;
    let labels = &ctx.config.labels;

    match name {
        // ── 모델/프로젝트 ──
        "model" => {
            let model = get_model_name(&ctx.stdin);
            let provider = get_provider_label(&ctx.stdin);
            let qualifier = provider.map(String::from).or_else(|| {
                if ctx.config.display.show_usage && std::env::var("ANTHROPIC_API_KEY").is_ok() {
                    Some(red("API"))
                } else {
                    None
                }
            });
            let display = match qualifier {
                Some(q) => format!("{model} | {q}"),
                None => model,
            };
            Some(model_color(&format!("[{display}]"), colors))
        }

        "model_name" => Some(get_model_name(&ctx.stdin)),

        "project" => {
            let cwd = ctx.stdin.cwd.as_ref()?;
            let segs: Vec<&str> = cwd.split(&['/', '\\'][..]).filter(|s| !s.is_empty()).collect();
            let path = if segs.is_empty() {
                "/".to_string()
            } else {
                let start = segs.len().saturating_sub(ctx.config.path_levels as usize);
                segs[start..].join("/")
            };
            Some(project_color(&path, colors))
        }

        "git" => {
            let gs = ctx.git_status.as_ref()?;
            if !ctx.config.git_status.enabled {
                return None;
            }
            let mut parts = vec![gs.branch.clone()];
            if ctx.config.git_status.show_dirty && gs.is_dirty {
                parts.push(icons.dirty.clone());
            }
            if ctx.config.git_status.show_ahead_behind {
                if gs.ahead > 0 {
                    parts.push(format!(" {}{}", icons.ahead, gs.ahead));
                }
                if gs.behind > 0 {
                    parts.push(format!(" {}{}", icons.behind, gs.behind));
                }
            }
            if ctx.config.git_status.show_file_stats {
                if let Some(fs) = &gs.file_stats {
                    let mut sp = Vec::new();
                    if fs.modified > 0 { sp.push(format!("!{}", fs.modified)); }
                    if fs.added > 0 { sp.push(format!("+{}", fs.added)); }
                    if fs.deleted > 0 { sp.push(format!("✘{}", fs.deleted)); }
                    if fs.untracked > 0 { sp.push(format!("?{}", fs.untracked)); }
                    if !sp.is_empty() {
                        parts.push(format!(" {}", sp.join(" ")));
                    }
                }
            }
            Some(format!(
                "{}{}{}",
                git_color("git:(", colors),
                git_branch_color(&parts.join(""), colors),
                git_color(")", colors)
            ))
        }

        // ── 컨텍스트 ──
        "context_bar" => {
            let pct = effective_context_pct(ctx);
            Some(custom_bar(pct, bar, |p| get_context_color(p, colors)))
        }

        "context_pct" => {
            let pct = effective_context_pct(ctx);
            let color = get_context_color(pct, colors);
            Some(format!("{color}{pct}%{RESET}"))
        }

        "context" => {
            let pct = effective_context_pct(ctx);
            let bar_str = custom_bar(pct, bar, |p| get_context_color(p, colors));
            let color = get_context_color(pct, colors);
            Some(format!(
                "{} {} {color}{pct}%{RESET}",
                label_color(&labels.context, colors),
                bar_str,
            ))
        }

        "token_breakdown" => {
            let pct = effective_context_pct(ctx);
            if pct < 85 && ctx.config.template.is_none() {
                // 디폴트 모드에서는 85% 이상일 때만
                return None;
            }
            let cw = ctx.stdin.context_window.as_ref()?;
            let usage = cw.current_usage.as_ref()?;
            let input = format_tokens(usage.input_tokens.unwrap_or(0));
            let cache = format_tokens(
                usage.cache_creation_input_tokens.unwrap_or(0)
                    + usage.cache_read_input_tokens.unwrap_or(0),
            );
            Some(label_color(&format!("in: {input}, cache: {cache}"), colors))
        }

        // ── 사용량 ──
        "usage_bar" => {
            let pct = ctx.usage_data.as_ref().and_then(|u| u.five_hour).unwrap_or(0);
            Some(custom_bar(pct, bar, |p| get_quota_color(p, colors)))
        }

        "usage_pct" => match ctx.usage_data.as_ref().and_then(|u| u.five_hour) {
            Some(p) => {
                let color = get_quota_color(p, colors);
                Some(format!("{color}{p}%{RESET}"))
            }
            None => Some(label_color("--%", colors)),
        },

        "usage" => {
            if let Some(d) = ctx.usage_data.as_ref() {
                if d.is_limit_reached() {
                    let reset = format_reset_time(if d.five_hour == Some(100) {
                        d.five_hour_reset_at
                    } else {
                        d.seven_day_reset_at
                    });
                    let msg = if reset.is_empty() {
                        format!("{} Limit reached", icons.warning)
                    } else {
                        format!("{} Limit reached (resets {reset})", icons.warning)
                    };
                    return Some(format!(
                        "{} {}",
                        label_color(&labels.usage, colors),
                        critical_color(&msg, colors)
                    ));
                }
                let pct = d.five_hour.unwrap_or(0);
                let bar_str = custom_bar(pct, bar, |p| get_quota_color(p, colors));
                let color = get_quota_color(pct, colors);
                let reset = format_reset_time(d.five_hour_reset_at);
                let reset_part = if reset.is_empty() {
                    String::new()
                } else {
                    format!(" ({reset})")
                };
                Some(format!(
                    "{} {} {color}{pct}%{RESET}{reset_part}",
                    label_color(&labels.usage, colors),
                    bar_str,
                ))
            } else {
                let bar_str = custom_bar(0, bar, |p| get_quota_color(p, colors));
                Some(format!(
                    "{} {} {}",
                    label_color(&labels.usage, colors),
                    bar_str,
                    label_color("--%", colors),
                ))
            }
        }

        "usage_reset" => {
            let reset = format_reset_time(ctx.usage_data.as_ref()?.five_hour_reset_at);
            if reset.is_empty() { None } else { Some(reset) }
        }

        "seven_day_bar" => {
            let pct = ctx.usage_data.as_ref().and_then(|u| u.seven_day).unwrap_or(0);
            Some(custom_bar(pct, bar, |p| get_seven_day_color(p, colors)))
        }

        "seven_day_pct" => match ctx.usage_data.as_ref().and_then(|u| u.seven_day) {
            Some(p) => {
                let color = get_seven_day_color(p, colors);
                Some(format!("{color}{p}%{RESET}"))
            }
            None => Some(label_color("--%", colors)),
        },

        "seven_day" => {
            if let Some(d) = ctx.usage_data.as_ref() {
                let pct = d.seven_day.unwrap_or(0);
                let bar_str = custom_bar(pct, bar, |p| get_seven_day_color(p, colors));
                let color = get_seven_day_color(pct, colors);
                let reset = format_reset_time(d.seven_day_reset_at);
                let reset_part = if reset.is_empty() {
                    String::new()
                } else {
                    format!(" ({reset})")
                };
                Some(format!("{} {bar_str} {color}{pct}%{RESET}{reset_part}", label_color(&labels.seven_day, colors)))
            } else {
                let bar_str = custom_bar(0, bar, |p| get_seven_day_color(p, colors));
                Some(format!(
                    "{} {} {}",
                    label_color(&labels.seven_day, colors),
                    bar_str,
                    label_color("--%", colors),
                ))
            }
        }

        // ── 활동 ──
        "tools" => render_tools_line(ctx),
        "agents" => render_agents_line(ctx),
        "todos" => render_todos_line(ctx),

        // ── TODO 프로그레스 바 ── (신규)
        "todo_bar" => {
            let todos = &ctx.transcript.todos;
            if todos.is_empty() {
                return None;
            }
            let completed = todos.iter().filter(|t| t.status == TodoStatus::Completed).count() as u32;
            let total = todos.len() as u32;
            let pct = if total > 0 { (completed * 100) / total } else { 0 };
            let filled = ((pct as f64 / 100.0) * 6.0).round() as usize;
            let empty = 6usize.saturating_sub(filled);
            let bar_str = format!(
                "[{}{}]",
                bar.filled.repeat(filled),
                bar.empty.repeat(empty)
            );
            let color = if completed == total {
                get_context_color(0, colors) // green
            } else {
                get_context_color(75, colors) // yellow
            };
            Some(format!(
                "{color}{bar_str}{RESET} {completed}/{total}"
            ))
        }

        // ── 환경 ──
        "env" => {
            let mut parts = Vec::new();
            if ctx.claude_md_count > 0 { parts.push(format!("{} CLAUDE.md", ctx.claude_md_count)); }
            if ctx.rules_count > 0 { parts.push(format!("{} rules", ctx.rules_count)); }
            if ctx.mcp_count > 0 { parts.push(format!("{} MCPs", ctx.mcp_count)); }
            if ctx.hooks_count > 0 { parts.push(format!("{} hooks", ctx.hooks_count)); }
            if parts.is_empty() { None } else { Some(label_color(&parts.join(" | "), colors)) }
        }

        "claude_md" => if ctx.claude_md_count > 0 { Some(format!("{}", ctx.claude_md_count)) } else { None },
        "rules" => if ctx.rules_count > 0 { Some(format!("{}", ctx.rules_count)) } else { None },
        "mcps" => if ctx.mcp_count > 0 { Some(format!("{}", ctx.mcp_count)) } else { None },
        "hooks" => if ctx.hooks_count > 0 { Some(format!("{}", ctx.hooks_count)) } else { None },

        // ── 메모리 ──
        "memory_bar" => {
            let mem = ctx.memory_usage.as_ref()?;
            Some(custom_bar(mem.used_percent, bar, |p| get_quota_color(p, colors)))
        }
        "memory_pct" => {
            let mem = ctx.memory_usage.as_ref()?;
            let color = get_quota_color(mem.used_percent, colors);
            Some(format!("{color}{}%{RESET}", mem.used_percent))
        }
        "memory_used" => Some(format_bytes(ctx.memory_usage.as_ref()?.used_bytes)),
        "memory_total" => Some(format_bytes(ctx.memory_usage.as_ref()?.total_bytes)),
        "memory" => {
            let mem = ctx.memory_usage.as_ref()?;
            let bar_str = custom_bar(mem.used_percent, bar, |p| get_quota_color(p, colors));
            let color = get_quota_color(mem.used_percent, colors);
            Some(format!(
                "{} {} {} / {} ({color}{}%{RESET})",
                label_color(&labels.memory, colors),
                bar_str,
                format_bytes(mem.used_bytes),
                format_bytes(mem.total_bytes),
                mem.used_percent,
            ))
        }

        // ── 스파크라인 (신규) ──
        "sparkline" => {
            let pct = effective_context_pct(ctx);
            let session_id = ctx.stdin.transcript_path.as_deref().unwrap_or("default");
            sparkline::record_and_render(session_id, pct)
        }

        // ── 비용 추정 (신규) ──
        "cost" => Some(cost::estimate_cost(&ctx.stdin).unwrap_or_else(|| "~$0.00".to_string())),

        // ── autocompact 예측 (신규) ──
        "predict" => {
            let pct = effective_context_pct(ctx);
            if pct < 10 {
                return None;
            }
            // autocompact는 보통 ~80% 에서 발생
            let remaining = 80u32.saturating_sub(pct);
            if remaining == 0 {
                return Some(label_color("autocompact soon", colors));
            }
            // 현재 세션 시작부터의 메시지 수 (transcript tool_use 수 기준 추정)
            let tool_count = ctx.transcript.tools.len() as u32;
            if tool_count < 3 || pct < 5 {
                return None;
            }
            let msgs_per_pct = tool_count as f64 / pct as f64;
            let msgs_left = (remaining as f64 * msgs_per_pct).round() as u32;
            if msgs_left > 0 {
                Some(label_color(&format!("~{msgs_left} msgs left"), colors))
            } else {
                None
            }
        }

        // ── 기타 ──
        "speed" => {
            let speed = ctx.speed?;
            if speed >= 1000.0 {
                Some(label_color(&format!("~{:.1}k tok/s", speed / 1000.0), colors))
            } else {
                Some(label_color(&format!("~{:.0} tok/s", speed), colors))
            }
        }

        "duration" => {
            if ctx.session_duration.is_empty() {
                None
            } else {
                Some(label_color(
                    &format!("{}{}", icons.timer, ctx.session_duration),
                    colors,
                ))
            }
        }

        "version" => {
            let ver = ctx.claude_code_version.as_ref()?;
            Some(label_color(&format!("CC v{ver}"), colors))
        }

        "session_name" => {
            let name = ctx.transcript.session_name.as_ref()?;
            Some(label_color(name, colors))
        }

        "custom" => {
            let line = &ctx.config.display.custom_line;
            if line.is_empty() { None } else { Some(custom_color(line, colors)) }
        }

        "extra" => ctx.extra_label.as_ref().map(|l| label_color(l, colors)),

        _ => None,
    }
}

// ── 헬퍼 ────────────────────────────────────────────────────

fn effective_context_pct(ctx: &RenderContext) -> u32 {
    if ctx.config.display.autocompact_buffer == AutocompactBuffer::Disabled {
        get_context_percent(&ctx.stdin)
    } else {
        get_buffered_percent(&ctx.stdin)
    }
}

fn custom_bar<F: Fn(u32) -> String>(percent: u32, cfg: &BarConfig, color_fn: F) -> String {
    let p = percent.min(100);
    let filled = ((p as f64 / 100.0) * cfg.width as f64).round() as u32;
    let empty = cfg.width.saturating_sub(filled);
    let color = color_fn(p);
    format!(
        "{color}{filled_bar}\x1b[2m{empty_bar}{RESET}",
        filled_bar = cfg.filled.repeat(filled as usize),
        empty_bar = cfg.empty.repeat(empty as usize),
    )
}

fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{}k", n / 1_000)
    } else {
        n.to_string()
    }
}

fn format_reset_time(reset_at: Option<SystemTime>) -> String {
    let reset_at = match reset_at {
        Some(t) => t,
        None => return String::new(),
    };
    let now = SystemTime::now();
    let diff = match reset_at.duration_since(now) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let total_mins = diff.as_secs().div_ceil(60);
    if total_mins < 60 {
        return format!("{total_mins}m");
    }
    let hours = total_mins / 60;
    let mins = total_mins % 60;
    if hours >= 24 {
        let days = hours / 24;
        let rem = hours % 24;
        if rem > 0 { format!("{days}d {rem}h") } else { format!("{days}d") }
    } else if mins > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{hours}h")
    }
}
