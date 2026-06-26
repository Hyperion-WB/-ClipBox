use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub data_dir: Option<String>,
}

pub fn default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("clipbox")
}

pub fn config_file_path() -> PathBuf {
    default_data_dir().join("config.json")
}

pub fn resolve_data_dir() -> PathBuf {
    let path = config_file_path();
    if path.exists() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&text) {
                if let Some(dir) = cfg.data_dir.filter(|d| !d.is_empty()) {
                    return PathBuf::from(dir);
                }
            }
        }
    }
    default_data_dir()
}

pub fn save_data_dir(data_dir: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(default_data_dir()).map_err(|e| e.to_string())?;
    let cfg = AppConfig {
        data_dir: Some(data_dir.to_string_lossy().to_string()),
    };
    let text = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    std::fs::write(config_file_path(), text).map_err(|e| e.to_string())
}
