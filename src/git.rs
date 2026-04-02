use crate::types::{FileStats, GitStatus};
use std::process::Command;

pub fn get_git_status(cwd: Option<&str>) -> Option<GitStatus> {
    let cwd = cwd?;

    let branch = {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(cwd)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let b = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if b.is_empty() {
            return None;
        }
        b
    };

    let (is_dirty, file_stats) = {
        let output = Command::new("git")
            .args(["--no-optional-locks", "status", "--porcelain"])
            .current_dir(cwd)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let dirty = !text.is_empty();
                let stats = if dirty {
                    Some(parse_file_stats(&text))
                } else {
                    None
                };
                (dirty, stats)
            }
            _ => (false, None),
        }
    };

    let (ahead, behind) = {
        let output = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "@{upstream}...HEAD"])
            .current_dir(cwd)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() == 2 {
                    let b = parts[0].parse().unwrap_or(0);
                    let a = parts[1].parse().unwrap_or(0);
                    (a, b)
                } else {
                    (0, 0)
                }
            }
            _ => (0, 0),
        }
    };

    Some(GitStatus {
        branch,
        is_dirty,
        ahead,
        behind,
        file_stats,
    })
}

fn parse_file_stats(porcelain: &str) -> FileStats {
    let mut stats = FileStats {
        modified: 0,
        added: 0,
        deleted: 0,
        untracked: 0,
    };

    for line in porcelain.lines() {
        if line.len() < 2 {
            continue;
        }
        let bytes = line.as_bytes();
        let index = bytes[0] as char;
        let worktree = bytes[1] as char;

        if line.starts_with("??") {
            stats.untracked += 1;
        } else if index == 'A' {
            stats.added += 1;
        } else if index == 'D' || worktree == 'D' {
            stats.deleted += 1;
        } else if index == 'M' || worktree == 'M' || index == 'R' || index == 'C' {
            stats.modified += 1;
        }
    }

    stats
}
