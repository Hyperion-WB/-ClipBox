export type ContentType = "text" | "html" | "image" | "file";
export type ClipCategory = "all" | "text" | "image" | "file";
export type ThemeMode = "system" | "light" | "dark";
export type AppLocale = "zh" | "en";

export interface ClipItem {
  id: number;
  content_type: ContentType;
  content_text: string;
  has_blob: boolean;
  has_thumbnail: boolean;
  pinned: boolean;
  source_app: string | null;
  created_at: string;
  last_used_at: string;
}

export interface Snippet {
  id: number;
  title: string;
  content: string;
  sort_order: number;
  created_at: string;
}

export interface AppSettings {
  max_history: number;
  hotkey: string;
  dedupe: boolean;
  start_on_boot: boolean;
  simulate_paste: boolean;
  dismiss_on_blur: boolean;
  poll_interval_ms: number;
  retention_days: number;
  image_retention_days: number;
  auto_cleanup: boolean;
  storage_path: string;
  theme: ThemeMode;
  locale: AppLocale;
  release_memory_on_close: boolean;
  group_by_time: boolean;
  group_by_source: boolean;
  enable_preview: boolean;
  enable_tag_filters: boolean;
  enable_smart_search: boolean;
  enable_notifications: boolean;
  app_filter_mode: "off" | "blacklist" | "whitelist";
  app_filter_list: string;
  minimal_mode: boolean;
  window_draggable: boolean;
  pinned_collapse_threshold: number;
  image_save_dir: string;
}

export interface HistoryStats {
  total_clips: number;
  pinned_clips: number;
  image_count: number;
  file_count: number;
  disk_bytes: number;
}

export type ContextMenuAction =
  | "paste"
  | "pastePlain"
  | "copy"
  | "saveImage"
  | "pin"
  | "delete";
