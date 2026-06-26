use crate::clipboard::ClipboardMonitor;
use crate::db::Database;
use crate::notifications::NotificationState;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri_plugin_global_shortcut::Shortcut;

pub struct AppState {
    pub db: Arc<Database>,
    pub monitor: Arc<ClipboardMonitor>,
    pub registered_hotkey: Mutex<Option<Shortcut>>,
    pub notifications: Arc<NotificationState>,
}
