use crate::backup::{export_backup, format_bytes, import_backup, migrate_storage};
use crate::clipboard::paste::{paste_clip, paste_snippet, write_clipboard_only};
use crate::config::save_data_dir;
use crate::db::Database;
use crate::models::{AppSettings, ClipCategory, ClipItem, HistoryStats, Snippet};
use crate::state::AppState;
use crate::{apply_hotkey, hotkey::parse_hotkey};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub fn list_clips(
    state: State<'_, AppState>,
    query: Option<String>,
    category: Option<String>,
) -> Result<Vec<ClipItem>, String> {
    let cat = category
        .map(|c| ClipCategory::from_str(&c))
        .unwrap_or_default();
    let smart = state.db.get_settings().enable_smart_search;
    state.db.list_clips(query.as_deref(), cat, smart)
}

#[tauri::command]
pub fn get_clip_image(state: State<'_, AppState>, id: i64) -> Result<Option<String>, String> {
    state.db.get_clip_image_data_url(id)
}

#[tauri::command]
pub fn get_recent_notifications(state: State<'_, AppState>) -> Result<Vec<ClipItem>, String> {
    Ok(state.notifications.recent())
}

#[tauri::command]
pub fn get_clip_thumbnail(state: State<'_, AppState>, id: i64) -> Result<Option<String>, String> {
    state.db.get_clip_thumbnail_data_url(id)
}

#[tauri::command]
pub fn save_clip_image(
    state: State<'_, AppState>,
    id: i64,
    dest_dir: String,
) -> Result<String, String> {
    state.db.save_clip_image(id, &dest_dir)
}

#[tauri::command]
pub fn get_history_stats(state: State<'_, AppState>) -> Result<HistoryStats, String> {
    state.db.get_history_stats()
}

#[tauri::command]
pub fn toggle_pin(state: State<'_, AppState>, id: i64) -> Result<bool, String> {
    state.db.toggle_pin(id)
}

#[tauri::command]
pub fn delete_clip(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_clip(id)
}

#[tauri::command]
pub fn delete_clips(state: State<'_, AppState>, ids: Vec<i64>) -> Result<(), String> {
    state.db.delete_clips(&ids)
}

#[tauri::command]
pub fn restore_clip(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.restore_clip(id)
}

#[tauri::command]
pub fn list_trash_clips(state: State<'_, AppState>) -> Result<Vec<ClipItem>, String> {
    state.db.list_trash_clips()
}

#[tauri::command]
pub fn empty_trash(state: State<'_, AppState>) -> Result<u32, String> {
    state.db.empty_trash()
}

#[tauri::command]
pub fn merge_duplicate_clips(state: State<'_, AppState>) -> Result<u32, String> {
    state.db.merge_duplicate_clips()
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>, keep_pinned: bool) -> Result<(), String> {
    state.db.clear_history(keep_pinned)
}

#[tauri::command]
pub async fn paste_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    plain_text_only: bool,
) -> Result<(), String> {
    let simulate = state.db.get_settings().simulate_paste;
    if simulate {
        hide_panel(app.clone()).await?;
        std::thread::sleep(std::time::Duration::from_millis(120));
    }

    paste_clip(&state.monitor, &state.db, id, plain_text_only, simulate)?;

    if !simulate {
        hide_panel(app).await?;
    }
    Ok(())
}

#[tauri::command]
pub fn copy_item_to_clipboard(
    state: State<'_, AppState>,
    id: i64,
    plain_text_only: bool,
) -> Result<(), String> {
    let clip = state.db.get_clip(id)?;
    state.monitor.set_internal_copy(true);
    write_clipboard_only(&clip, plain_text_only, &state.db)?;
    std::thread::spawn({
        let monitor = Arc::clone(&state.monitor);
        move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            monitor.set_internal_copy(false);
        }
    });
    Ok(())
}

#[tauri::command]
pub fn list_snippets(state: State<'_, AppState>) -> Result<Vec<Snippet>, String> {
    state.db.list_snippets()
}

