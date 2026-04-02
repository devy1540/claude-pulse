mod config;
mod config_reader;
mod cost;
mod extra_cmd;
mod git;
mod memory;
mod render;
mod sparkline;
mod speed;
mod stdin;
mod terminal;
mod transcript;
mod types;
mod version;

use types::*;

fn main() {
    // --version 플래그 지원
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("claude-pulse {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let stdin_data = match stdin::read_stdin() {
        Some(data) => data,
        None => {
            println!("[claude-pulse] Initializing...");
            if cfg!(target_os = "macos") {
                println!("[claude-pulse] Note: On macOS, you may need to restart Claude Code for the HUD to appear.");
            }
            return;
        }
    };

    if let Err(e) = run(stdin_data) {
        eprintln!("[claude-pulse] {e}");
    }
}

fn run(stdin_data: StdinData) -> Result<(), String> {
    let transcript_path = stdin_data.transcript_path.as_deref().unwrap_or("");
    let transcript = transcript::parse_transcript(transcript_path);

    let cwd = stdin_data.cwd.as_deref();
    let counts = config_reader::count_configs(cwd);

    let terminal_width = terminal::get_terminal_width();
    let mut config = config::load_config();

    // Adaptive bar width
    if let Some(tw) = terminal_width {
        if tw < 60 {
            config.bar.width = config.bar.width.min(4);
        } else if tw < 100 {
            config.bar.width = config.bar.width.min(6);
        }
    }

    let git_status = if config.git_status.enabled {
        git::get_git_status(cwd)
    } else {
        None
    };

    // 템플릿 모드에서는 {usage}가 lines에 포함되어 있으면 항상 파싱
    let usage_data = if config.display.show_usage || config.template.is_some() {
        stdin::get_usage_from_stdin(&stdin_data)
    } else {
        None
    };

    let session_duration = format_session_duration(&transcript);

    let claude_code_version = if config.display.show_claude_code_version
        || has_placeholder(&config, "version")
    {
        version::get_claude_code_version()
    } else {
        None
    };

    let memory_usage = if config.display.show_memory_usage || has_placeholder(&config, "memory") {
        memory::get_memory_usage()
    } else {
        None
    };

    let speed = if config.display.show_speed || has_placeholder(&config, "speed") {
        speed::get_output_speed(&stdin_data)
    } else {
        None
    };

    let extra_label = extra_cmd::parse_extra_cmd_arg()
        .and_then(|cmd| extra_cmd::run_extra_cmd(&cmd));

    let ctx = RenderContext {
        stdin: stdin_data,
        transcript,
        claude_md_count: counts.claude_md_count,
        rules_count: counts.rules_count,
        mcp_count: counts.mcp_count,
        hooks_count: counts.hooks_count,
        session_duration,
        git_status,
        usage_data,
        memory_usage,
        config,
        extra_label,
        claude_code_version,
        speed,
        terminal_width,
    };

    render::render(&ctx);
    Ok(())
}

/// 템플릿 lines에 특정 플레이스홀더가 포함되어 있는지 확인
fn has_placeholder(config: &HudConfig, name: &str) -> bool {
    config
        .template
        .as_ref()
        .map(|t| {
            let pattern = format!("{{{name}}}");
            t.lines.iter().any(|l| l.contains(&pattern))
        })
        .unwrap_or(false)
}

fn format_session_duration(transcript: &TranscriptData) -> String {
    let start = match transcript.session_start {
        Some(t) => t,
        None => return String::new(),
    };

    let elapsed = match std::time::SystemTime::now().duration_since(start) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };

    let mins = elapsed.as_secs() / 60;
    if mins < 1 {
        "<1m".to_string()
    } else if mins < 60 {
        format!("{mins}m")
    } else {
        let hours = mins / 60;
        let remaining = mins % 60;
        format!("{hours}h {remaining}m")
    }
}
