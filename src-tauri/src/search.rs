use crate::models::{ClipCategory, ContentType};
use chrono::{DateTime, Datelike, Duration, Utc};

#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub text_query: Option<String>,
    pub category: ClipCategory,
    pub time_range: Option<TimeRange>,
    pub source_app: Option<String>,
    pub tag: Option<ContentTag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Today,
    Yesterday,
    ThisWeek,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentTag {
    Url,
    Code,
    Image,
}

pub fn parse_search_query(raw: &str, category: ClipCategory) -> SearchFilters {
    let mut q = raw.trim().to_string();
    let mut filters = SearchFilters {
        category,
        ..Default::default()
    };

    for tag in ["#img", "#image", "#图片"] {
        if q.to_lowercase().contains(tag) {
            filters.tag = Some(ContentTag::Image);
            filters.category = ClipCategory::Image;
            q = q.replace(tag, "").replace(&tag[1..], "");
        }
    }
    for tag in ["#url", "#link", "#链接"] {
        if q.to_lowercase().contains(tag) {
            filters.tag = Some(ContentTag::Url);
            q = q.replace(tag, "").replace(&tag[1..], "");
        }
    }
    for tag in ["#code", "#代码"] {
        if q.to_lowercase().contains(tag) {
            filters.tag = Some(ContentTag::Code);
            q = q.replace(tag, "").replace(&tag[1..], "");
        }
    }

    let lower = q.to_lowercase();
    if lower.contains("昨天") {
        filters.time_range = Some(TimeRange::Yesterday);
        q = q.replace("昨天", "");
    } else if lower.contains("今天") {
        filters.time_range = Some(TimeRange::Today);
        q = q.replace("今天", "");
    } else if lower.contains("本周") || lower.contains("这周") {
        filters.time_range = Some(TimeRange::ThisWeek);
        q = q.replace("本周", "").replace("这周", "");
    }

    if lower.contains("链接") || lower.contains("url") {
        filters.tag = Some(ContentTag::Url);
        q = q.replace("链接", "").replace("url", "").replace("URL", "");
    }
    if lower.contains("图片") {
        filters.tag = Some(ContentTag::Image);
        filters.category = ClipCategory::Image;
        q = q.replace("图片", "");
    }
    if lower.contains("代码") || lower.contains("code") {
        filters.tag = Some(ContentTag::Code);
        q = q.replace("代码", "").replace("code", "").replace("Code", "");
    }

    let known_apps = [
        "chrome", "edge", "firefox", "vscode", "code", "微信", "wechat", "notepad",
    ];
    for app in known_apps {
        if lower.contains(app) {
            filters.source_app = Some(app.to_string());
            let re_app = regex_lite(app, &q);
            q = re_app;
        }
    }

    let trimmed = q.split_whitespace().collect::<Vec<_>>().join(" ");
    if !trimmed.is_empty() {
        filters.text_query = Some(trimmed);
    }

    filters
}

fn regex_lite(needle: &str, haystack: &str) -> String {
    haystack
        .split_whitespace()
        .filter(|w| !w.eq_ignore_ascii_case(needle))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn time_range_sql(range: TimeRange) -> (String, String) {
    let now = Utc::now();
    match range {
        TimeRange::Today => {
            let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let start: DateTime<Utc> = DateTime::from_naive_utc_and_offset(start, Utc);
            (start.to_rfc3339(), now.to_rfc3339())
        }
        TimeRange::Yesterday => {
            let yesterday = now - Duration::days(1);
            let start = yesterday.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let end = yesterday.date_naive().and_hms_opt(23, 59, 59).unwrap();
            (
                DateTime::<Utc>::from_naive_utc_and_offset(start, Utc).to_rfc3339(),
                DateTime::<Utc>::from_naive_utc_and_offset(end, Utc).to_rfc3339(),
            )
        }
        TimeRange::ThisWeek => {
            let weekday = now.weekday().num_days_from_monday();
            let start_date = now.date_naive() - Duration::days(weekday as i64);
            let start = start_date.and_hms_opt(0, 0, 0).unwrap();
            (
                DateTime::<Utc>::from_naive_utc_and_offset(start, Utc).to_rfc3339(),
                now.to_rfc3339(),
            )
        }
    }
}

pub fn matches_content_tag(
    tag: ContentTag,
    content_type: &ContentType,
    text: &str,
) -> bool {
    match tag {
        ContentTag::Image => *content_type == ContentType::Image,
        ContentTag::Url => is_url_like(text),
        ContentTag::Code => is_code_like(text, content_type),
    }
}

pub fn is_url_like(text: &str) -> bool {
    let t = text.trim();
    t.starts_with("http://")
        || t.starts_with("https://")
        || t.starts_with("www.")
        || (t.contains('.') && !t.contains(' ') && t.len() < 2000)
}

pub fn is_code_like(text: &str, content_type: &ContentType) -> bool {
    if *content_type == ContentType::Image || *content_type == ContentType::File {
        return false;
    }
    let t = text.trim();
    if t.len() < 4 {
        return false;
    }
    let markers = [
        "function ", "const ", "let ", "var ", "import ", "class ", "def ", "public ",
        "private ", "#include", "fn ", "async ", "=>", "();", "{\n", "}\n", "    ",
    ];
    markers.iter().any(|m| t.contains(m)) || t.lines().count() > 3
}
