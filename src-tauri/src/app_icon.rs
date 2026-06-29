use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct IconManifest {
    apps: HashMap<String, ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestEntry {
    exe: String,
    file: String,
}

static MANIFEST: LazyLock<Mutex<Option<IconManifest>>> = LazyLock::new(|| Mutex::new(None));

fn icons_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("app-icons")
}

fn manifest_path(data_dir: &Path) -> PathBuf {
    icons_dir(data_dir).join("manifest.json")
}

fn load_manifest(data_dir: &Path) -> IconManifest {
    let path = manifest_path(data_dir);
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(m) = serde_json::from_str(&text) {
            return m;
        }
    }
    IconManifest::default()
}

fn save_manifest(data_dir: &Path, manifest: &IconManifest) -> Result<(), String> {
    let dir = icons_dir(data_dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    std::fs::write(manifest_path(data_dir), text).map_err(|e| e.to_string())
}

fn icon_key(name: &str) -> String {
    let digest = Sha256::digest(name.trim().to_lowercase().as_bytes());
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

pub fn ensure_app_icon(data_dir: &Path, app_name: &str, exe_path: &Path) {
    if app_name.trim().is_empty() || !exe_path.exists() {
        return;
    }
    let data_dir = data_dir.to_path_buf();
    let app_name = app_name.trim().to_string();
    let exe_path = exe_path.to_path_buf();
    std::thread::spawn(move || {
        let _ = ensure_app_icon_sync(&data_dir, &app_name, &exe_path);
    });
}

fn ensure_app_icon_sync(data_dir: &Path, app_name: &str, exe_path: &Path) -> Result<(), String> {
    let mut manifest = load_manifest(data_dir);
    if let Some(entry) = manifest.apps.get(app_name) {
        if Path::new(&entry.exe) == exe_path {
            let icon_path = icons_dir(data_dir).join(&entry.file);
            if icon_path.exists() {
                return Ok(());
            }
        }
    }

    let file_name = format!("{}.png", icon_key(app_name));
    let out_path = icons_dir(data_dir).join(&file_name);
    std::fs::create_dir_all(icons_dir(data_dir)).map_err(|e| e.to_string())?;

    #[cfg(windows)]
    extract_exe_icon(exe_path, &out_path)?;

    #[cfg(not(windows))]
    return Ok(());

    if out_path.exists() {
        manifest.apps.insert(
            app_name.to_string(),
            ManifestEntry {
                exe: exe_path.to_string_lossy().into_owned(),
                file: file_name,
            },
        );
        save_manifest(data_dir, &manifest)?;
        *MANIFEST.lock() = None;
    }
    Ok(())
}

pub fn get_icon_data_url(data_dir: &Path, app_name: &str) -> Option<String> {
    let name = app_name.trim();
    if name.is_empty() {
        return None;
    }
    let manifest = {
        let mut cache = MANIFEST.lock();
        if cache.is_none() {
            *cache = Some(load_manifest(data_dir));
        }
        cache.clone()?
    };

    if let Some(url) = icon_for_key(data_dir, &manifest, name) {
        return Some(url);
    }

    let lower = name.to_lowercase();
    for (key, _) in &manifest.apps {
        if key.to_lowercase() == lower {
            return icon_for_key(data_dir, &manifest, key);
        }
    }

    for (key, _) in &manifest.apps {
        let kl = key.to_lowercase();
        if kl.contains(&lower) || lower.contains(&kl) {
            return icon_for_key(data_dir, &manifest, key);
        }
    }

    None
}

fn icon_for_key(data_dir: &Path, manifest: &IconManifest, name: &str) -> Option<String> {
    let entry = manifest.apps.get(name)?;
    let path = icons_dir(data_dir).join(&entry.file);
    if !path.exists() {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
    Some(format!("data:image/png;base64,{b64}"))
}

#[cfg(windows)]
fn extract_exe_icon(exe_path: &Path, out_path: &Path) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, DrawIconEx, GetIconInfo, DI_NORMAL};

    let wide: Vec<u16> = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut info = SHFILEINFOW::default();
    unsafe {
        SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            Default::default(),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
    }
    let hicon = info.hIcon;
    if hicon.is_invalid() {
        return Err("icon not found".into());
    }

    let result = (|| {
        unsafe {
            let mut icon_info = std::mem::zeroed();
            GetIconInfo(hicon, &mut icon_info).map_err(|e| e.to_string())?;

            let hdc = GetDC(HWND::default());
            let mem_dc = CreateCompatibleDC(hdc);
            let size = 32i32;
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: size,
                    biHeight: -size,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let hbmp = windows::Win32::Graphics::Gdi::CreateDIBSection(
                mem_dc,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0,
            )
            .map_err(|e| e.to_string())?;
            let old = SelectObject(mem_dc, hbmp);
            DrawIconEx(mem_dc, 0, 0, hicon, size, size, 0, None, DI_NORMAL)
                .map_err(|e| e.to_string())?;

            let mut buf = vec![0u8; (size * size * 4) as usize];
            let lines = GetDIBits(
                mem_dc,
                hbmp,
                0,
                size as u32,
                Some(buf.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );
            if lines == 0 {
                return Err("GetDIBits failed".into());
            }

            SelectObject(mem_dc, old);
            DeleteObject(hbmp);
            DeleteDC(mem_dc);
            ReleaseDC(HWND::default(), hdc);
            DeleteObject(icon_info.hbmColor);
            DeleteObject(icon_info.hbmMask);

            for px in buf.chunks_exact_mut(4) {
                px.swap(0, 2);
            }

            let img = image::RgbaImage::from_raw(size as u32, size as u32, buf)
                .ok_or_else(|| "invalid icon buffer".to_string())?;
            img.save(out_path).map_err(|e| e.to_string())?;
            Ok(())
        }
    })();

    unsafe {
        let _ = DestroyIcon(hicon);
    }
    result
}
