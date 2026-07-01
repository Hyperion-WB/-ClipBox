mod schema;

use crate::backup;
use crate::models::{AppSettings, ClipCategory, ClipItem, ContentType, HistoryStats, Snippet};
use crate::search::{matches_content_tag, parse_search_query, time_range_sql, SearchFilters};
use chrono::{Duration, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Database {
    conn: Mutex<Connection>,
    data_dir: PathBuf,
}

impl Database {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        let db_path = data_dir.join("clipbox.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;
             PRAGMA synchronous=NORMAL;",
        )
        .map_err(|e| e.to_string())?;
        schema::init_schema(&conn)?;
        let db = Self {
            conn: Mutex::new(conn),
            data_dir,
        };
        db.seed_defaults();
        Ok(db)
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    fn seed_defaults(&self) {
        let defaults = [
            ("max_history", "300"),
            ("hotkey", "Ctrl+Shift+V"),
            ("dedupe", "true"),
            ("start_on_boot", "true"),
            ("simulate_paste", "true"),
            ("dismiss_on_blur", "true"),
            ("poll_interval_ms", "800"),
            ("retention_days", "7"),
            ("image_retention_days", "7"),
            ("auto_cleanup", "true"),
            ("storage_path", ""),
            ("theme", "system"),
            ("locale", "zh"),
            ("release_memory_on_close", "true"),
            ("group_by_time", "false"),
            ("group_by_source", "false"),
            ("enable_preview", "false"),
            ("enable_hover_preview", "true"),
            ("enable_tag_filters", "true"),
            ("enable_smart_search", "true"),
            ("enable_notifications", "false"),
            ("app_filter_mode", "off"),
            ("app_filter_list", "[]"),
            ("minimal_mode", "false"),
            ("window_draggable", "true"),
            ("pinned_collapse_threshold", "10"),
            ("image_save_dir", ""),
        ];
        let conn = self.conn.lock();
        for (key, value) in defaults {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            );
        }
    }

    pub fn get_setting(&self, key: &str) -> Option<String> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .ok()
        .flatten()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_settings(&self) -> AppSettings {
        let storage_path = self
            .get_setting("storage_path")
            .unwrap_or_default();
        let storage_path = if storage_path.is_empty() {
            self.data_dir.to_string_lossy().to_string()
        } else {
            storage_path
        };

        AppSettings {
            max_history: self
                .get_setting("max_history")
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            hotkey: self
                .get_setting("hotkey")
                .unwrap_or_else(|| "Ctrl+Shift+V".to_string()),
            dedupe: self
                .get_setting("dedupe")
                .map(|v| v == "true")
                .unwrap_or(true),
            start_on_boot: self
                .get_setting("start_on_boot")
                .map(|v| v == "true")
                .unwrap_or(true),
            simulate_paste: self
                .get_setting("simulate_paste")
                .map(|v| v == "true")
                .unwrap_or(true),
            dismiss_on_blur: self
                .get_setting("dismiss_on_blur")
                .map(|v| v == "true")
                .unwrap_or(true),
            poll_interval_ms: self
                .get_setting("poll_interval_ms")
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
            retention_days: self
                .get_setting("retention_days")
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            image_retention_days: self
                .get_setting("image_retention_days")
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            auto_cleanup: self
                .get_setting("auto_cleanup")
                .map(|v| v == "true")
                .unwrap_or(true),
            storage_path,
            theme: self
                .get_setting("theme")
                .unwrap_or_else(|| "system".to_string()),
            locale: self
                .get_setting("locale")
                .unwrap_or_else(|| "zh".to_string()),
            release_memory_on_close: self.bool_setting("release_memory_on_close", true),
            group_by_time: self.bool_setting("group_by_time", false),
            group_by_source: self.bool_setting("group_by_source", false),
            enable_preview: self.bool_setting("enable_preview", false),
            enable_tag_filters: self.bool_setting("enable_tag_filters", true),
            enable_smart_search: self.bool_setting("enable_smart_search", true),
            enable_notifications: self.bool_setting("enable_notifications", false),
            app_filter_mode: self
                .get_setting("app_filter_mode")
                .unwrap_or_else(|| "off".to_string()),
            app_filter_list: self
                .get_setting("app_filter_list")
                .unwrap_or_else(|| "[]".to_string()),
            minimal_mode: self.bool_setting("minimal_mode", false),
            window_draggable: self.bool_setting("window_draggable", true),
            pinned_collapse_threshold: self
                .get_setting("pinned_collapse_threshold")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            image_save_dir: self
                .get_setting("image_save_dir")
                .unwrap_or_default(),
            panel_follow_cursor: self.bool_setting("panel_follow_cursor", true),
            trash_retention_hours: self
                .get_setting("trash_retention_hours")
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
        }
    }

    fn bool_setting(&self, key: &str, default: bool) -> bool {
        self.get_setting(key)
            .map(|v| v == "true")
            .unwrap_or(default)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), String> {
        self.set_setting("max_history", &settings.max_history.to_string())?;
        self.set_setting("hotkey", &settings.hotkey)?;
        self.set_setting("dedupe", if settings.dedupe { "true" } else { "false" })?;
        self.set_setting(
            "start_on_boot",
            if settings.start_on_boot { "true" } else { "false" },
        )?;
        self.set_setting(
            "simulate_paste",
            if settings.simulate_paste { "true" } else { "false" },
        )?;
        self.set_setting(
            "dismiss_on_blur",
            if settings.dismiss_on_blur { "true" } else { "false" },
        )?;
        self.set_setting("poll_interval_ms", &settings.poll_interval_ms.to_string())?;
        self.set_setting("retention_days", &settings.retention_days.to_string())?;
        self.set_setting(
            "image_retention_days",
            &settings.image_retention_days.to_string(),
        )?;
        self.set_setting(
            "auto_cleanup",
            if settings.auto_cleanup { "true" } else { "false" },
        )?;
        self.set_setting("theme", &settings.theme)?;
        self.set_setting("locale", &settings.locale)?;
        self.set_setting(
            "release_memory_on_close",
            if settings.release_memory_on_close {
                "true"
            } else {
                "false"
            },
        )?;
        let storage = if settings.storage_path == self.data_dir.to_string_lossy() {
            String::new()
        } else {
            settings.storage_path.clone()
        };
        self.set_setting("storage_path", &storage)?;
        self.set_bool("group_by_time", settings.group_by_time)?;
        self.set_bool("group_by_source", settings.group_by_source)?;
        self.set_bool("enable_preview", settings.enable_preview)?;
        self.set_bool("enable_tag_filters", settings.enable_tag_filters)?;
        self.set_bool("enable_smart_search", settings.enable_smart_search)?;
        self.set_bool("enable_notifications", settings.enable_notifications)?;
        self.set_setting("app_filter_mode", &settings.app_filter_mode)?;
        self.set_setting("app_filter_list", &settings.app_filter_list)?;
        self.set_bool("minimal_mode", settings.minimal_mode)?;
        self.set_bool("window_draggable", settings.window_draggable)?;
        self.set_setting(
            "pinned_collapse_threshold",
            &settings.pinned_collapse_threshold.to_string(),
        )?;
        self.set_setting("image_save_dir", &settings.image_save_dir)?;
        self.set_setting(
            "panel_follow_cursor",
            if settings.panel_follow_cursor { "true" } else { "false" },
        )?;
        self.set_setting(
            "trash_retention_hours",
            &settings.trash_retention_hours.to_string(),
        )?;
        Ok(())
    }

    fn set_bool(&self, key: &str, value: bool) -> Result<(), String> {
        self.set_setting(key, if value { "true" } else { "false" })
    }

    pub fn insert_clip(
        &self,
        content_type: ContentType,
        content_text: &str,
        image_path: Option<&str>,
        thumb_path: Option<&str>,
        source_app: Option<&str>,
    ) -> Result<i64, String> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO clips (content_type, content_text, content_blob, image_path, thumb_path, source_app, pinned, created_at, last_used_at)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, 0, ?6, ?6)",
            params![
                content_type.as_str(),
                content_text,
                image_path,
                thumb_path,
                source_app,
                now,
            ],
        )
        .map_err(|e| e.to_string())?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.prune_history()?;
        Ok(id)
    }

    /// Find an existing clip with the same type and text; touch it and remove other duplicates.
    pub fn touch_existing_clip(
        &self,
        content_type: &ContentType,
        content_text: &str,
    ) -> Result<Option<i64>, String> {
        let conn = self.conn.lock();
        let keep_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM clips
                 WHERE content_type = ?1 AND content_text = ?2 AND deleted_at IS NULL
                 ORDER BY pinned DESC, last_used_at DESC
                 LIMIT 1",
                params![content_type.as_str(), content_text],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        let Some(keep_id) = keep_id else {
            return Ok(None);
        };

        let dup_ids: Vec<i64> = conn
            .prepare(
                "SELECT id FROM clips
                 WHERE content_type = ?1 AND content_text = ?2 AND id != ?3 AND deleted_at IS NULL",
            )
            .map_err(|e| e.to_string())?
            .query_map(params![content_type.as_str(), content_text, keep_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        drop(conn);

        for id in dup_ids {
            let _ = self.permanently_delete_clip(id);
        }

        self.touch_clip(keep_id)?;
        Ok(Some(keep_id))
    }

    pub fn get_clip_thumbnail_data_url(&self, id: i64) -> Result<Option<String>, String> {
        let conn = self.conn.lock();
        let thumb_path: Option<String> = conn
            .query_row(
                "SELECT thumb_path FROM clips WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();

        let Some(path) = thumb_path else {
            return Ok(None);
        };

        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        Ok(Some(format!("data:image/jpeg;base64,{}", encoded)))
    }

    pub fn list_clips(
        &self,
        query: Option<&str>,
        category: ClipCategory,
        smart_search: bool,
    ) -> Result<Vec<ClipItem>, String> {
        let filters = if smart_search {
            parse_search_query(query.unwrap_or(""), category)
        } else {
            SearchFilters {
                text_query: query.filter(|s| !s.trim().is_empty()).map(|s| s.to_string()),
                category,
                ..Default::default()
            }
        };
        self.list_clips_filtered(&filters)
    }

    pub fn list_clips_filtered(&self, filters: &SearchFilters) -> Result<Vec<ClipItem>, String> {
        let conn = self.conn.lock();
        let category_sql = category_where_clause(&filters.category);
        let mut sql = format!(
            "SELECT id, content_type, content_text, content_blob IS NOT NULL,
                    thumb_path IS NOT NULL, pinned, source_app, created_at, last_used_at
             FROM clips WHERE ({category_sql}) AND deleted_at IS NULL"
        );
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some((start, end)) = filters.time_range.map(time_range_sql) {
            sql.push_str(" AND last_used_at >= ? AND last_used_at <= ?");
            params_vec.push(Box::new(start));
            params_vec.push(Box::new(end));
        }

        if let Some(ref src) = filters.source_app {
            sql.push_str(" AND LOWER(COALESCE(source_app, '')) LIKE ?");
            params_vec.push(Box::new(format!("%{}%", src.to_lowercase())));
        }

        if let Some(ref text) = filters.text_query {
            if crate::search::query_has_cjk(text) {
                sql.push_str(" AND content_text LIKE ?");
                params_vec.push(Box::new(format!("%{}%", text)));
            } else {
                sql.push_str(
                    " AND (content_text LIKE ? OR id IN (SELECT rowid FROM clips_fts WHERE clips_fts MATCH ?))",
                );
                params_vec.push(Box::new(format!("%{}%", text)));
                params_vec.push(Box::new(format!(
                    "{}*",
                    text.replace('"', "\"\"")
                )));
            }
        }

        sql.push_str(" ORDER BY pinned DESC, last_used_at DESC LIMIT 200");

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), row_to_clip)
            .map_err(|e| e.to_string())?;

        let mut items = Vec::new();
        for row in rows {
            let item = row.map_err(|e| e.to_string())?;
            if let Some(tag) = filters.tag {
                if !matches_content_tag(tag, &item.content_type, &item.content_text) {
                    continue;
                }
            }
            items.push(item);
        }
        Ok(items)
    }

    pub fn get_clip_image_data_url(&self, id: i64) -> Result<Option<String>, String> {
        let conn = self.conn.lock();
        let image_path: Option<String> = conn
            .query_row(
                "SELECT image_path FROM clips WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();

        let Some(path) = image_path else {
            return Ok(None);
        };

        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        Ok(Some(format!("data:image/png;base64,{}", encoded)))
    }

    pub fn save_clip_image(&self, id: i64, dest_dir: &str) -> Result<String, String> {
        let conn = self.conn.lock();
        let image_path: Option<String> = conn
            .query_row(
                "SELECT image_path FROM clips WHERE id = ?1 AND content_type = 'image'",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();

        let Some(src) = image_path else {
            return Err("该条目没有可保存的图片".to_string());
        };

        if !std::path::Path::new(&src).exists() {
            return Err("图片文件不存在".to_string());
        }

        let dest = PathBuf::from(dest_dir);
        if !dest.is_dir() {
            return Err("目标路径不是文件夹".to_string());
        }

        let ext = std::path::Path::new(&src)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let stem = format!(
            "clipbox_{}",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let mut dest_file = dest.join(format!("{stem}.{ext}"));
        let mut n = 1u32;
        while dest_file.exists() {
            dest_file = dest.join(format!("{stem}_{n}.{ext}"));
            n += 1;
        }

        std::fs::copy(&src, &dest_file).map_err(|e| e.to_string())?;
        Ok(dest_file.to_string_lossy().to_string())
    }

    pub fn get_history_stats(&self) -> Result<HistoryStats, String> {
        let conn = self.conn.lock();
        let total_clips: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM clips WHERE deleted_at IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let pinned_clips: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM clips WHERE pinned = 1 AND deleted_at IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let image_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM clips WHERE content_type = 'image' AND deleted_at IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let file_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM clips WHERE content_type = 'file' AND deleted_at IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let trash_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM clips WHERE deleted_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let disk_bytes = backup::dir_size(self.data_dir());
        Ok(HistoryStats {
            total_clips,
            pinned_clips,
            image_count,
            file_count,
            disk_bytes,
            trash_count,
        })
    }

    pub fn get_clip_blob(&self, id: i64) -> Result<Option<Vec<u8>>, String> {
        let conn = self.conn.lock();
        if let Some(blob) = conn
            .query_row(
                "SELECT content_blob FROM clips WHERE id = ?1",
                params![id],
                |row| row.get::<_, Option<Vec<u8>>>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten()
        {
            return Ok(Some(blob));
        }

        let image_path: Option<String> = conn
            .query_row(
                "SELECT image_path FROM clips WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();

        match image_path {
            Some(path) => std::fs::read(&path)
                .map(Some)
                .map_err(|e| e.to_string()),
            None => Ok(None),
        }
    }

    pub fn get_clip(&self, id: i64) -> Result<ClipItem, String> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, content_type, content_text, content_blob IS NOT NULL,
                    thumb_path IS NOT NULL, pinned, source_app, created_at, last_used_at
             FROM clips WHERE id = ?1 AND deleted_at IS NULL",
            params![id],
            row_to_clip,
        )
        .map_err(|e| e.to_string())
    }

    pub fn toggle_pin(&self, id: i64) -> Result<bool, String> {
        let conn = self.conn.lock();
        let pinned: i32 = conn
            .query_row("SELECT pinned FROM clips WHERE id = ?1", params![id], |row| {
                row.get(0)
            })
            .map_err(|e| e.to_string())?;
        let new_pinned = if pinned == 1 { 0 } else { 1 };
        conn.execute(
            "UPDATE clips SET pinned = ?1 WHERE id = ?2",
            params![new_pinned, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(new_pinned == 1)
    }

    pub fn delete_clip(&self, id: i64) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock();
        let row: Option<(String,)> = conn
            .query_row(
                "SELECT content_text FROM clips WHERE id = ?1 AND deleted_at IS NULL",
                params![id],
                |row| Ok((row.get(0)?,)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((text,)) = row else {
            return Ok(());
        };
        conn.execute(
            "UPDATE clips SET deleted_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(|e| e.to_string())?;
        fts_remove(&conn, id, &text)?;
        Ok(())
    }

    pub fn permanently_delete_clip(&self, id: i64) -> Result<(), String> {
        self.remove_clip_files(id)?;
        let conn = self.conn.lock();
        conn.execute("DELETE FROM clips WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn restore_clip(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock();
        let updated = conn
            .execute(
                "UPDATE clips SET deleted_at = NULL WHERE id = ?1 AND deleted_at IS NOT NULL",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        if updated == 0 {
            return Err("条目不在回收站中".to_string());
        }
        Ok(())
    }

    pub fn list_trash_clips(&self) -> Result<Vec<ClipItem>, String> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, content_type, content_text, content_blob IS NOT NULL,
                        thumb_path IS NOT NULL, pinned, source_app, created_at, last_used_at
                 FROM clips WHERE deleted_at IS NOT NULL
                 ORDER BY deleted_at DESC LIMIT 100",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], row_to_clip)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn empty_trash(&self) -> Result<u32, String> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT id FROM clips WHERE deleted_at IS NOT NULL")
            .map_err(|e| e.to_string())?;
        let ids: Vec<i64> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);
        drop(conn);
        let mut n = 0u32;
        for id in ids {
            self.permanently_delete_clip(id)?;
            n += 1;
        }
        Ok(n)
    }

    pub fn purge_expired_trash(&self) -> Result<u32, String> {
        let hours = self.get_settings().trash_retention_hours.max(1) as i64;
        let cutoff = (Utc::now() - Duration::hours(hours)).to_rfc3339();
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT id FROM clips WHERE deleted_at IS NOT NULL AND deleted_at < ?1")
            .map_err(|e| e.to_string())?;
        let ids: Vec<i64> = stmt
            .query_map(params![cutoff], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);
        drop(conn);
        let mut n = 0u32;
        for id in ids {
            self.permanently_delete_clip(id)?;
            n += 1;
        }
        Ok(n)
    }

    pub fn merge_duplicate_clips(&self) -> Result<u32, String> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, content_type, content_text, pinned, last_used_at
                 FROM clips WHERE deleted_at IS NULL
                 ORDER BY pinned DESC, last_used_at DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(i64, String, String, i32, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);
        drop(conn);

        let mut seen: HashMap<(String, String), i64> = HashMap::new();
        let mut remove_ids: Vec<i64> = Vec::new();
        for (id, ctype, text, _pinned, _used) in rows {
            let key = (ctype, text);
            if seen.contains_key(&key) {
                remove_ids.push(id);
            } else {
                seen.insert(key, id);
            }
        }

        let mut merged = 0u32;
        for id in remove_ids {
            self.permanently_delete_clip(id)?;
            merged += 1;
        }
        Ok(merged)
    }

    pub fn delete_clips(&self, ids: &[i64]) -> Result<(), String> {
        for id in ids {
            self.delete_clip(*id)?;
        }
        Ok(())
    }

    pub fn clear_history(&self, keep_pinned: bool) -> Result<(), String> {
        if keep_pinned {
            let conn = self.conn.lock();
            let mut stmt = conn
                .prepare("SELECT id FROM clips WHERE pinned = 0 AND deleted_at IS NULL")
                .map_err(|e| e.to_string())?;
            let ids: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);
            drop(conn);
            self.delete_clips(&ids)?;
        } else {
            let conn = self.conn.lock();
            let mut stmt = conn
                .prepare("SELECT id FROM clips WHERE deleted_at IS NULL")
                .map_err(|e| e.to_string())?;
            let ids: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);
            drop(conn);
            self.delete_clips(&ids)?;
        }
        Ok(())
    }

    fn remove_clip_files(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock();
        let paths: (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT image_path, thumb_path FROM clips WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((None, None));
        drop(conn);
        for path in [paths.0, paths.1].into_iter().flatten() {
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    }

    pub fn touch_clip(&self, id: i64) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE clips SET last_used_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn prune_history(&self) -> Result<(), String> {
        let max_history = self.get_settings().max_history;
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id FROM clips WHERE pinned = 0 AND deleted_at IS NULL AND id NOT IN (
                    SELECT id FROM clips WHERE deleted_at IS NULL ORDER BY last_used_at DESC LIMIT ?1
                 )",
            )
            .map_err(|e| e.to_string())?;
        let ids: Vec<i64> = stmt
            .query_map(params![max_history], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);
        drop(conn);
        for id in ids {
            let _ = self.delete_clip(id);
        }
        Ok(())
    }

    pub fn cleanup_expired(&self) -> Result<u32, String> {
        let settings = self.get_settings();
        if !settings.auto_cleanup {
            return Ok(0);
        }

        let mut removed = 0u32;
        let now = Utc::now();

        if settings.retention_days > 0 {
            let cutoff = (now - Duration::days(settings.retention_days as i64)).to_rfc3339();
            let conn = self.conn.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT id FROM clips WHERE pinned = 0 AND content_type NOT IN ('image')
                     AND last_used_at < ?1",
                )
                .map_err(|e| e.to_string())?;
            let ids: Vec<i64> = stmt
                .query_map(params![cutoff], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);
            drop(conn);
            for id in ids {
                self.delete_clip(id)?;
                removed += 1;
            }
        }

        if settings.image_retention_days > 0 {
            let cutoff = (now - Duration::days(settings.image_retention_days as i64)).to_rfc3339();
            let conn = self.conn.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT id FROM clips WHERE pinned = 0 AND content_type = 'image'
                     AND last_used_at < ?1",
                )
                .map_err(|e| e.to_string())?;
            let ids: Vec<i64> = stmt
                .query_map(params![cutoff], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);
            drop(conn);
            for id in ids {
                self.permanently_delete_clip(id)?;
                removed += 1;
            }
        }

        let _ = self.purge_expired_trash();

        Ok(removed)
    }

    pub fn list_snippets(&self) -> Result<Vec<Snippet>, String> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, sort_order, created_at
                 FROM snippets ORDER BY sort_order ASC, id ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Snippet {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    sort_order: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn create_snippet(&self, title: &str, content: &str) -> Result<Snippet, String> {
        if title.len() > 200 {
            return Err("标题过长".to_string());
        }
        if content.len() > 512_000 {
            return Err("片段内容过长".to_string());
        }
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock();
        let max_order: i32 = conn
            .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM snippets", [], |row| {
                row.get(0)
            })
            .unwrap_or(-1);
        conn.execute(
            "INSERT INTO snippets (title, content, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![title, content, max_order + 1, now],
        )
        .map_err(|e| e.to_string())?;
        let id = conn.last_insert_rowid();
        Ok(Snippet {
            id,
            title: title.to_string(),
            content: content.to_string(),
            sort_order: max_order + 1,
            created_at: now,
        })
    }

    pub fn update_snippet(&self, id: i64, title: &str, content: &str) -> Result<(), String> {
        if title.len() > 200 {
            return Err("标题过长".to_string());
        }
        if content.len() > 512_000 {
            return Err("片段内容过长".to_string());
        }
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE snippets SET title = ?1, content = ?2 WHERE id = ?3",
            params![title, content, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_snippet(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reorder_snippets(&self, ids: &[i64]) -> Result<(), String> {
        let conn = self.conn.lock();
        for (order, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE snippets SET sort_order = ?1 WHERE id = ?2",
                params![order as i32, id],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

fn fts_remove(conn: &Connection, id: i64, text: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO clips_fts(clips_fts, rowid, content_text) VALUES ('delete', ?1, ?2)",
        params![id, text],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn category_where_clause(category: &ClipCategory) -> &'static str {
    match category {
        ClipCategory::All => "1=1",
        ClipCategory::Text => "content_type IN ('text', 'html')",
        ClipCategory::Image => "content_type = 'image'",
        ClipCategory::File => "content_type = 'file'",
    }
}

fn row_to_clip(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipItem> {
    Ok(ClipItem {
        id: row.get(0)?,
        content_type: ContentType::from_str(&row.get::<_, String>(1)?),
        content_text: row.get(2)?,
        has_blob: row.get(3)?,
        has_thumbnail: row.get(4)?,
        pinned: row.get::<_, i32>(5)? == 1,
        source_app: row.get(6)?,
        created_at: row.get(7)?,
        last_used_at: row.get(8)?,
    })
}
