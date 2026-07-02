#[cfg(windows)]
pub fn recognize_image(path: &std::path::Path) -> Result<String, String> {
    use std::time::Duration;
    use windows::core::HSTRING;
    use windows::Foundation::{AsyncStatus, IAsyncOperation};
    use windows::Graphics::Imaging::BitmapDecoder;
    use windows::Media::Ocr::OcrEngine;
    use windows::Storage::FileAccessMode;
    use windows::Storage::StorageFile;

    fn wait<T>(op: IAsyncOperation<T>) -> Result<T, String>
    where
        T: windows::core::RuntimeType,
    {
        loop {
            match op.Status().map_err(|e| e.to_string())? {
                AsyncStatus::Started => std::thread::sleep(Duration::from_millis(15)),
                AsyncStatus::Completed => {
                    return op.GetResults().map_err(|e| e.to_string());
                }
                AsyncStatus::Error => {
                    return Err("OCR 异步操作失败".to_string());
                }
                AsyncStatus::Canceled => {
                    return Err("OCR 操作已取消".to_string());
                }
                _ => return Err("OCR 未知状态".to_string()),
            }
        }
    }

    let abs = path
        .canonicalize()
        .map_err(|e| format!("无法读取图片路径: {e}"))?;
    let path_h = HSTRING::from(abs.to_string_lossy().as_ref());

    let file = wait(
        StorageFile::GetFileFromPathAsync(&path_h).map_err(|e| e.to_string())?,
    )?;
    let stream = wait(
        file.OpenAsync(FileAccessMode::Read)
            .map_err(|e| e.to_string())?,
    )?;
    let decoder = wait(
        BitmapDecoder::CreateAsync(&stream).map_err(|e| e.to_string())?,
    )?;
    let bitmap = wait(
        decoder
            .GetSoftwareBitmapAsync()
            .map_err(|e| e.to_string())?,
    )?;
    let engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|e| format!("无法创建 OCR 引擎（请确认系统已安装 OCR 语言包）: {e}"))?;
    let result = wait(
        engine
            .RecognizeAsync(&bitmap)
            .map_err(|e| e.to_string())?,
    )?;
    let text = result
        .Text()
        .map_err(|e| e.to_string())?
        .to_string();
    Ok(normalize_ocr_text(&text))
}

#[cfg(not(windows))]
pub fn recognize_image(_path: &std::path::Path) -> Result<String, String> {
    Err("OCR 仅支持 Windows".to_string())
}

fn normalize_ocr_text(text: &str) -> String {
    let t = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();
    // Ignore trivial OCR noise
    if t.chars().count() < 2 {
        return String::new();
    }
    t
}

pub fn should_run_ocr(path: &std::path::Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if meta.len() < 8_192 {
        return false;
    }
    if let Ok((w, h)) = image::image_dimensions(path) {
        if w < 80 || h < 80 {
            return false;
        }
    }
    true
}

pub fn run_ocr_for_clip(
    db: &crate::db::Database,
    clip_id: i64,
    force: bool,
) -> Result<bool, String> {
    let path = db.clip_image_path(clip_id)?;
    let Some(path) = path else {
        return Ok(false);
    };
    if !force && !should_run_ocr(&path) {
        return Ok(false);
    }
    if db.clip_has_ocr(clip_id)? {
        return Ok(false);
    }
    let text = recognize_image(&path)?;
    if text.is_empty() {
        return Ok(false);
    }
    db.set_clip_ocr_text(clip_id, &text)?;
    Ok(true)
}

pub fn spawn_ocr_job(
    db: std::sync::Arc<crate::db::Database>,
    clip_id: i64,
    image_path: std::path::PathBuf,
) {
    if !db.get_settings().enable_image_ocr {
        return;
    }
    if !should_run_ocr(&image_path) {
        return;
    }
    std::thread::spawn(move || {
        if db.clip_has_ocr(clip_id).unwrap_or(false) {
            return;
        }
        match recognize_image(&image_path) {
            Ok(text) if !text.is_empty() => {
                let _ = db.set_clip_ocr_text(clip_id, &text);
            }
            Ok(_) => {}
            Err(err) => eprintln!("ocr error for clip {clip_id}: {err}"),
        }
    });
}
