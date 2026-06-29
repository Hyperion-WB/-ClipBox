import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, ClipCategory, ClipItem, HistoryStats, Snippet } from "./types";

export const api = {
  listClips: (query?: string, category?: ClipCategory) =>
    invoke<ClipItem[]>("list_clips", {
      query: query ?? null,
      category: category && category !== "all" ? category : null,
    }),

  getClipThumbnail: (id: number) =>
    invoke<string | null>("get_clip_thumbnail", { id }),

  getClipImage: (id: number) =>
    invoke<string | null>("get_clip_image", { id }),

  saveClipImage: (id: number, destDir: string) =>
    invoke<string>("save_clip_image", { id, destDir }),

  getRecentNotifications: () =>
    invoke<ClipItem[]>("get_recent_notifications"),

  getHistoryStats: () => invoke<HistoryStats>("get_history_stats"),

  togglePin: (id: number) => invoke<boolean>("toggle_pin", { id }),

  deleteClip: (id: number) => invoke<void>("delete_clip", { id }),

  deleteClips: (ids: number[]) => invoke<void>("delete_clips", { ids }),

  clearHistory: (keepPinned: boolean) =>
    invoke<void>("clear_history", { keepPinned }),

  pasteItem: (id: number, plainTextOnly = false) =>
    invoke<void>("paste_item", { id, plainTextOnly }),

  copyItemToClipboard: (id: number, plainTextOnly = false) =>
    invoke<void>("copy_item_to_clipboard", { id, plainTextOnly }),

  listSnippets: () => invoke<Snippet[]>("list_snippets"),

  createSnippet: (title: string, content: string) =>
    invoke<Snippet>("create_snippet", { title, content }),

  updateSnippet: (id: number, title: string, content: string) =>
    invoke<void>("update_snippet", { id, title, content }),

  deleteSnippet: (id: number) => invoke<void>("delete_snippet", { id }),

  reorderSnippets: (ids: number[]) =>
    invoke<void>("reorder_snippets", { ids }),

  pasteSnippet: (content: string) =>
    invoke<void>("paste_snippet_cmd", { content }),

  pasteText: (text: string) => invoke<void>("paste_text", { text }),

  openPath: (path: string) => invoke<void>("open_path", { path }),

  openUrl: (url: string) => invoke<void>("open_url", { url }),

  getAppIcon: (appName: string) =>
    invoke<string | null>("get_app_icon", { appName }),

  getSettings: () => invoke<AppSettings>("get_settings"),

  saveSettings: (settings: AppSettings) =>
    invoke<void>("save_settings", { settings }),

  migrateStoragePath: (newPath: string) =>
    invoke<string>("migrate_storage_path", { newPath }),

  exportBackup: (dest: string) => invoke<void>("export_backup_cmd", { dest }),

  importBackup: (destPath: string, src: string) =>
    invoke<string>("import_backup_cmd", { destPath, src }),

  runCleanup: () => invoke<number>("run_cleanup"),

  formatDiskSize: (bytes: number) =>
    invoke<string>("format_disk_size", { bytes }),

  setMonitorPaused: (paused: boolean) =>
    invoke<void>("set_monitor_paused", { paused }),

  isMonitorPaused: () => invoke<boolean>("is_monitor_paused"),

  showPanel: () => invoke<void>("show_panel"),

  hidePanel: () => invoke<void>("hide_panel"),

  togglePanel: () => invoke<boolean>("toggle_panel"),
};
