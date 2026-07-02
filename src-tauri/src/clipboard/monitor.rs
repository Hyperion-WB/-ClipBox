use crate::app_filter::should_record;
use crate::db::Database;
use crate::models::ContentType;
use crate::source_app::{detect_file_paths, resolve_source_app};
use arboard::Clipboard;
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct MonitorConfig {
    pub dedupe: bool,
    pub poll_interval_ms: u64,
}

impl Clone for MonitorConfig {
    fn clone(&self) -> Self {
        Self {
            dedupe: self.dedupe,
            poll_interval_ms: self.poll_interval_ms,
        }
    }
}

pub struct ClipboardMonitor {
    pub paused: AtomicBool,
    pub panel_open: AtomicBool,
    internal_copy: AtomicBool,
    last_hash: Mutex<Option<String>>,
    config: Mutex<MonitorConfig>,
    #[cfg(windows)]
    last_sequence: AtomicU32,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        Self {
            paused: AtomicBool::new(false),
            panel_open: AtomicBool::new(false),
            internal_copy: AtomicBool::new(false),
            last_hash: Mutex::new(None),
            config: Mutex::new(MonitorConfig {
                dedupe: true,
                poll_interval_ms: 600,
            }),
            #[cfg(windows)]
            last_sequence: AtomicU32::new(0),
        }
    }

    pub fn set_panel_open(&self, open: bool) {
        self.panel_open.store(open, Ordering::SeqCst);
    }

    pub fn update_config(&self, dedupe: bool, poll_interval_ms: i32) {
        *self.config.lock() = MonitorConfig {
            dedupe,
            poll_interval_ms: poll_interval_ms.clamp(200, 3000) as u64,
        };
    }

    pub fn set_internal_copy(&self, value: bool) {
        self.internal_copy.store(value, Ordering::SeqCst);
    }

    pub fn start(self: Arc<Self>, app: AppHandle, db: Arc<Database>, notifications: Arc<crate::notifications::NotificationState>) {
        let settings = db.get_settings();
        self.update_config(settings.dedupe, settings.poll_interval_ms);

        thread::spawn(move || {
            let mut clipboard = Clipboard::new().expect("failed to open clipboard");
            let images_dir = db.data_dir().join("images");
            let thumbs_dir = db.data_dir().join("thumbs");
            let _ = std::fs::create_dir_all(&images_dir);
            let _ = std::fs::create_dir_all(&thumbs_dir);

            let mut idle_streak = 0u32;

            loop {
                let config = self.config.lock().clone();
                let base_interval = if self.panel_open.load(Ordering::SeqCst) {
                    config.poll_interval_ms.min(400)
                } else {
                    config.poll_interval_ms
                };
                let sleep_ms = if idle_streak > 8 {
                    base_interval.saturating_mul(3)
                } else if idle_streak > 3 {
                    base_interval.saturating_mul(2)
                } else {
                    base_interval
                };
                thread::sleep(Duration::from_millis(sleep_ms));

                if self.paused.load(Ordering::SeqCst) || self.internal_copy.load(Ordering::SeqCst) {
                    continue;
                }

                #[cfg(windows)]
                if !clipboard_changed(self.last_sequence.load(Ordering::SeqCst)) {
                    idle_streak = idle_streak.saturating_add(1);
                    continue;
                }

                match poll_clipboard(
                    &mut clipboard,
                    Arc::clone(&db),
                    &images_dir,
                    &thumbs_dir,
                    config.dedupe,
                    &self.last_hash,
                    #[cfg(windows)]
                    &self.last_sequence,
                ) {
                    Ok(Some(id)) => {
                        idle_streak = 0;
                        if let Ok(clip) = db.get_clip(id) {
                            let settings = db.get_settings();
                            if settings.enable_notifications
                                && !self.panel_open.load(Ordering::SeqCst)
                            {
                                crate::notifications::show_clip_notification(&app, &clip);
                                notifications.push(clip.clone());
                            }
                        }
                        if self.panel_open.load(Ordering::SeqCst) {
                            let _ = app.emit("clip-added", id);
                        }
                    }
                    Ok(None) => {
                        idle_streak = idle_streak.saturating_add(1);
                    }
                    Err(err) => eprintln!("clipboard poll error: {err}"),
                }
            }
        });
    }
}

#[cfg(windows)]
fn clipboard_changed(last: u32) -> bool {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    let current = unsafe { GetClipboardSequenceNumber() };
    current != last
}

