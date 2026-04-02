use crate::types::MemoryInfo;

#[cfg(target_os = "macos")]
pub fn get_memory_usage() -> Option<MemoryInfo> {
    use std::process::Command;

    // Get total memory via sysctl
    let total_output = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    let total_bytes: u64 = String::from_utf8_lossy(&total_output.stdout)
        .trim()
        .parse()
        .ok()?;

    // Get page size and free/inactive pages via vm_stat
    let vm_output = Command::new("vm_stat").output().ok()?;
    let vm_text = String::from_utf8_lossy(&vm_output.stdout);

    let page_size: u64 = 16384; // Default for Apple Silicon, 4096 for Intel
    let mut free_pages: u64 = 0;
    let mut inactive_pages: u64 = 0;

    for line in vm_text.lines() {
        if line.starts_with("Pages free:") {
            free_pages = parse_vm_stat_value(line);
        } else if line.starts_with("Pages inactive:") {
            inactive_pages = parse_vm_stat_value(line);
        }
    }

    // Check actual page size from the first line
    let actual_page_size = vm_text
        .lines()
        .next()
        .and_then(|l| {
            l.split("page size of ")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<u64>().ok())
        })
        .unwrap_or(page_size);

    let free_bytes = (free_pages + inactive_pages) * actual_page_size;
    let free_bytes = free_bytes.min(total_bytes);
    let used_bytes = total_bytes - free_bytes;
    let used_percent = ((used_bytes as f64 / total_bytes as f64) * 100.0).round() as u32;

    Some(MemoryInfo {
        total_bytes,
        used_bytes,
        free_bytes,
        used_percent: used_percent.min(100),
    })
}

#[cfg(target_os = "macos")]
fn parse_vm_stat_value(line: &str) -> u64 {
    line.split(':')
        .nth(1)
        .and_then(|v| v.trim().trim_end_matches('.').parse().ok())
        .unwrap_or(0)
}

#[cfg(target_os = "linux")]
pub fn get_memory_usage() -> Option<MemoryInfo> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb: u64 = 0;
    let mut available_kb: u64 = 0;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = parse_meminfo_value(line);
        } else if line.starts_with("MemAvailable:") {
            available_kb = parse_meminfo_value(line);
        }
    }

    if total_kb == 0 {
        return None;
    }

    let total_bytes = total_kb * 1024;
    let free_bytes = (available_kb * 1024).min(total_bytes);
    let used_bytes = total_bytes - free_bytes;
    let used_percent = ((used_bytes as f64 / total_bytes as f64) * 100.0).round() as u32;

    Some(MemoryInfo {
        total_bytes,
        used_bytes,
        free_bytes,
        used_percent: used_percent.min(100),
    })
}

#[cfg(target_os = "linux")]
fn parse_meminfo_value(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn get_memory_usage() -> Option<MemoryInfo> {
    None
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }

    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_idx = 0;

    while value >= 1024.0 && unit_idx < units.len() - 1 {
        value /= 1024.0;
        unit_idx += 1;
    }

    if value >= 10.0 || unit_idx == 0 {
        format!("{:.0} {}", value, units[unit_idx])
    } else {
        format!("{:.1} {}", value, units[unit_idx])
    }
}
