use crate::app_filter::should_record;
use crate::db::Database;
use crate::models::ContentType;
use crate::source_app::{detect_file_paths, foreground_app_name};
use arboard::Clipboard;
use image::imageops::FilterType;
use image::ImageEncoder;
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
                    &db,
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

fn poll_clipboard(
    clipboard: &mut Clipboard,
    db: &Database,
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
            let source = foreground_app_name();
            if !should_record(&db.get_settings(), source.as_deref()) {
                return Ok(None);
            }
            if let Some(path) = detect_file_paths(&text) {
                let hash = content_hash("file", &path);
                if dedupe && is_duplicate(last_hash, &hash) {
                    return Ok(None);
                }
                let id = db.insert_clip(
                    ContentType::File,
                    &path,
                    None,
                    None,
                    source.as_deref(),
                )?;
                *last_hash.lock() = Some(hash);
                return Ok(Some(id));
            }

            let content_type = detect_content_type(&text);
            let hash = content_hash("text", &text);
            if dedupe && is_duplicate(last_hash, &hash) {
                return Ok(None);
            }
            let id = db.insert_clip(content_type, &text, None, None, source.as_deref())?;
            *last_hash.lock() = Some(hash);
            return Ok(Some(id));
        }
    }

    if let Ok(img) = clipboard.get_image() {
        let source = foreground_app_name();
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

        let png_bytes = encode_png(&rgba, width, height)?;
        let stamp = chrono::Utc::now().timestamp_millis();
        let filename = format!("{stamp}.png");
        let path = images_dir.join(&filename);
        std::fs::write(&path, &png_bytes).map_err(|e| e.to_string())?;

        let thumb_filename = format!("{stamp}_thumb.jpg");
        let thumb_path = thumbs_dir.join(&thumb_filename);
        save_thumbnail(&png_bytes, &thumb_path)?;

        let id = db.insert_clip(
            ContentType::Image,
            "[图片]",
            Some(&path.to_string_lossy()),
            Some(&thumb_path.to_string_lossy()),
            source.as_deref(),
        )?;
        *last_hash.lock() = Some(hash);
        return Ok(Some(id));
    }

    Ok(None)
}

fn save_thumbnail(png_bytes: &[u8], thumb_path: &PathBuf) -> Result<(), String> {
    let img = image::load_from_memory(png_bytes).map_err(|e| e.to_string())?;
    let thumb = img.resize(48, 48, FilterType::Triangle).into_rgb8();
    let mut buf = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 75);
    encoder
        .write_image(
            thumb.as_raw(),
            thumb.width(),
            thumb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| e.to_string())?;
    std::fs::write(thumb_path, buf).map_err(|e| e.to_string())?;
    Ok(())
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

fn encode_png(rgba: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder
        .write_image(rgba, width as u32, height as u32, image::ExtendedColorType::Rgba8)
        .map_err(|e| e.to_string())?;
    Ok(buf)
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
