use crate::types::{ColorOverrides, ColorValue, NamedColor};

pub const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const BRIGHT_BLUE: &str = "\x1b[94m";
const BRIGHT_MAGENTA: &str = "\x1b[95m";

fn named_to_ansi(name: NamedColor) -> &'static str {
    match name {
        NamedColor::Dim => DIM,
        NamedColor::Red => RED,
        NamedColor::Green => GREEN,
        NamedColor::Yellow => YELLOW,
        NamedColor::Magenta => MAGENTA,
        NamedColor::Cyan => CYAN,
        NamedColor::BrightBlue => BRIGHT_BLUE,
        NamedColor::BrightMagenta => BRIGHT_MAGENTA,
    }
}

pub fn resolve_ansi(value: &ColorValue) -> String {
    match value {
        ColorValue::Named(name) => named_to_ansi(*name).to_string(),
        ColorValue::Ansi256(n) => format!("\x1b[38;5;{n}m"),
        ColorValue::Hex(r, g, b) => format!("\x1b[38;2;{r};{g};{b}m"),
    }
}

fn colorize(text: &str, color: &str) -> String {
    format!("{color}{text}{RESET}")
}

fn with_override(text: &str, value: &ColorValue) -> String {
    colorize(text, &resolve_ansi(value))
}

pub fn green(text: &str) -> String {
    colorize(text, GREEN)
}

pub fn yellow(text: &str) -> String {
    colorize(text, YELLOW)
}

pub fn red(text: &str) -> String {
    colorize(text, RED)
}

pub fn cyan(text: &str) -> String {
    colorize(text, CYAN)
}

pub fn magenta(text: &str) -> String {
    colorize(text, MAGENTA)
}

pub fn model_color(text: &str, colors: &ColorOverrides) -> String {
    with_override(text, &colors.model)
}

pub fn project_color(text: &str, colors: &ColorOverrides) -> String {
    with_override(text, &colors.project)
}

pub fn git_color(text: &str, colors: &ColorOverrides) -> String {
    with_override(text, &colors.git)
}

pub fn git_branch_color(text: &str, colors: &ColorOverrides) -> String {
    with_override(text, &colors.git_branch)
}

pub fn label_color(text: &str, colors: &ColorOverrides) -> String {
    with_override(text, &colors.label)
}

pub fn custom_color(text: &str, colors: &ColorOverrides) -> String {
    with_override(text, &colors.custom)
}

pub fn critical_color(text: &str, colors: &ColorOverrides) -> String {
    with_override(text, &colors.critical)
}

pub fn get_context_color(percent: u32, colors: &ColorOverrides) -> String {
    if percent >= 85 {
        resolve_ansi(&colors.critical)
    } else if percent >= 70 {
        resolve_ansi(&colors.warning)
    } else {
        resolve_ansi(&colors.context)
    }
}

pub fn get_quota_color(percent: u32, colors: &ColorOverrides) -> String {
    if percent >= 90 {
        resolve_ansi(&colors.critical)
    } else if percent >= 75 {
        resolve_ansi(&colors.usage_warning)
    } else {
        resolve_ansi(&colors.usage)
    }
}

pub fn get_seven_day_color(percent: u32, colors: &ColorOverrides) -> String {
    if percent >= 90 {
        resolve_ansi(&colors.critical)
    } else if percent >= 75 {
        resolve_ansi(&colors.usage_warning)
    } else {
        resolve_ansi(&colors.seven_day)
    }
}

