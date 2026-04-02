use serde::Deserialize;
use std::time::SystemTime;

// ── Stdin JSON ──────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
pub struct StdinData {
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<ModelInfo>,
    pub context_window: Option<ContextWindow>,
    pub rate_limits: Option<RateLimits>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ModelInfo {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ContextWindow {
    pub context_window_size: Option<u64>,
    pub current_usage: Option<CurrentUsage>,
    pub used_percentage: Option<f64>,
    #[allow(dead_code)]
    pub remaining_percentage: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct CurrentUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RateLimits {
    pub five_hour: Option<RateWindow>,
    pub seven_day: Option<RateWindow>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RateWindow {
    pub used_percentage: Option<f64>,
    pub resets_at: Option<f64>,
}

// ── Transcript ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Completed,
    Error,
}

#[derive(Debug, Clone)]
pub struct ToolEntry {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    pub target: Option<String>,
    pub status: ToolStatus,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Running,
    Completed,
}

#[derive(Debug, Clone)]
pub struct AgentEntry {
    #[allow(dead_code)]
    pub id: String,
    pub agent_type: String,
    pub model: Option<String>,
    pub description: Option<String>,
    pub status: AgentStatus,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub content: String,
    pub status: TodoStatus,
}

#[derive(Debug, Default)]
pub struct TranscriptData {
    pub tools: Vec<ToolEntry>,
    pub agents: Vec<AgentEntry>,
    pub todos: Vec<TodoItem>,
    pub session_start: Option<SystemTime>,
    pub session_name: Option<String>,
}

// ── Usage ───────────────────────────────────────────────────

#[derive(Debug)]
pub struct UsageData {
    pub five_hour: Option<u32>,
    pub seven_day: Option<u32>,
    pub five_hour_reset_at: Option<SystemTime>,
    pub seven_day_reset_at: Option<SystemTime>,
}

impl UsageData {
    pub fn is_limit_reached(&self) -> bool {
        self.five_hour == Some(100) || self.seven_day == Some(100)
    }
}

// ── Memory ──────────────────────────────────────────────────

#[derive(Debug)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    #[allow(dead_code)]
    pub free_bytes: u64,
    pub used_percent: u32,
}

// ── Git ─────────────────────────────────────────────────────

#[derive(Debug)]
pub struct FileStats {
    pub modified: u32,
    pub added: u32,
    pub deleted: u32,
    pub untracked: u32,
}

#[derive(Debug)]
pub struct GitStatus {
    pub branch: String,
    pub is_dirty: bool,
    pub ahead: u32,
    pub behind: u32,
    pub file_stats: Option<FileStats>,
}

// ── Config ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HudConfig {
    pub line_layout: LineLayout,
    pub show_separators: bool,
    pub path_levels: u8,
    #[allow(dead_code)]
    pub element_order: Vec<HudElement>,
    pub git_status: GitConfig,
    pub display: DisplayConfig,
    pub colors: ColorOverrides,
    pub bar: BarConfig,
    pub icons: IconConfig,
    pub labels: LabelsConfig,
    pub template: Option<TemplateConfig>,
}

#[derive(Debug, Clone)]
pub struct LabelsConfig {
    pub context: String,
    pub usage: String,
    pub seven_day: String,
    pub memory: String,
}

impl Default for LabelsConfig {
    fn default() -> Self {
        Self {
            context: "ctx".to_string(),
            usage: "5h".to_string(),
            seven_day: "7d".to_string(),
            memory: "mem".to_string(),
        }
    }
}

// ── Template ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub lines: Vec<String>,
    pub rules: Vec<DisplayRule>,
}

#[derive(Debug, Clone)]
pub struct BarConfig {
    pub filled: String,
    pub empty: String,
    pub width: u32,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            filled: "█".to_string(),
            empty: "░".to_string(),
            width: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IconConfig {
    pub running: String,
    pub completed: String,
    pub error: String,
    pub todo_active: String,
    pub todo_done: String,
    pub dirty: String,
    pub ahead: String,
    pub behind: String,
    pub timer: String,
    pub warning: String,
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            running: "◐".to_string(),
            completed: "✓".to_string(),
            error: "✗".to_string(),
            todo_active: "▸".to_string(),
            todo_done: "✓".to_string(),
            dirty: "*".to_string(),
            ahead: "↑".to_string(),
            behind: "↓".to_string(),
            timer: "⏱️ ".to_string(),
            warning: "⚠".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DisplayRule {
    pub target: String,
    pub op: RuleOp,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuleOp {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Neq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineLayout {
    Compact,
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HudElement {
    Project,
    Context,
    Usage,
    Memory,
    Environment,
    Tools,
    Agents,
    Todos,
}

#[derive(Debug, Clone)]
pub struct GitConfig {
    pub enabled: bool,
    pub show_dirty: bool,
    pub show_ahead_behind: bool,
    pub show_file_stats: bool,
}

#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub show_model: bool,
    pub show_project: bool,
    pub show_context_bar: bool,
    pub context_value: ContextValueMode,
    pub show_config_counts: bool,
    pub show_duration: bool,
    pub show_speed: bool,
    pub show_token_breakdown: bool,
    pub show_usage: bool,
    pub usage_bar_enabled: bool,
    pub show_tools: bool,
    pub show_agents: bool,
    pub show_todos: bool,
    pub show_session_name: bool,
    pub show_claude_code_version: bool,
    pub show_memory_usage: bool,
    pub autocompact_buffer: AutocompactBuffer,
    pub usage_threshold: u32,
    pub seven_day_threshold: u32,
    pub environment_threshold: u32,
    pub custom_line: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextValueMode {
    Percent,
    Tokens,
    Remaining,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutocompactBuffer {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone)]
pub enum ColorValue {
    Named(NamedColor),
    Ansi256(u8),
    Hex(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedColor {
    Dim,
    Red,
    Green,
    Yellow,
    Magenta,
    Cyan,
    BrightBlue,
    BrightMagenta,
}

#[derive(Debug, Clone)]
pub struct ColorOverrides {
    pub context: ColorValue,
    pub usage: ColorValue,
    pub warning: ColorValue,
    pub usage_warning: ColorValue,
    pub critical: ColorValue,
    pub model: ColorValue,
    pub project: ColorValue,
    pub git: ColorValue,
    pub git_branch: ColorValue,
    pub seven_day: ColorValue,
    pub label: ColorValue,
    pub custom: ColorValue,
}

// ── Config Counts ───────────────────────────────────────────

#[derive(Debug, Default)]
pub struct ConfigCounts {
    pub claude_md_count: u32,
    pub rules_count: u32,
    pub mcp_count: u32,
    pub hooks_count: u32,
}

// ── Render Context ──────────────────────────────────────────

pub struct RenderContext {
    pub stdin: StdinData,
    pub transcript: TranscriptData,
    pub claude_md_count: u32,
    pub rules_count: u32,
    pub mcp_count: u32,
    pub hooks_count: u32,
    pub session_duration: String,
    pub git_status: Option<GitStatus>,
    pub usage_data: Option<UsageData>,
    pub memory_usage: Option<MemoryInfo>,
    pub config: HudConfig,
    pub extra_label: Option<String>,
    pub claude_code_version: Option<String>,
    pub speed: Option<f64>,
    pub terminal_width: Option<u32>,
}
