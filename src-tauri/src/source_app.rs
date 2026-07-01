#[cfg(windows)]
pub fn foreground_process_exe() -> Option<std::path::PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::prelude::OsStringExt;
    use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = vec![0u16; MAX_PATH as usize];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        if ok.is_err() || len == 0 {
            return None;
        }
        let path = OsString::from_wide(&buf[..len as usize]);
        Some(std::path::PathBuf::from(path))
    }
}

#[cfg(not(windows))]
pub fn foreground_process_exe() -> Option<std::path::PathBuf> {
    None
}

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

/// Prefer executable file name over window title for stable icons and filters.
pub fn resolve_source_app() -> Option<String> {
    if let Some(exe) = foreground_process_exe() {
        return Some(friendly_process_name(&exe));
    }
    foreground_app_name()
}

pub fn friendly_process_name(exe: &std::path::Path) -> String {
    let stem = exe
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");
    let lower = stem.to_lowercase();
    match lower.as_str() {
        "chrome" => "Chrome".into(),
        "msedge" => "Edge".into(),
        "firefox" => "Firefox".into(),
        "wechat" | "weixin" => "微信".into(),
        "cursor" => "Cursor 编辑器".into(),
        "code" => "VS Code".into(),
        "doubao" | "doubaoapp" => "豆包".into(),
        "notepad" => "Notepad".into(),
        "windowsterminal" => "Terminal".into(),
        "explorer" => "Explorer".into(),
        _ => {
            if stem.len() <= 1 {
                stem.to_string()
            } else {
                let mut chars = stem.chars();
                let first = chars.next().unwrap().to_uppercase().to_string();
                first + &chars.as_str().to_lowercase()
            }
        }
    }
}
