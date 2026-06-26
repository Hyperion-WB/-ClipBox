#[cfg(windows)]
pub fn foreground_app_name() -> Option<String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buf);
        if len == 0 {
            return None;
        }
        let title = String::from_utf16_lossy(&buf[..len as usize]);
        let name = title.split(" - ").last().unwrap_or(&title).trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
}

#[cfg(not(windows))]
pub fn foreground_app_name() -> Option<String> {
    None
}

pub fn detect_file_paths(text: &str) -> Option<String> {
    let line = text.trim();
    if line.is_empty() || line.contains('\n') {
        return None;
    }
    let path = std::path::Path::new(line);
    if path.exists() {
        return Some(line.to_string());
    }
    None
}
