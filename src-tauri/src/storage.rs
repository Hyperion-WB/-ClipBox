use crate::backup;
use crate::db::Database;
use crate::models::{StorageDetails, StorageLargestItem};
use std::path::PathBuf;

pub fn get_storage_details(db: &Database) -> Result<StorageDetails, String> {
    let data_dir = db.data_dir().clone();
    let db_bytes = Database::database_files_bytes(&data_dir);
    let images_bytes = backup::dir_size(&data_dir.join("images"));
    let thumbs_bytes = backup::dir_size(&data_dir.join("thumbs"));
    let app_icons_bytes = backup::dir_size(&data_dir.join("app-icons"));
    let total_bytes = backup::dir_size(&data_dir);

    let (orphan_count, orphan_bytes) = count_orphan_media(db)?;
    let (trash_count, trash_bytes) = trash_usage(db)?;
    let active_count = db.active_clip_count()?;
    let largest = largest_clips(db, 8)?;

    Ok(StorageDetails {
        data_dir: data_dir.to_string_lossy().to_string(),
        total_bytes,
        db_bytes,
        images_bytes,
        thumbs_bytes,
        app_icons_bytes,
        orphan_count,
        orphan_bytes,
        trash_count,
        trash_bytes,
        active_count,
        largest,
    })
}

fn count_orphan_media(db: &Database) -> Result<(u32, u64), String> {
    let referenced = db.referenced_media_paths()?;
    let mut count = 0u32;
    let mut bytes = 0u64;
    for sub in ["images", "thumbs"] {
        let dir = db.data_dir().join(sub);
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
            let path = entry.path();
            if path.is_file() && !referenced.contains(&path) {
                if let Ok(meta) = entry.metadata() {
                    bytes += meta.len();
                }
                count += 1;
            }
        }
    }
    Ok((count, bytes))
}

fn trash_usage(db: &Database) -> Result<(i64, u64), String> {
    let paths = db.trash_media_paths()?;
    let mut bytes = 0u64;
    for path in paths {
        if let Ok(meta) = std::fs::metadata(&path) {
            bytes += meta.len();
        }
    }
    Ok((db.trash_count()?, bytes))
}

fn largest_clips(db: &Database, limit: usize) -> Result<Vec<StorageLargestItem>, String> {
    let rows = db.clip_media_sizes()?;
    let mut items: Vec<StorageLargestItem> = rows
        .into_iter()
        .filter(|(_, _, _, bytes)| *bytes > 0)
        .map(|(id, ctype, preview, bytes)| StorageLargestItem {
            id,
            content_type: ctype,
            preview,
            bytes,
        })
        .collect();
    items.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    items.truncate(limit);
    Ok(items)
}
