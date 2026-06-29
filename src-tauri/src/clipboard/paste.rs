use crate::clipboard::monitor::strip_html;
use crate::clipboard::ClipboardMonitor;
use crate::db::Database;
use crate::models::ContentType;
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn paste_clip(
    monitor: &Arc<ClipboardMonitor>,
    db: &Arc<Database>,
    id: i64,
    plain_text_only: bool,
    simulate_paste: bool,
) -> Result<(), String> {
    let clip = db.get_clip(id)?;
    db.touch_clip(id)?;

    monitor.set_internal_copy(true);

    let result = write_clipboard(&clip, plain_text_only, db);

    if result.is_ok() && simulate_paste {
        thread::sleep(Duration::from_millis(100));
        simulate_ctrl_v();
    }

    thread::spawn({
        let monitor = Arc::clone(monitor);
        move || {
            thread::sleep(Duration::from_millis(500));
            monitor.set_internal_copy(false);
        }
    });

    result
}

pub fn write_clipboard_only(
    clip: &crate::models::ClipItem,
    plain_text_only: bool,
    db: &Database,
) -> Result<(), String> {
    write_clipboard(clip, plain_text_only, db)
}

pub fn paste_snippet(
    monitor: &Arc<ClipboardMonitor>,
    content: &str,
    simulate_paste: bool,
) -> Result<(), String> {
    monitor.set_internal_copy(true);

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(content).map_err(|e| e.to_string())?;

    if simulate_paste {
        thread::sleep(Duration::from_millis(100));
        simulate_ctrl_v();
    }

    thread::spawn({
        let monitor = Arc::clone(monitor);
        move || {
            thread::sleep(Duration::from_millis(500));
            monitor.set_internal_copy(false);
        }
    });

    Ok(())
}

fn write_clipboard(
    clip: &crate::models::ClipItem,
    plain_text_only: bool,
    db: &Database,
) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

    match clip.content_type {
        ContentType::Image => {
            if let Some(blob) = db.get_clip_blob(clip.id)? {
                let img = image::load_from_memory(&blob).map_err(|e| e.to_string())?;
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                clipboard
                    .set_image(arboard::ImageData {
                        width: width as usize,
                        height: height as usize,
                        bytes: std::borrow::Cow::Owned(rgba.into_raw()),
                    })
                    .map_err(|e| e.to_string())?;
            }
        }
        ContentType::Html if plain_text_only => {
            let plain = strip_html(&clip.content_text);
            clipboard.set_text(&plain).map_err(|e| e.to_string())?;
        }
        ContentType::Html | ContentType::Text | ContentType::File => {
            clipboard
                .set_text(&clip.content_text)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn simulate_ctrl_v() {
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.key(Key::Control, Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }
}
