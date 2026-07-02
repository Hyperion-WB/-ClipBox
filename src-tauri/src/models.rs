use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text,
    Html,
    Image,
    File,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Html => "html",
            ContentType::Image => "image",
            ContentType::File => "file",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "html" => ContentType::Html,
            "image" => ContentType::Image,
            "file" => ContentType::File,
            _ => ContentType::Text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ClipCategory {
    #[default]
    All,
    Text,
    Image,
    File,
}

impl ClipCategory {
    pub fn from_str(s: &str) -> Self {
        match s {
            "text" => ClipCategory::Text,
            "image" => ClipCategory::Image,
            "file" => ClipCategory::File,
            _ => ClipCategory::All,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipItem {
    pub id: i64,
    pub content_type: ContentType,
    pub content_text: String,
    pub has_blob: bool,
    pub has_thumbnail: bool,
    pub pinned: bool,
    pub source_app: Option<String>,
    pub created_at: String,
    pub last_used_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub max_history: i32,
    pub hotkey: String,
    pub dedupe: bool,
    pub start_on_boot: bool,
    pub simulate_paste: bool,
    pub dismiss_on_blur: bool,
    pub poll_interval_ms: i32,
    pub retention_days: i32,
    pub image_retention_days: i32,
    pub auto_cleanup: bool,
    pub storage_path: String,
    pub theme: String,
    pub locale: String,
    pub release_memory_on_close: bool,
    pub group_by_time: bool,
    pub group_by_source: bool,
    pub enable_preview: bool,
    pub enable_tag_filters: bool,
    pub enable_smart_search: bool,
    pub enable_notifications: bool,
    pub app_filter_mode: String,
    pub app_filter_list: String,
    pub minimal_mode: bool,
    pub window_draggable: bool,
    pub pinned_collapse_threshold: i32,
    pub image_save_dir: String,
    pub panel_follow_cursor: bool,
    pub trash_retention_hours: i32,
    pub compress_images: bool,
    pub image_max_dimension: i32,
    pub image_jpeg_quality: i32,
    pub image_compress_min_kb: i32,
    pub mask_sensitive: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_history: 300,
            hotkey: "Ctrl+Shift+V".to_string(),
            dedupe: true,
            start_on_boot: true,
            simulate_paste: true,
            dismiss_on_blur: true,
            poll_interval_ms: 800,
            retention_days: 7,
            image_retention_days: 7,
            auto_cleanup: true,
            storage_path: String::new(),
            theme: "system".to_string(),
            locale: "zh".to_string(),
            release_memory_on_close: true,
            group_by_time: false,
            group_by_source: false,
            enable_preview: false,
            enable_tag_filters: true,
            enable_smart_search: true,
            enable_notifications: false,
            app_filter_mode: "off".to_string(),
            app_filter_list: "[]".to_string(),
            minimal_mode: false,
            window_draggable: true,
            pinned_collapse_threshold: 10,
            image_save_dir: String::new(),
            panel_follow_cursor: true,
            trash_retention_hours: 24,
            compress_images: true,
            image_max_dimension: 1920,
            image_jpeg_quality: 82,
            image_compress_min_kb: 512,
            mask_sensitive: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total_clips: i64,
    pub pinned_clips: i64,
    pub image_count: i64,
    pub file_count: i64,
    pub disk_bytes: u64,
    pub trash_count: i64,
    pub db_bytes: u64,
    pub media_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLargestItem {
    pub id: i64,
    pub content_type: String,
    pub preview: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDetails {
    pub data_dir: String,
    pub total_bytes: u64,
    pub db_bytes: u64,
    pub images_bytes: u64,
    pub thumbs_bytes: u64,
    pub app_icons_bytes: u64,
    pub orphan_count: u32,
    pub orphan_bytes: u64,
    pub trash_count: i64,
    pub trash_bytes: u64,
    pub active_count: i64,
    pub largest: Vec<StorageLargestItem>,
}
