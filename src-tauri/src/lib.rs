mod app_filter;
mod app_icon;
mod backup;
mod clipboard;
mod commands;
mod config;
mod db;
mod hotkey;
mod image_store;
mod models;
mod notifications;
mod ocr;
mod search;
mod source_app;
mod state;
mod storage;

use clipboard::ClipboardMonitor;
use commands::spawn_cleanup_loop;
use state::AppState;
use config::resolve_data_dir;
use db::Database;
use hotkey::parse_hotkey;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn apply_hotkey(app: &tauri::AppHandle, hotkey: &str) -> Result<(), String> {
    let shortcut = parse_hotkey(hotkey)?;
    let gs = app.global_shortcut();

    if let Some(state) = app.try_state::<AppState>() {
        if let Some(old) = state.registered_hotkey.lock().take() {
            let _ = gs.unregister(old);
        }
    }

    gs.register(shortcut).map_err(|e| e.to_string())?;

    if let Some(state) = app.try_state::<AppState>() {
        *state.registered_hotkey.lock() = Some(shortcut);
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = commands::toggle_panel(handle).await;
                        });
                    }
                })
                .build(),
        )
        .setup(|app| {
            let data_dir = resolve_data_dir();
            let db = Arc::new(Database::new(data_dir)?);
            let _ = db.cleanup_expired();
            spawn_cleanup_loop(Arc::clone(&db));

            let monitor = Arc::new(ClipboardMonitor::new());
            let notifications = Arc::new(notifications::NotificationState::new());
            monitor.clone().start(
                app.handle().clone(),
                Arc::clone(&db),
                Arc::clone(&notifications),
            );

            app.manage(AppState {
                db: Arc::clone(&db),
                monitor: Arc::clone(&monitor),
                registered_hotkey: parking_lot::Mutex::new(None),
                notifications,
            });

            setup_tray(app)?;

            let settings = db.get_settings();
            apply_hotkey(app.handle(), &settings.hotkey)?;

            if settings.start_on_boot {
                use tauri_plugin_autostart::ManagerExt;
                let _ = app.autolaunch().enable();
            }

            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        let app = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            std::thread::sleep(std::time::Duration::from_millis(220));
                            if let Some(state) = app.try_state::<AppState>() {
                                if !state.db.get_settings().dismiss_on_blur {
                                    return;
                                }
                                if let Some(window) = app.get_webview_window("main") {
                                    if window.is_focused().unwrap_or(false) {
                                        return;
                                    }
                                    if !window.is_visible().unwrap_or(false) {
                                        return;
                                    }
                                }
                                state.monitor.set_panel_open(false);
                                let _ = commands::hide_panel(app.clone()).await;
                            }
                        });
                    }
                });
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_clips,
            commands::get_clip_thumbnail,
            commands::get_clip_image,
            commands::get_recent_notifications,
            commands::save_clip_image,
            commands::get_history_stats,
            commands::toggle_pin,
            commands::delete_clip,
            commands::delete_clips,
            commands::restore_clip,
            commands::list_trash_clips,
            commands::empty_trash,
            commands::merge_duplicate_clips,
            commands::get_storage_details,
            commands::reclaim_storage,
            commands::open_data_folder,
            commands::ocr_clip,
            commands::ocr_backfill,
            commands::clear_history,
            commands::paste_item,
            commands::copy_item_to_clipboard,
            commands::list_snippets,
            commands::create_snippet,
            commands::update_snippet,
            commands::delete_snippet,
            commands::reorder_snippets,
            commands::paste_snippet_cmd,
            commands::paste_text,
            commands::open_path,
            commands::open_url,
            commands::get_app_icon,
            commands::get_settings,
            commands::save_settings,
            commands::migrate_storage_path,
            commands::export_backup_cmd,
            commands::import_backup_cmd,
            commands::run_cleanup,
            commands::format_disk_size,
            commands::set_monitor_paused,
            commands::is_monitor_paused,
            commands::show_panel,
            commands::hide_panel,
            commands::toggle_panel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "打开面板", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "暂停记录", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&show, &settings, &pause, &PredefinedMenuItem::separator(app)?, &quit],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = commands::show_panel(app).await;
                });
            }
            "settings" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = commands::show_panel(app.clone()).await;
                    let _ = app.emit("open-settings", ());
                });
            }
            "pause" => {
                if let Some(state) = app.try_state::<AppState>() {
                    let paused = !state.monitor.paused.load(std::sync::atomic::Ordering::SeqCst);
                    state
                        .monitor
                        .paused
                        .store(paused, std::sync::atomic::Ordering::SeqCst);
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    let _ = commands::toggle_panel(app).await;
                });
            }
        })
        .build(app)?;

    Ok(())
}
