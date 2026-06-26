use crate::models::ClipItem;
use parking_lot::Mutex;
use std::collections::VecDeque;

pub struct NotificationState {
    recent: Mutex<VecDeque<ClipItem>>,
}

impl NotificationState {
    pub fn new() -> Self {
        Self {
            recent: Mutex::new(VecDeque::with_capacity(3)),
        }
    }

    pub fn push(&self, item: ClipItem) {
        let mut q = self.recent.lock();
        if q.len() >= 3 {
            q.pop_back();
        }
        q.push_front(item);
    }

    pub fn recent(&self) -> Vec<ClipItem> {
        self.recent.lock().iter().cloned().collect()
    }
}

pub fn show_clip_notification(app: &tauri::AppHandle, item: &ClipItem) {
    use tauri_plugin_notification::NotificationExt;
    let body = clip_preview_label(item);
    let _ = app
        .notification()
        .builder()
        .title("ClipBox")
        .body(body)
        .show();
}

pub fn clip_preview_label(item: &ClipItem) -> String {
    let text = item.content_text.trim();
    if item.content_type == crate::models::ContentType::Image {
        return "[图片]".to_string();
    }
    if text.len() > 80 {
        format!("{}…", &text[..80])
    } else {
        text.to_string()
    }
}
