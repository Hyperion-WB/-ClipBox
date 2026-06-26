use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

pub fn parse_hotkey(hotkey: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = hotkey.split('+').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Err("快捷键不能为空".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_part: Option<&str> = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "meta" | "win" | "super" | "cmd" => modifiers |= Modifiers::SUPER,
            _ => {
                if key_part.is_some() {
                    return Err(format!("无法解析快捷键: {hotkey}"));
                }
                key_part = Some(part);
            }
        }
    }

    let key = key_part.ok_or_else(|| "缺少按键".to_string())?;
    let code = parse_key_code(key)?;

    Ok(Shortcut::new(Some(modifiers), code))
}

fn parse_key_code(key: &str) -> Result<Code, String> {
    let normalized = key.trim();
    if normalized.len() == 1 {
        let ch = normalized.chars().next().unwrap().to_ascii_uppercase();
        if ch.is_ascii_alphabetic() {
            return key_letter(ch);
        }
        if ch.is_ascii_digit() {
            return key_digit(ch);
        }
    }

    match normalized.to_lowercase().as_str() {
        "space" => Ok(Code::Space),
        "enter" | "return" => Ok(Code::Enter),
        "tab" => Ok(Code::Tab),
        "escape" | "esc" => Ok(Code::Escape),
        "backspace" => Ok(Code::Backspace),
        "delete" | "del" => Ok(Code::Delete),
        "home" => Ok(Code::Home),
        "end" => Ok(Code::End),
        "pageup" => Ok(Code::PageUp),
        "pagedown" => Ok(Code::PageDown),
        "up" | "arrowup" => Ok(Code::ArrowUp),
        "down" | "arrowdown" => Ok(Code::ArrowDown),
        "left" | "arrowleft" => Ok(Code::ArrowLeft),
        "right" | "arrowright" => Ok(Code::ArrowRight),
        "f1" => Ok(Code::F1),
        "f2" => Ok(Code::F2),
        "f3" => Ok(Code::F3),
        "f4" => Ok(Code::F4),
        "f5" => Ok(Code::F5),
        "f6" => Ok(Code::F6),
        "f7" => Ok(Code::F7),
        "f8" => Ok(Code::F8),
        "f9" => Ok(Code::F9),
        "f10" => Ok(Code::F10),
        "f11" => Ok(Code::F11),
        "f12" => Ok(Code::F12),
        other if other.len() == 1 => {
            let ch = other.chars().next().unwrap().to_ascii_uppercase();
            key_letter(ch)
        }
        other => Err(format!("不支持的按键: {other}")),
    }
}

fn key_letter(ch: char) -> Result<Code, String> {
    match ch {
        'A' => Ok(Code::KeyA),
        'B' => Ok(Code::KeyB),
        'C' => Ok(Code::KeyC),
        'D' => Ok(Code::KeyD),
        'E' => Ok(Code::KeyE),
        'F' => Ok(Code::KeyF),
        'G' => Ok(Code::KeyG),
        'H' => Ok(Code::KeyH),
        'I' => Ok(Code::KeyI),
        'J' => Ok(Code::KeyJ),
        'K' => Ok(Code::KeyK),
        'L' => Ok(Code::KeyL),
        'M' => Ok(Code::KeyM),
        'N' => Ok(Code::KeyN),
        'O' => Ok(Code::KeyO),
        'P' => Ok(Code::KeyP),
        'Q' => Ok(Code::KeyQ),
        'R' => Ok(Code::KeyR),
        'S' => Ok(Code::KeyS),
        'T' => Ok(Code::KeyT),
        'U' => Ok(Code::KeyU),
        'V' => Ok(Code::KeyV),
        'W' => Ok(Code::KeyW),
        'X' => Ok(Code::KeyX),
        'Y' => Ok(Code::KeyY),
        'Z' => Ok(Code::KeyZ),
        _ => Err(format!("不支持的字母键: {ch}")),
    }
}

fn key_digit(ch: char) -> Result<Code, String> {
    match ch {
        '0' => Ok(Code::Digit0),
        '1' => Ok(Code::Digit1),
        '2' => Ok(Code::Digit2),
        '3' => Ok(Code::Digit3),
        '4' => Ok(Code::Digit4),
        '5' => Ok(Code::Digit5),
        '6' => Ok(Code::Digit6),
        '7' => Ok(Code::Digit7),
        '8' => Ok(Code::Digit8),
        '9' => Ok(Code::Digit9),
        _ => Err(format!("不支持的数字键: {ch}")),
    }
}