fn cache_source_icon(db: &Database, source: &Option<String>) {
    if let Some(name) = source {
        if let Some(exe) = crate::source_app::foreground_process_exe() {
            crate::app_icon::ensure_app_icon(db.data_dir(), name, &exe);
        }
    }
}

fn record_source() -> Option<String> {
    resolve_source_app()
}

fn poll_clipboard(
    clipboard: &mut Clipboard,
    db: Arc<Database>,
    images_dir: &PathBuf,
    thumbs_dir: &PathBuf,
    dedupe: bool,
    last_hash: &Mutex<Option<String>>,
    #[cfg(windows)] last_sequence: &AtomicU32,
) -> Result<Option<i64>, String> {
    #[cfg(windows)]
    {
        use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
        let current = unsafe { GetClipboardSequenceNumber() };
        last_sequence.store(current, Ordering::SeqCst);
    }

    if let Ok(text) = clipboard.get_text() {
        if !text.is_empty() {
            let source = record_source();
            if !should_record(&db.get_settings(), source.as_deref()) {
                return Ok(None);
            }
            if let Some(path) = detect_file_paths(&text) {
                let hash = content_hash("file", &path);
                if dedupe {
                    if let Some(id) = db.touch_existing_clip(&ContentType::File, &path)? {
                        *last_hash.lock() = Some(hash);
                        return Ok(Some(id));
                    }
                } else if is_duplicate(last_hash, &hash) {
                    return Ok(None);
                }
                let id = db.insert_clip(
                    ContentType::File,
                    &path,
                    None,
                    None,
                    source.as_deref(),
                )?;
                cache_source_icon(&db, &source);
                *last_hash.lock() = Some(hash);
                return Ok(Some(id));
            }

            let content_type = detect_content_type(&text);
            let hash = content_hash("text", &text);
            if dedupe {
                if let Some(id) = db.touch_existing_clip(&content_type, &text)? {
                    *last_hash.lock() = Some(hash);
                    return Ok(Some(id));
                }
            } else if is_duplicate(last_hash, &hash) {
                return Ok(None);
            }
            let id = db.insert_clip(content_type, &text, None, None, source.as_deref())?;
            cache_source_icon(&db, &source);
            *last_hash.lock() = Some(hash);
            return Ok(Some(id));
        }
    }

    if let Ok(img) = clipboard.get_image() {
        let source = record_source();
        if !should_record(&db.get_settings(), source.as_deref()) {
            return Ok(None);
        }
        let rgba = img.bytes.to_vec();
        let width = img.width;
        let height = img.height;
        let hash = content_hash(
            "image",
            &format!("{width}x{height}:{:?}", &rgba[..rgba.len().min(64)]),
        );
        if dedupe && is_duplicate(last_hash, &hash) {
            return Ok(None);
        }

        let settings = db.get_settings();
        let (saved, thumb_path) = crate::image_store::save_clipboard_image(
            &rgba,
            width,
            height,
            images_dir,
            thumbs_dir,
            &settings,
        )?;

        let id = db.insert_clip(
            ContentType::Image,
            "[图片]",
            Some(&saved.path.to_string_lossy()),
            Some(&thumb_path.to_string_lossy()),
            source.as_deref(),
        )?;
        cache_source_icon(&db, &source);
        *last_hash.lock() = Some(content_hash("image", &saved.hash_sample));

        #[cfg(windows)]
        if settings.enable_image_ocr {
            crate::ocr::spawn_ocr_job(Arc::clone(&db), id, saved.path.clone());
        }

        return Ok(Some(id));
    }

    Ok(None)
}

fn detect_content_type(text: &str) -> ContentType {
    let trimmed = text.trim();
    if (trimmed.starts_with('<') && trimmed.contains('>'))
        || trimmed.contains("<html")
        || trimmed.contains("<div")
        || trimmed.contains("<p")
    {
        ContentType::Html
    } else {
        ContentType::Text
    }
}

fn content_hash(kind: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn is_duplicate(last_hash: &Mutex<Option<String>>, hash: &str) -> bool {
    last_hash.lock().as_deref() == Some(hash)
}

pub fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    html_escape::decode_html_entities(&result)
        .to_string()
        .trim()
        .to_string()
}

mod html_escape {
    pub fn decode_html_entities(s: &str) -> std::borrow::Cow<'_, str> {
        s.replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .into()
    }
}
