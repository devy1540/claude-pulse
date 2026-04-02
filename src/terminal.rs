pub fn get_terminal_width() -> Option<u32> {
    // Try stdout columns
    if let Some(w) = get_terminal_size_fd(1) {
        return Some(w);
    }
    // Try stderr columns (statusline subprocess has piped stdout)
    if let Some(w) = get_terminal_size_fd(2) {
        return Some(w);
    }
    // Try COLUMNS env var
    if let Ok(cols) = std::env::var("COLUMNS") {
        if let Ok(w) = cols.parse::<u32>() {
            if w > 0 {
                return Some(w);
            }
        }
    }
    None
}

#[cfg(unix)]
fn get_terminal_size_fd(fd: i32) -> Option<u32> {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            Some(ws.ws_col as u32)
        } else {
            None
        }
    }
}

#[cfg(unix)]
mod libc {
    #[repr(C)]
    pub struct winsize {
        pub ws_row: u16,
        pub ws_col: u16,
        pub ws_xpixel: u16,
        pub ws_ypixel: u16,
    }

    extern "C" {
        pub fn ioctl(fd: i32, request: u64, ...) -> i32;
    }

    #[cfg(target_os = "macos")]
    pub const TIOCGWINSZ: u64 = 0x40087468;

    #[cfg(target_os = "linux")]
    pub const TIOCGWINSZ: u64 = 0x5413;
}

#[cfg(not(unix))]
fn get_terminal_size_fd(_fd: i32) -> Option<u32> {
    None
}

