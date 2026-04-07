use crate::config::get_hud_plugin_dir;
use crate::types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ── 캐시 타입 ──────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct TranscriptFileCache {
    path: String,
    size: u64,
    #[serde(default)]
    mtime_ms: u64, // 하위호환용, 더 이상 체크 안 함
    tools: Vec<CachedTool>,
    agents: Vec<CachedAgent>,
    todos: Vec<CachedTodo>,
    session_start_ms: Option<u64>,
    session_name: Option<String>,
    #[serde(default)]
    task_ids: HashMap<String, usize>,
}

#[derive(Serialize, Deserialize)]
struct CachedTool {
    id: String,
    name: String,
    target: Option<String>,
    status: u8,
    start_ms: u64,
    end_ms: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct CachedAgent {
    id: String,
    agent_type: String,
    model: Option<String>,
    description: Option<String>,
    status: u8,
    start_ms: u64,
    end_ms: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct CachedTodo {
    content: String,
    status: u8,
}

fn system_time_to_ms(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn ms_to_system_time(ms: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(ms)
}

fn cache_path() -> std::path::PathBuf {
    get_hud_plugin_dir().join(".transcript-cache.json")
}

// ── 파싱 상태 ──────────────────────────────────────────────

struct ParseState {
    tool_map: HashMap<String, ToolEntry>,
    agent_map: HashMap<String, AgentEntry>,
    latest_todos: Vec<TodoItem>,
    task_id_to_index: HashMap<String, usize>,
    latest_slug: Option<String>,
    custom_title: Option<String>,
    session_start: Option<SystemTime>,
}

impl Default for ParseState {
    fn default() -> Self {
        Self {
            tool_map: HashMap::new(),
            agent_map: HashMap::new(),
            latest_todos: Vec::new(),
            task_id_to_index: HashMap::new(),
            latest_slug: None,
            custom_title: None,
            session_start: None,
        }
    }
}

impl ParseState {
    /// 캐시에서 파싱 상태를 복원한다 (증분 파싱용).
    fn from_cache(cache: &TranscriptFileCache) -> Self {
        let mut tool_map = HashMap::new();
        for t in &cache.tools {
            tool_map.insert(
                t.id.clone(),
                ToolEntry {
                    id: t.id.clone(),
                    name: t.name.clone(),
                    target: t.target.clone(),
                    status: match t.status {
                        0 => ToolStatus::Running,
                        2 => ToolStatus::Error,
                        _ => ToolStatus::Completed,
                    },
                    start_time: ms_to_system_time(t.start_ms),
                    end_time: t.end_ms.map(ms_to_system_time),
                },
            );
        }

        let mut agent_map = HashMap::new();
        for a in &cache.agents {
            agent_map.insert(
                a.id.clone(),
                AgentEntry {
                    id: a.id.clone(),
                    agent_type: a.agent_type.clone(),
                    model: a.model.clone(),
                    description: a.description.clone(),
                    status: if a.status == 0 {
                        AgentStatus::Running
                    } else {
                        AgentStatus::Completed
                    },
                    start_time: ms_to_system_time(a.start_ms),
                    end_time: a.end_ms.map(ms_to_system_time),
                },
            );
        }

        let latest_todos: Vec<TodoItem> = cache
            .todos
            .iter()
            .map(|t| TodoItem {
                content: t.content.clone(),
                status: match t.status {
                    1 => TodoStatus::InProgress,
                    2 => TodoStatus::Completed,
                    _ => TodoStatus::Pending,
                },
            })
            .collect();

        ParseState {
            tool_map,
            agent_map,
            latest_todos,
            task_id_to_index: cache.task_ids.clone(),
            latest_slug: None,
            custom_title: cache.session_name.clone(),
            session_start: cache.session_start_ms.map(ms_to_system_time),
        }
    }

    fn finalize(self) -> TranscriptData {
        let mut tools: Vec<ToolEntry> = self.tool_map.into_values().collect();
        tools.sort_by_key(|t| t.start_time);
        let len = tools.len();
        if len > 20 {
            tools = tools.split_off(len - 20);
        }

        let mut agents: Vec<AgentEntry> = self.agent_map.into_values().collect();
        agents.sort_by_key(|a| a.start_time);
        let len = agents.len();
        if len > 10 {
            agents = agents.split_off(len - 10);
        }

        TranscriptData {
            tools,
            agents,
            todos: self.latest_todos,
            session_start: self.session_start,
            session_name: self.custom_title.or(self.latest_slug),
        }
    }
}

// ── 캐시 로드/저장 ─────────────────────────────────────────

fn load_cache(transcript_path: &str) -> Option<TranscriptFileCache> {
    let content = std::fs::read_to_string(cache_path()).ok()?;
    let cache: TranscriptFileCache = serde_json::from_str(&content).ok()?;
    if cache.path != transcript_path {
        return None;
    }
    Some(cache)
}

fn save_cache(
    transcript_path: &str,
    file_size: u64,
    data: &TranscriptData,
    task_ids: &HashMap<String, usize>,
) {
    let cache = TranscriptFileCache {
        path: transcript_path.to_string(),
        mtime_ms: 0,
        size: file_size,
        tools: data
            .tools
            .iter()
            .map(|t| CachedTool {
                id: t.id.clone(),
                name: t.name.clone(),
                target: t.target.clone(),
                status: match t.status {
                    ToolStatus::Running => 0,
                    ToolStatus::Completed => 1,
                    ToolStatus::Error => 2,
                },
                start_ms: system_time_to_ms(t.start_time),
                end_ms: t.end_time.map(system_time_to_ms),
            })
            .collect(),
        agents: data
            .agents
            .iter()
            .map(|a| CachedAgent {
                id: a.id.clone(),
                agent_type: a.agent_type.clone(),
                model: a.model.clone(),
                description: a.description.clone(),
                status: match a.status {
                    AgentStatus::Running => 0,
                    AgentStatus::Completed => 1,
                },
                start_ms: system_time_to_ms(a.start_time),
                end_ms: a.end_time.map(system_time_to_ms),
            })
            .collect(),
        todos: data
            .todos
            .iter()
            .map(|t| CachedTodo {
                content: t.content.clone(),
                status: match t.status {
                    TodoStatus::Pending => 0,
                    TodoStatus::InProgress => 1,
                    TodoStatus::Completed => 2,
                },
            })
            .collect(),
        session_start_ms: data.session_start.map(system_time_to_ms),
        session_name: data.session_name.clone(),
        task_ids: task_ids.clone(),
    };

    let p = cache_path();
    let _ = std::fs::create_dir_all(p.parent().unwrap());
    let _ = std::fs::write(&p, serde_json::to_string(&cache).unwrap_or_default());
}

// ── 퍼블릭 API ─────────────────────────────────────────────

pub fn parse_transcript(transcript_path: &str) -> TranscriptData {
    if transcript_path.is_empty() || !Path::new(transcript_path).exists() {
        return TranscriptData::default();
    }

    let current_size = match std::fs::metadata(transcript_path) {
        Ok(m) => m.len(),
        Err(_) => return TranscriptData::default(),
    };

    if let Some(cache) = load_cache(transcript_path) {
        if cache.size == current_size {
            // 파일 변화 없음 → 캐시 히트
            return ParseState::from_cache(&cache).finalize();
        }
        if current_size > cache.size {
            // 파일이 커짐 → 증분 파싱 (새 라인만 파싱)
            let mut state = ParseState::from_cache(&cache);
            parse_lines(transcript_path, cache.size, &mut state);
            let task_ids = state.task_id_to_index.clone();
            let data = state.finalize();
            save_cache(transcript_path, current_size, &data, &task_ids);
            return data;
        }
        // 파일이 작아짐 (세션 재시작 등) → 전체 재파싱
    }

    // 전체 재파싱
    let mut state = ParseState::default();
    parse_lines(transcript_path, 0, &mut state);
    let task_ids = state.task_id_to_index.clone();
    let data = state.finalize();
    save_cache(transcript_path, current_size, &data, &task_ids);
    data
}

// ── 파싱 엔진 ──────────────────────────────────────────────

fn parse_lines(transcript_path: &str, offset: u64, state: &mut ParseState) {
    let file = match File::open(transcript_path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let mut reader = BufReader::new(file);

    if offset > 0 {
        if reader.seek(SeekFrom::Start(offset)).is_err() {
            return;
        }
    }

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let entry: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Track session name
        if entry.get("type").and_then(|v| v.as_str()) == Some("custom-title") {
            if let Some(title) = entry.get("customTitle").and_then(|v| v.as_str()) {
                state.custom_title = Some(title.to_string());
            }
        } else if let Some(slug) = entry.get("slug").and_then(|v| v.as_str()) {
            state.latest_slug = Some(slug.to_string());
        }

        let timestamp = entry
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(parse_iso_timestamp)
            .unwrap_or_else(SystemTime::now);

        if state.session_start.is_none() && entry.get("timestamp").is_some() {
            state.session_start = Some(timestamp);
        }

        let content = match entry
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_array())
        {
            Some(arr) => arr,
            None => continue,
        };

        for block in content {
            let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if block_type == "tool_use" {
                let id = match block.get("id").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                let name = match block.get("name").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };

                let input = block.get("input");

                if name == "Task" {
                    let agent_entry = AgentEntry {
                        id: id.clone(),
                        agent_type: input
                            .and_then(|i| i.get("subagent_type"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        model: input
                            .and_then(|i| i.get("model"))
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        description: input
                            .and_then(|i| i.get("description"))
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        status: AgentStatus::Running,
                        start_time: timestamp,
                        end_time: None,
                    };
                    state.agent_map.insert(id, agent_entry);
                } else if name == "TodoWrite" {
                    if let Some(todos) =
                        input.and_then(|i| i.get("todos")).and_then(|v| v.as_array())
                    {
                        state.latest_todos.clear();
                        state.task_id_to_index.clear();
                        for todo in todos {
                            let content = todo
                                .get("content")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let status = todo
                                .get("status")
                                .and_then(|v| v.as_str())
                                .and_then(normalize_task_status)
                                .unwrap_or(TodoStatus::Pending);
                            state.latest_todos.push(TodoItem { content, status });
                        }
                    }
                } else if name == "TaskCreate" {
                    let subject = input
                        .and_then(|i| i.get("subject"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let description = input
                        .and_then(|i| i.get("description"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let content = if !subject.is_empty() {
                        subject
                    } else if !description.is_empty() {
                        description
                    } else {
                        "Untitled task"
                    };
                    let status = input
                        .and_then(|i| i.get("status"))
                        .and_then(|v| v.as_str())
                        .and_then(normalize_task_status)
                        .unwrap_or(TodoStatus::Pending);
                    state.latest_todos.push(TodoItem {
                        content: content.to_string(),
                        status,
                    });

                    let task_id = input
                        .and_then(|i| i.get("taskId"))
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            _ => id.clone(),
                        })
                        .unwrap_or_else(|| id.clone());
                    state
                        .task_id_to_index
                        .insert(task_id, state.latest_todos.len() - 1);
                } else if name == "TaskUpdate" {
                    if let Some(idx) = resolve_task_index(
                        input,
                        &state.task_id_to_index,
                        &state.latest_todos,
                    ) {
                        if let Some(status) = input
                            .and_then(|i| i.get("status"))
                            .and_then(|v| v.as_str())
                            .and_then(normalize_task_status)
                        {
                            state.latest_todos[idx].status = status;
                        }
                        let subject = input
                            .and_then(|i| i.get("subject"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let description = input
                            .and_then(|i| i.get("description"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let content = if !subject.is_empty() {
                            subject
                        } else {
                            description
                        };
                        if !content.is_empty() {
                            state.latest_todos[idx].content = content.to_string();
                        }
                    }
                } else {
                    let target = extract_target(&name, input);
                    state.tool_map.insert(
                        id.clone(),
                        ToolEntry {
                            id,
                            name,
                            target,
                            status: ToolStatus::Running,
                            start_time: timestamp,
                            end_time: None,
                        },
                    );
                }
            }

            if block_type == "tool_result" {
                if let Some(tool_use_id) = block.get("tool_use_id").and_then(|v| v.as_str()) {
                    let is_error =
                        block.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
                    if let Some(tool) = state.tool_map.get_mut(tool_use_id) {
                        tool.status = if is_error {
                            ToolStatus::Error
                        } else {
                            ToolStatus::Completed
                        };
                        tool.end_time = Some(timestamp);
                    }
                    if let Some(agent) = state.agent_map.get_mut(tool_use_id) {
                        agent.status = AgentStatus::Completed;
                        agent.end_time = Some(timestamp);
                    }
                }
            }
        }
    }
}

// ── 헬퍼 함수 ──────────────────────────────────────────────

fn extract_target(tool_name: &str, input: Option<&Value>) -> Option<String> {
    let input = input?;
    match tool_name {
        "Read" | "Write" | "Edit" => input
            .get("file_path")
            .or_else(|| input.get("path"))
            .and_then(|v| v.as_str())
            .map(String::from),
        "Glob" | "Grep" => input.get("pattern").and_then(|v| v.as_str()).map(String::from),
        "Bash" => {
            let cmd = input.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let truncated: String = cmd.chars().take(30).collect();
            if truncated.len() < cmd.len() {
                Some(format!("{truncated}..."))
            } else {
                Some(cmd.to_string())
            }
        }
        _ => None,
    }
}

fn resolve_task_index(
    input: Option<&Value>,
    task_id_to_index: &HashMap<String, usize>,
    latest_todos: &[TodoItem],
) -> Option<usize> {
    let task_id_val = input?.get("taskId")?;
    let key = match task_id_val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };

    if let Some(&idx) = task_id_to_index.get(&key) {
        return Some(idx);
    }

    if let Ok(n) = key.parse::<usize>() {
        let idx = n.checked_sub(1)?;
        if idx < latest_todos.len() {
            return Some(idx);
        }
    }

    None
}

fn normalize_task_status(status: &str) -> Option<TodoStatus> {
    match status {
        "pending" | "not_started" => Some(TodoStatus::Pending),
        "in_progress" | "running" => Some(TodoStatus::InProgress),
        "completed" | "complete" | "done" => Some(TodoStatus::Completed),
        _ => None,
    }
}

fn parse_iso_timestamp(s: &str) -> Option<SystemTime> {
    // Simple ISO 8601 parser: 2024-03-15T10:30:00.000Z
    let s = s.trim_end_matches('Z');
    let (date_part, time_part) = s.split_once('T')?;
    let date_parts: Vec<&str> = date_part.split('-').collect();
    if date_parts.len() != 3 {
        return None;
    }

    let year: i64 = date_parts[0].parse().ok()?;
    let month: i64 = date_parts[1].parse().ok()?;
    let day: i64 = date_parts[2].parse().ok()?;

    let time_base = time_part.split('+').next().unwrap_or(time_part);
    let time_base = time_base.split('-').next().unwrap_or(time_base);
    let (hms, frac) = if time_base.contains('.') {
        let (h, f) = time_base.split_once('.')?;
        (h, f)
    } else {
        (time_base, "0")
    };

    let time_parts: Vec<&str> = hms.split(':').collect();
    if time_parts.len() < 2 {
        return None;
    }

    let hour: i64 = time_parts[0].parse().ok()?;
    let min: i64 = time_parts[1].parse().ok()?;
    let sec: i64 = time_parts
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let millis: i64 = frac.get(..3).unwrap_or(frac).parse().ok().unwrap_or(0);

    // Days from epoch (simplified - not handling leap seconds)
    let days = days_from_epoch(year, month, day)?;
    let total_secs = days * 86400 + hour * 3600 + min * 60 + sec;

    if total_secs < 0 {
        return None;
    }

    Some(UNIX_EPOCH + Duration::from_millis(total_secs as u64 * 1000 + millis as u64))
}

fn days_from_epoch(year: i64, month: i64, day: i64) -> Option<i64> {
    // Simplified date calculation
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 9 } else { month - 3 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}