#[tauri::command]
pub fn create_snippet(
    state: State<'_, AppState>,
    title: String,
    content: String,
) -> Result<Snippet, String> {
    state.db.create_snippet(&title, &content)
}

#[tauri::command]
pub fn update_snippet(
    state: State<'_, AppState>,
    id: i64,
    title: String,
    content: String,
) -> Result<(), String> {
    state.db.update_snippet(id, &title, &content)
}

#[tauri::command]
pub fn delete_snippet(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_snippet(id)
}

#[tauri::command]
pub fn reorder_snippets(state: State<'_, AppState>, ids: Vec<i64>) -> Result<(), String> {
    state.db.reorder_snippets(&ids)
}

#[tauri::command]
pub async fn paste_text(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
) -> Result<(), String> {
    if text.len() > 512_000 {
        return Err("内容过长".to_string());
    }
    let simulate = state.db.get_settings().simulate_paste;
    if simulate {
        hide_panel(app.clone()).await?;
        std::thread::sleep(std::time::Duration::from_millis(120));
    }

    paste_snippet(&state.monitor, &text, simulate)?;

    if !simulate {
        hide_panel(app).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let url = url.trim();
    if url.is_empty() {
        return Err("链接为空".to_string());
    }
    let open_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{url}")
    };
    app.opener()
        .open_url(&open_url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_icon(state: State<'_, AppState>, app_name: String) -> Result<Option<String>, String> {
    Ok(crate::app_icon::get_icon_data_url(state.db.data_dir(), &app_name))
}

#[tauri::command]
pub async fn open_path(app: AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let path = path.trim();
    if path.is_empty() {
        return Err("路径为空".to_string());
    }
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn paste_snippet_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    content: String,
) -> Result<(), String> {
    if content.len() > 512_000 {
        return Err("片段内容过长".to_string());
    }
    let simulate = state.db.get_settings().simulate_paste;
    if simulate {
        hide_panel(app.clone()).await?;
        std::thread::sleep(std::time::Duration::from_millis(120));
    }

    paste_snippet(&state.monitor, &content, simulate)?;

    if !simulate {
        hide_panel(app).await?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    Ok(state.db.get_settings())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    parse_hotkey(&settings.hotkey)?;
    state.db.save_settings(&settings)?;
    state
        .monitor
        .update_config(settings.dedupe, settings.poll_interval_ms);

    apply_hotkey(&app, &settings.hotkey)?;

    if settings.start_on_boot {
        use tauri_plugin_autostart::ManagerExt;
        let _ = app.autolaunch().enable();
    } else {
        use tauri_plugin_autostart::ManagerExt;
        let _ = app.autolaunch().disable();
    }

    let _ = app.emit("settings-changed", settings.theme.clone());
    Ok(())
}

#[tauri::command]
pub fn migrate_storage_path(
    state: State<'_, AppState>,
    new_path: String,
) -> Result<String, String> {
    let new_dir = PathBuf::from(&new_path);
    let old_dir = state.db.data_dir().clone();
    if new_dir == old_dir {
        return Ok("存储路径未变更".to_string());
    }
    migrate_storage(&old_dir, &new_dir)?;
    save_data_dir(&new_dir)?;
    state.db.set_setting("storage_path", &new_path)?;
    Ok(format!(
        "已迁移至 {}，请重启应用生效",
        new_path
    ))
}

#[tauri::command]
pub fn export_backup_cmd(state: State<'_, AppState>, dest: String) -> Result<(), String> {
    export_backup(&state.db, PathBuf::from(dest).as_path())
}

#[tauri::command]
pub fn import_backup_cmd(dest_path: String, src: String) -> Result<String, String> {
    let dest = PathBuf::from(&dest_path);
    import_backup(&dest, PathBuf::from(src).as_path())?;
    save_data_dir(&dest)?;
    Ok("导入完成，请重启应用生效".to_string())
}

#[tauri::command]
pub fn run_cleanup(state: State<'_, AppState>) -> Result<u32, String> {
    state.db.cleanup_expired()
}

#[tauri::command]
pub fn format_disk_size(bytes: u64) -> String {
    format_bytes(bytes)
}

#[tauri::command]
pub fn set_monitor_paused(state: State<'_, AppState>, paused: bool) -> Result<(), String> {
    state
        .monitor
        .paused
        .store(paused, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn is_monitor_paused(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.monitor.paused.load(std::sync::atomic::Ordering::SeqCst))
}

#[tauri::command]
pub async fn show_panel(app: AppHandle) -> Result<(), String> {
    position_panel(&app)?;
    if let Some(state) = app.try_state::<AppState>() {
        state.monitor.set_panel_open(true);
    }
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    let _ = app.emit("panel-opened", ());
    Ok(())
}

#[tauri::command]
pub async fn hide_panel(app: AppHandle) -> Result<(), String> {
    if let Some(state) = app.try_state::<AppState>() {
        state.monitor.set_panel_open(false);
        save_panel_position(&app, &state.db);
    }
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    let _ = app.emit("panel-closed", ());
    Ok(())
}

#[tauri::command]
pub async fn toggle_panel(app: AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().map_err(|e| e.to_string())?;
        if visible {
            hide_panel(app).await?;
            Ok(false)
        } else {
            show_panel(app).await?;
            Ok(true)
        }
    } else {
        Err("main window not found".to_string())
    }
}

pub fn spawn_cleanup_loop(db: Arc<Database>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(6 * 3600));
        let _ = db.cleanup_expired();
    });
}

