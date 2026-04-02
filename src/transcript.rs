use crate::config::get_hud_plugin_dir;
use crate::types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ── 캐시 타입 ──────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct TranscriptFileCache {
    path: String,
    mtime_ms: u64,
    size: u64,
    tools: Vec<CachedTool>,
    agents: Vec<CachedAgent>,
    todos: Vec<CachedTodo>,
    session_start_ms: Option<u64>,
    session_name: Option<String>,
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

fn try_load_cache(transcript_path: &str) -> Option<TranscriptData> {
    let content = std::fs::read_to_string(cache_path()).ok()?;
    let cache: TranscriptFileCache = serde_json::from_str(&content).ok()?;

    if cache.path != transcript_path {
        return None;
    }

    let meta = std::fs::metadata(transcript_path).ok()?;
    let mtime_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    if cache.mtime_ms != mtime_ms || cache.size != meta.len() {
        return None;
    }

    Some(TranscriptData {
        tools: cache
            .tools
            .into_iter()
            .map(|t| ToolEntry {
                id: t.id,
                name: t.name,
                target: t.target,
                status: match t.status {
                    0 => ToolStatus::Running,
                    2 => ToolStatus::Error,
                    _ => ToolStatus::Completed,
                },
                start_time: ms_to_system_time(t.start_ms),
                end_time: t.end_ms.map(ms_to_system_time),
            })
            .collect(),
        agents: cache
            .agents
            .into_iter()
            .map(|a| AgentEntry {
                id: a.id,
                agent_type: a.agent_type,
                model: a.model,
                description: a.description,
                status: if a.status == 0 {
                    AgentStatus::Running
                } else {
                    AgentStatus::Completed
                },
                start_time: ms_to_system_time(a.start_ms),
                end_time: a.end_ms.map(ms_to_system_time),
            })
            .collect(),
        todos: cache
            .todos
            .into_iter()
            .map(|t| TodoItem {
                content: t.content,
                status: match t.status {
                    1 => TodoStatus::InProgress,
                    2 => TodoStatus::Completed,
                    _ => TodoStatus::Pending,
                },
            })
            .collect(),
        session_start: cache.session_start_ms.map(ms_to_system_time),
        session_name: cache.session_name,
    })
}

fn save_cache(transcript_path: &str, data: &TranscriptData) {
    let meta = match std::fs::metadata(transcript_path) {
        Ok(m) => m,
        Err(_) => return,
    };
    let mtime_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let cache = TranscriptFileCache {
        path: transcript_path.to_string(),
        mtime_ms,
        size: meta.len(),
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

    if let Some(cached) = try_load_cache(transcript_path) {
        return cached;
    }

    let result = parse_transcript_fresh(transcript_path);
    save_cache(transcript_path, &result);
    result
}

fn parse_transcript_fresh(transcript_path: &str) -> TranscriptData {
    let mut result = TranscriptData::default();

    let file = match File::open(transcript_path) {
        Ok(f) => f,
        Err(_) => return result,
    };

    let reader = BufReader::new(file);
    let mut tool_map: HashMap<String, ToolEntry> = HashMap::new();
    let mut agent_map: HashMap<String, AgentEntry> = HashMap::new();
    let mut latest_todos: Vec<TodoItem> = Vec::new();
    let mut task_id_to_index: HashMap<String, usize> = HashMap::new();
    let mut latest_slug: Option<String> = None;
    let mut custom_title: Option<String> = None;

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
                custom_title = Some(title.to_string());
            }
        } else if let Some(slug) = entry.get("slug").and_then(|v| v.as_str()) {
            latest_slug = Some(slug.to_string());
        }

        let timestamp = entry
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(parse_iso_timestamp)
            .unwrap_or_else(SystemTime::now);

        if result.session_start.is_none() && entry.get("timestamp").is_some() {
            result.session_start = Some(timestamp);
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
                    agent_map.insert(id, agent_entry);
                } else if name == "TodoWrite" {
                    if let Some(todos) = input.and_then(|i| i.get("todos")).and_then(|v| v.as_array()) {
                        latest_todos.clear();
                        task_id_to_index.clear();
                        for todo in todos {
                            let content = todo.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let status = todo.get("status").and_then(|v| v.as_str()).and_then(normalize_task_status).unwrap_or(TodoStatus::Pending);
                            latest_todos.push(TodoItem { content, status });
                        }
                    }
                } else if name == "TaskCreate" {
                    let subject = input.and_then(|i| i.get("subject")).and_then(|v| v.as_str()).unwrap_or("");
                    let description = input.and_then(|i| i.get("description")).and_then(|v| v.as_str()).unwrap_or("");
                    let content = if !subject.is_empty() { subject } else if !description.is_empty() { description } else { "Untitled task" };
                    let status = input.and_then(|i| i.get("status")).and_then(|v| v.as_str()).and_then(normalize_task_status).unwrap_or(TodoStatus::Pending);
                    latest_todos.push(TodoItem { content: content.to_string(), status });

                    let task_id = input
                        .and_then(|i| i.get("taskId"))
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            _ => id.clone(),
                        })
                        .unwrap_or_else(|| id.clone());
                    task_id_to_index.insert(task_id, latest_todos.len() - 1);
                } else if name == "TaskUpdate" {
                    if let Some(idx) = resolve_task_index(input, &task_id_to_index, &latest_todos) {
                        if let Some(status) = input.and_then(|i| i.get("status")).and_then(|v| v.as_str()).and_then(normalize_task_status) {
                            latest_todos[idx].status = status;
                        }
                        let subject = input.and_then(|i| i.get("subject")).and_then(|v| v.as_str()).unwrap_or("");
                        let description = input.and_then(|i| i.get("description")).and_then(|v| v.as_str()).unwrap_or("");
                        let content = if !subject.is_empty() { subject } else { description };
                        if !content.is_empty() {
                            latest_todos[idx].content = content.to_string();
                        }
                    }
                } else {
                    let target = extract_target(&name, input);
                    tool_map.insert(
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
                    let is_error = block.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
                    if let Some(tool) = tool_map.get_mut(tool_use_id) {
                        tool.status = if is_error { ToolStatus::Error } else { ToolStatus::Completed };
                        tool.end_time = Some(timestamp);
                    }
                    if let Some(agent) = agent_map.get_mut(tool_use_id) {
                        agent.status = AgentStatus::Completed;
                        agent.end_time = Some(timestamp);
                    }
                }
            }
        }
    }

    let mut tools: Vec<ToolEntry> = tool_map.into_values().collect();
    tools.sort_by_key(|t| t.start_time);
    let len = tools.len();
    if len > 20 {
        tools = tools.split_off(len - 20);
    }

    let mut agents: Vec<AgentEntry> = agent_map.into_values().collect();
    agents.sort_by_key(|a| a.start_time);
    let len = agents.len();
    if len > 10 {
        agents = agents.split_off(len - 10);
    }

    result.tools = tools;
    result.agents = agents;
    result.todos = latest_todos;
    result.session_name = custom_title.or(latest_slug);

    result
}

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
            if cmd.len() > 30 {
                Some(format!("{}...", &cmd[..30]))
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
    let sec: i64 = time_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
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
