use crate::models::AppSettings;

pub fn should_record(settings: &AppSettings, source_app: Option<&str>) -> bool {
    if settings.app_filter_mode == "off" {
        return true;
    }
    let list = parse_list(&settings.app_filter_list);
    if list.is_empty() {
        return true;
    }
    let source = source_app.unwrap_or("").to_lowercase();
    let matched = list.iter().any(|entry| {
        let e = entry.to_lowercase();
        source.contains(&e) || e.contains(&source)
    });
    match settings.app_filter_mode.as_str() {
        "blacklist" => !matched,
        "whitelist" => matched,
        _ => true,
    }
}

fn parse_list(raw: &str) -> Vec<String> {
    if raw.trim().is_empty() {
        return Vec::new();
    }
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(raw) {
        return arr;
    }
    raw.split([',', ';', '\n'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn list_to_json(items: &[String]) -> String {
    serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())
}
