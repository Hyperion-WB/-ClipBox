use crate::models::AppSettings;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageEncoder, RgbaImage};
use std::path::{Path, PathBuf};

pub struct SavedImage {
    pub path: PathBuf,
    pub hash_sample: String,
}

pub fn save_clipboard_image(
    rgba: &[u8],
    width: usize,
    height: usize,
    images_dir: &Path,
    thumbs_dir: &Path,
    settings: &AppSettings,
) -> Result<(SavedImage, PathBuf), String> {
    let stamp = chrono::Utc::now().timestamp_millis();
    let raw_bytes = width.saturating_mul(height).saturating_mul(4);
    let min_bytes = (settings.image_compress_min_kb.max(64) as usize) * 1024;
    let should_compress = settings.compress_images
        && (raw_bytes >= min_bytes
            || width.max(height) > settings.image_max_dimension.max(480) as usize);

    let img = RgbaImage::from_raw(width as u32, height as u32, rgba.to_vec())
        .ok_or_else(|| "无效的图片尺寸".to_string())?;
    let mut dynamic = DynamicImage::ImageRgba8(img);

    if should_compress {
        let max_dim = settings.image_max_dimension.clamp(720, 4096) as u32;
        let (w, h) = dynamic.dimensions();
        if w.max(h) > max_dim {
            dynamic = dynamic.resize(max_dim, max_dim, FilterType::Triangle);
        }
    }

    let (path, hash_sample) = if should_compress {
        let filename = format!("{stamp}.jpg");
        let path = images_dir.join(&filename);
        let quality = settings.image_jpeg_quality.clamp(50, 95) as u8;
        let rgb = dynamic.to_rgb8();
        let mut buf = Vec::new();
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
        encoder
            .write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| e.to_string())?;
        std::fs::write(&path, &buf).map_err(|e| e.to_string())?;
        let sample = format!("jpg:{}:{}", path.display(), buf.len().min(64));
        (path, sample)
    } else {
        let filename = format!("{stamp}.png");
        let path = images_dir.join(&filename);
        let mut buf = Vec::new();
        let rgba8 = dynamic.to_rgba8();
        let encoder = image::codecs::png::PngEncoder::new(&mut buf);
        encoder
            .write_image(
                rgba8.as_raw(),
                rgba8.width(),
                rgba8.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| e.to_string())?;
        std::fs::write(&path, &buf).map_err(|e| e.to_string())?;
        let sample = format!("png:{}:{}", path.display(), buf.len().min(64));
        (path, sample)
    };

    let thumb_filename = format!("{stamp}_thumb.jpg");
    let thumb_path = thumbs_dir.join(&thumb_filename);
    save_thumbnail_file(&dynamic, &thumb_path)?;

    Ok((
        SavedImage {
            path,
            hash_sample,
        },
        thumb_path,
    ))
}

fn save_thumbnail_file(img: &DynamicImage, thumb_path: &Path) -> Result<(), String> {
    let thumb = img.resize(48, 48, FilterType::Triangle).to_rgb8();
    let mut buf = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 75);
    encoder
        .write_image(
            thumb.as_raw(),
            thumb.width(),
            thumb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| e.to_string())?;
    std::fs::write(thumb_path, buf).map_err(|e| e.to_string())?;
    Ok(())
}