fn save_panel_position(app: &AppHandle, db: &Database) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Ok(pos) = window.outer_position() {
        let _ = db.set_setting("panel_pos_x", &pos.x.to_string());
        let _ = db.set_setting("panel_pos_y", &pos.y.to_string());
    }
}

fn position_panel(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    if let Some(state) = app.try_state::<AppState>() {
        let settings = state.db.get_settings();
        if settings.panel_follow_cursor {
            if let Some((cx, cy)) = cursor_physical_pos() {
                return position_near_cursor(&window, cx, cy);
            }
        } else if let (Some(xs), Some(ys)) = (
            state.db.get_setting("panel_pos_x"),
            state.db.get_setting("panel_pos_y"),
        ) {
            if let (Ok(x), Ok(y)) = (xs.parse::<f64>(), ys.parse::<f64>()) {
                if x >= 0.0 && y >= 0.0 {
                    window
                        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                            x: x as i32,
                            y: y as i32,
                        }))
                        .map_err(|e| e.to_string())?;
                    return Ok(());
                }
            }
        }
    }

    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen = monitor.size();
        let scale = monitor.scale_factor();
        let window_size = window.outer_size().map_err(|e| e.to_string())?;
        let x = ((screen.width as f64 / scale) - (window_size.width as f64 / scale)) / 2.0;
        let y = (screen.height as f64 / scale) - (window_size.height as f64 / scale) - 48.0;
        window
            .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))
            .map_err(|e| e.to_string())?;
    } else {
        window.center().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(windows)]
fn cursor_physical_pos() -> Option<(i32, i32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    unsafe {
        let mut pt = POINT::default();
        GetCursorPos(&mut pt).ok()?;
        Some((pt.x, pt.y))
    }
}

#[cfg(not(windows))]
fn cursor_physical_pos() -> Option<(i32, i32)> {
    None
}

fn position_near_cursor(
    window: &tauri::WebviewWindow,
    cx: i32,
    cy: i32,
) -> Result<(), String> {
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let mut x = cx + 16;
    let mut y = cy + 16;
    if let Ok(Some(monitor)) = window.current_monitor() {
        let pos = monitor.position();
        let screen = monitor.size();
        let max_x = pos.x + screen.width as i32 - size.width as i32;
        let max_y = pos.y + screen.height as i32 - size.height as i32;
        x = x.clamp(pos.x, max_x.max(pos.x));
        y = y.clamp(pos.y, max_y.max(pos.y));
    }
    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|e| e.to_string())
}
