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
            q = q.replace(tag, "");
        }
    }
    for tag in ["#url", "#link", "#链接"] {
        if q.to_lowercase().contains(tag) {
            filters.tag = Some(ContentTag::Url);
            q = q.replace(tag, "");
        }
    }
    for tag in ["#code", "#代码"] {
        if q.to_lowercase().contains(tag) {
            filters.tag = Some(ContentTag::Code);
            q = q.replace(tag, "");
        }
    }

    let tokens: Vec<String> = q
        .split_whitespace()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut remaining = Vec::new();
    for token in &tokens {
        let lower = token.to_lowercase();
        if matches_token(&lower, &["昨天", "yesterday"]) {
            filters.time_range = Some(TimeRange::Yesterday);
            continue;
        }
        if matches_token(&lower, &["今天", "today"]) {
            filters.time_range = Some(TimeRange::Today);
            continue;
        }
        if matches_token(&lower, &["本周", "这周", "thisweek"]) {
            filters.time_range = Some(TimeRange::ThisWeek);
            continue;
        }
        if matches_token(&lower, &["链接", "url", "link"]) {
            filters.tag = Some(ContentTag::Url);
            continue;
        }
        if matches_token(&lower, &["图片", "image", "img"]) {
            filters.tag = Some(ContentTag::Image);
            filters.category = ClipCategory::Image;
            continue;
        }
        if matches_token(&lower, &["代码", "code"]) {
            filters.tag = Some(ContentTag::Code);
            continue;
        }

        let known_apps = [
            ("chrome", "chrome"),
            ("edge", "edge"),
            ("firefox", "firefox"),
            ("vscode", "vscode"),
            ("微信", "微信"),
            ("wechat", "wechat"),
            ("weixin", "wechat"),
            ("豆包", "豆包"),
            ("doubao", "豆包"),
            ("cursor", "Cursor 编辑器"),
            ("notepad", "notepad"),
        ];
        let mut matched_app = false;
        for (needle, canonical) in known_apps {
            if lower == needle {
                filters.source_app = Some(canonical.to_string());
                matched_app = true;
                break;
            }
        }
        if matched_app {
            continue;
        }

        remaining.push(token.clone());
    }

    let trimmed = remaining.join(" ");
    if !trimmed.is_empty() {
        filters.text_query = Some(trimmed);
    }

    filters
}

fn matches_token(token: &str, options: &[&str]) -> bool {
    options.iter().any(|o| token.eq_ignore_ascii_case(o))
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
    if t.len() < 8 {
        return false;
    }

    let cjk_count = t
        .chars()
        .filter(|ch| {
            let u = *ch as u32;
            (0x4E00..=0x9FFF).contains(&u)
        })
        .count();
    let cjk_ratio = cjk_count as f64 / t.len().max(1) as f64;
    let has_code_kw = [
        "function", "const ", "let ", "var ", "import ", "export ", "class ",
        "def ", "fn ", "public ", "private ",
    ]
    .iter()
    .any(|kw| t.contains(kw));
    if cjk_ratio > 0.12 && !has_code_kw {
        return false;
    }

    let strong = [
        "function ", "const ", "let ", "import ", "export ", "class ", "def ",
        "fn ", "public ", "#include", "=>", "package ", "using namespace",
    ];
    if strong.iter().any(|m| t.contains(m)) {
        return true;
    }

    let lines: Vec<&str> = t.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() < 3 {
        return false;
    }

    let code_lines = lines.iter().filter(|line| {
        let s = line.trim();
        s.starts_with("const ")
            || s.starts_with("let ")
            || s.starts_with("import ")
            || s.starts_with("function ")
            || s.starts_with("class ")
            || s.starts_with("def ")
            || s.starts_with("fn ")
            || ((s.ends_with(';') || s.ends_with('{') || s.ends_with('}'))
                && s.chars().any(|c| c.is_ascii_alphabetic()))
    }).count();

    code_lines >= 2 && (code_lines as f64 / lines.len() as f64) >= 0.4
}

pub fn query_has_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        let u = c as u32;
        (0x4E00..=0x9FFF).contains(&u) || (0x3400..=0x4DBF).contains(&u)
    })
}
