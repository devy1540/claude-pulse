use crate::render::colors::*;
use crate::types::{RenderContext, TodoStatus};

pub fn render_todos_line(ctx: &RenderContext) -> Option<String> {
    let todos = &ctx.transcript.todos;
    let colors = &ctx.config.colors;

    if todos.is_empty() {
        return None;
    }

    let in_progress = todos.iter().find(|t| t.status == TodoStatus::InProgress);
    let completed = todos.iter().filter(|t| t.status == TodoStatus::Completed).count();
    let total = todos.len();

    match in_progress {
        None => {
            if completed == total && total > 0 {
                Some(format!(
                    "{} All todos complete {}",
                    green("✓"),
                    label_color(&format!("({completed}/{total})"), colors)
                ))
            } else {
                None
            }
        }
        Some(todo) => {
            let content = truncate_content(&todo.content, 50);
            let progress = label_color(&format!("({completed}/{total})"), colors);
            Some(format!("{} {content} {progress}", yellow("▸")))
        }
    }
}

fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...", &content[..max_len.saturating_sub(3)])
    }
}
