use image::GenericImageView;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub ext: String,
    pub size: u64,
    pub modified: String,
}

fn format_modified_time(meta: &std::fs::Metadata) -> String {
    use std::time::UNIX_EPOCH;
    match meta.modified() {
        Ok(time) => {
            let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
            let secs = duration.as_secs();
            // Convert to local time using chrono-like manual calculation
            // Simple approach: format timestamp as ISO-like string
            let days_since_epoch = secs / 86400;
            let time_of_day = secs % 86400;
            let hours = time_of_day / 3600;
            let minutes = (time_of_day % 3600) / 60;
            let seconds = time_of_day % 60;

            // Calculate year/month/day from days since epoch (1970-01-01)
            let mut y = 1970i64;
            let mut remaining = days_since_epoch as i64;
            loop {
                let days_in_year = if is_leap(y) { 366 } else { 365 };
                if remaining < days_in_year { break; }
                remaining -= days_in_year;
                y += 1;
            }
            let month_days = if is_leap(y) { LEAP_MONTH_DAYS } else { MONTH_DAYS };
            let mut m = 0;
            for &md in month_days.iter() {
                if remaining < md { break; }
                remaining -= md;
                m += 1;
            }
            let d = remaining + 1;
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m + 1, d, hours, minutes, seconds)
        }
        Err(_) => String::new(),
    }
}

const MONTH_DAYS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const LEAP_MONTH_DAYS: [i64; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[derive(Debug, Serialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub fn read_dir(path: String) -> Result<Vec<FileEntry>, String> {
    let entries = fs::read_dir(&path).map_err(|e| format!("璇诲彇鐩綍澶辫触: {}", e))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("璇诲彇鏉＄洰澶辫触: {}", e))?;
        let meta = entry.metadata().map_err(|e| format!("璇诲彇鍏冩暟鎹け璐? {}", e))?;
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let ext = p
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let size = meta.len();
        files.push(FileEntry {
            name,
            path: p.to_string_lossy().to_string(),
            is_dir: meta.is_dir(),
            ext,
            size,
            modified: format_modified_time(&meta),
        });
    }
    files.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });
    Ok(files)
}

#[tauri::command]
pub fn read_file_binary(path: String) -> Result<Vec<u8>, String> {
    fs::read(&path).map_err(|e| format!("璇诲彇鏂囦欢澶辫触: {}", e))
}

#[tauri::command]
pub fn read_file_text(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("璇诲彇鏂囨湰鏂囦欢澶辫触: {}", e))
}

#[tauri::command]
pub fn write_file(path: String, data: Vec<u8>) -> Result<(), String> {
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("鍒涘缓鐩綍澶辫触: {}", e))?;
    }
    fs::write(&path, &data).map_err(|e| format!("鍐欏叆鏂囦欢澶辫触: {}", e))
}

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::FilePath;
    let (tx, rx) = std::sync::mpsc::channel::<Option<FilePath>>();
    app.dialog()
        .file()
        .pick_folder(move |path| {
            let _ = tx.send(path);
        });
    let result = rx.recv().map_err(|e| format!("閫夋嫨鏂囦欢澶瑰け璐? {}", e))?;
    match result {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_image_dimensions(path: String) -> Result<ImageDimensions, String> {
    let p = path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let img = image::open(&p).map_err(|e| format!("打开图片失败: {}", e))?;
        let (width, height) = img.dimensions();
        Ok::<_, String>(ImageDimensions { width, height })
    }).await.map_err(|e| format!("异步任务失败: {}", e))?;
    result
}

#[tauri::command]
pub async fn get_thumbnail(path: String, max_size: u32) -> Result<Vec<u8>, String> {
    let result = tokio::task::spawn_blocking(move || {
        let img = image::open(&path).map_err(|e| format!("打开图片失败: {}", e))?;
        let thumbnail = img.thumbnail(max_size, max_size);
        let mut buf = std::io::Cursor::new(Vec::new());
        thumbnail
            .write_to(&mut buf, image::ImageFormat::Jpeg)
            .map_err(|e| format!("生成缩略图失败: {}", e))?;
        Ok::<_, String>(buf.into_inner())
    }).await.map_err(|e| format!("异步任务失败: {}", e))?;
    result
}

#[tauri::command]
pub fn rotate_image(path: String, degrees: i32, output_path: Option<String>) -> Result<String, String> {
    let img = image::open(&path).map_err(|e| format!("鎵撳紑鍥剧墖澶辫触: {}", e))?;
    let rotated = match degrees.rem_euclid(360) {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => return Err(format!("涓嶆敮鎸佺殑瑙掑害: {}", degrees)),
    };
    let out = output_path.unwrap_or_else(|| path.clone());
    rotated
        .save(&out)
        .map_err(|e| format!("淇濆瓨鏃嬭浆缁撴灉澶辫触: {}", e))?;
    Ok(out)
}

#[tauri::command]
pub fn crop_image(
    path: String,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    output_path: Option<String>,
) -> Result<String, String> {
    let img = image::open(&path).map_err(|e| format!("鎵撳紑鍥剧墖澶辫触: {}", e))?;
    let cropped = img.crop_imm(x, y, w, h);
    let out = output_path.unwrap_or_else(|| path.clone());
    cropped
        .save(&out)
        .map_err(|e| format!("淇濆瓨瑁佸垏缁撴灉澶辫触: {}", e))?;
    Ok(out)
}

fn annotations_path(image_path: &str) -> PathBuf {
    let p = Path::new(image_path);
    let parent = p.parent().unwrap_or_else(|| Path::new("."));
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "unknown".into());
    parent.join(format!(".tafiles-annotations-{}.json", stem))
}

#[tauri::command]
pub fn load_annotations(path: String) -> Result<String, String> {
    let ap = annotations_path(&path);
    if ap.exists() {
        fs::read_to_string(&ap).map_err(|e| format!("璇诲彇鏍囨敞鏂囦欢澶辫触: {}", e))
    } else {
        Ok("[]".to_string())
    }
}

#[tauri::command]
pub fn save_annotations(path: String, data: String) -> Result<(), String> {
    let ap = annotations_path(&path);
    fs::write(&ap, &data).map_err(|e| format!("淇濆瓨鏍囨敞鏂囦欢澶辫触: {}", e))
}

#[tauri::command]
pub fn list_drives() -> Result<Vec<FileEntry>, String> {
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let path = format!("{}:\\", letter as char);
        if Path::new(&path).exists() {
            drives.push(FileEntry {
                name: format!("({}:)", letter as char),
                path,
                is_dir: true,
                ext: String::new(),
                size: 0,
                modified: String::new(),
            });
        }
    }
    Ok(drives)
}

#[tauri::command]
pub fn get_home_dir() -> Result<String, String> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|e| format!("鏃犳硶鑾峰彇涓荤洰褰? {}", e))
}

#[tauri::command]
pub fn list_dirs(path: String) -> Result<Vec<FileEntry>, String> {
    let entries = read_dir_list(&path)?;
    Ok(entries.into_iter().filter(|e| e.is_dir).collect())
}

fn read_dir_list(path: &str) -> Result<Vec<FileEntry>, String> {
    let entries = fs::read_dir(path).map_err(|e| format!("璇诲彇鐩綍澶辫触: {}", e))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("璇诲彇鏉＄洰澶辫触: {}", e))?;
        let meta = entry.metadata().map_err(|e| format!("璇诲彇鍏冩暟鎹け璐? {}", e))?;
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let ext = p
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let size = meta.len();
        files.push(FileEntry {
            name,
            path: p.to_string_lossy().to_string(),
            is_dir: meta.is_dir(),
            ext,
            size,
            modified: format_modified_time(&meta),
        });
    }
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(files)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn get_file_icon(ext: String, size: u32, file_path: Option<String>) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    use windows_sys::Win32::UI::Shell::*;
    use windows_sys::Win32::Storage::FileSystem::*;

    let ext_lower = ext.to_lowercase();

    // SHIL_SMALL=0(16), SHIL_LARGE=1(32), SHIL_EXTRALARGE=2(48), SHIL_JUMBO=4(256)
    let image_list_type: i32 = if size <= 16 { 0 }
        else if size <= 32 { 1 }
        else if size <= 48 { 2 }
        else { 4 };

    let (path_wide, attributes, use_file_attributes) = if let Some(ref fp) = file_path {
        if Path::new(fp).exists() {
            let p: Vec<u16> = fp.encode_utf16().chain(std::iter::once(0)).collect();
            (p, FILE_ATTRIBUTE_NORMAL, false)
        } else {
            return Err("文件路径不存在".to_string());
        }
    } else if ext_lower == "folder" {
        let p: Vec<u16> = "folder".encode_utf16().chain(std::iter::once(0)).collect();
        (p, FILE_ATTRIBUTE_DIRECTORY, true)
    } else {
        let dot_ext = if ext_lower.starts_with('.') { ext_lower } else { format!(".{}", ext_lower) };
        let filename = format!("file{}", dot_ext);
        let p: Vec<u16> = filename.encode_utf16().chain(std::iter::once(0)).collect();
        (p, FILE_ATTRIBUTE_NORMAL, true)
    };

    let mut uflags = SHGFI_SYSICONINDEX;
    if use_file_attributes {
        uflags |= SHGFI_USEFILEATTRIBUTES;
    }

    // Step 1: get icon index
    let mut shfi: SHFILEINFOW = unsafe { std::mem::zeroed() };
    let result = unsafe {
        SHGetFileInfoW(
            path_wide.as_ptr(),
            attributes,
            &mut shfi,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            uflags,
        )
    };
    if result == 0 {
        return Err("无法获取文件图标索引".to_string());
    }
    let icon_index = shfi.iIcon;

    // Step 2: get system image list
    const IID_IIMAGE_LIST: windows_sys::core::GUID = windows_sys::core::GUID {
        data1: 0x46EB5926,
        data2: 0x582E,
        data3: 0x4017,
        data4: [0x9F, 0xDF, 0xE8, 0x99, 0x8D, 0xAA, 0x09, 0x50],
    };
    let mut image_list: *mut core::ffi::c_void = std::ptr::null_mut();
    let hr = unsafe {
        SHGetImageList(image_list_type, &IID_IIMAGE_LIST, &mut image_list)
    };
    if hr < 0 || image_list.is_null() {
        return Err("SHGetImageList 失败".to_string());
    }

    // Step 3: call IImageList::GetIcon via COM vtable (offset 10 after IUnknown's 3)
    type GetIconFn = unsafe extern "system" fn(*mut core::ffi::c_void, i32, u32, *mut HICON) -> i32;
    let vtable = unsafe { *(image_list as *mut *mut *mut core::ffi::c_void) };
    let get_icon: GetIconFn = unsafe { std::mem::transmute(*vtable.offset(10)) };

    let mut icon: HICON = std::ptr::null_mut();
    let icon_hr = unsafe { get_icon(image_list, icon_index, 0, &mut icon) };
    if icon_hr < 0 || icon.is_null() {
        return Err("GetIcon 澶辫触".to_string());
    }

    let png = icon_to_png(icon, size);
    unsafe { DestroyIcon(icon) };
    png
}

#[cfg(target_os = "windows")]
fn icon_to_png(icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON, target_size: u32) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Graphics::Gdi::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    let dst_w = target_size as i32;
    let dst_h = target_size as i32;

    let hdc_screen = unsafe { GetDC(std::ptr::null_mut()) };
    let hdc = unsafe { CreateCompatibleDC(hdc_screen) };
    unsafe { ReleaseDC(std::ptr::null_mut(), hdc_screen) };

    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: dst_w,
            biHeight: -dst_h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        ..unsafe { std::mem::zeroed() }
    };

    let mut bits_ptr: *mut u8 = std::ptr::null_mut();
    let hbm = unsafe {
        CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits_ptr as *mut _ as *mut _, std::ptr::null_mut(), 0)
    };
    if hbm.is_null() {
        unsafe { DeleteDC(hdc) };
        return Err("CreateDIBSection 澶辫触".to_string());
    }

    let old_bmp = unsafe { SelectObject(hdc, hbm) };

    let draw_ok = unsafe { DrawIconEx(hdc, 0, 0, icon, dst_w, dst_h, 0, std::ptr::null_mut(), DI_NORMAL) };
    if draw_ok == 0 {
        unsafe {
            SelectObject(hdc, old_bmp);
            DeleteObject(hbm);
            DeleteDC(hdc);
        };
        return Err("DrawIconEx 澶辫触".to_string());
    }

    let pixel_count = (target_size * target_size) as usize;
    let mut pixels = unsafe { std::slice::from_raw_parts(bits_ptr, pixel_count * 4) }.to_vec();

    unsafe {
        SelectObject(hdc, old_bmp);
        DeleteObject(hbm);
        DeleteDC(hdc);
    };

    // Windows DIB stores pixels as BGRA, swap R and B for RGBA
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    let img = image::RgbaImage::from_raw(target_size, target_size, pixels)
        .ok_or("创建图像失败")?;

    let mut png_buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut png_buf, image::ImageFormat::Png)
        .map_err(|e| format!("编码 PNG 失败: {}", e))?;
    Ok(png_buf.into_inner())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn get_file_icon(_ext: String, _size: u32) -> Result<Vec<u8>, String> {
    Err("仅支持 Windows".to_string())
}

// ────────── WebDAV 命令 ──────────

#[tauri::command]
pub async fn webdav_connect(url: String, name: String, username: String, password: String) -> Result<crate::webdav::ConnectionInfo, String> {
    crate::webdav::connect(&url, &name, &username, &password)
}

#[tauri::command]
pub async fn webdav_disconnect(session_id: String) -> Result<(), String> {
    crate::webdav::disconnect(&session_id)
}

#[tauri::command]
pub async fn webdav_list_sessions() -> Result<Vec<crate::webdav::ConnectionInfo>, String> {
    crate::webdav::list_sessions()
}

#[tauri::command]
pub async fn webdav_list(session_id: String, path: String) -> Result<Vec<FileEntry>, String> {
    crate::webdav::list(&session_id, &path).await
}

#[tauri::command]
pub async fn webdav_read_binary(session_id: String, path: String) -> Result<Vec<u8>, String> {
    crate::webdav::read_binary(&session_id, &path).await
}

#[tauri::command]
pub async fn webdav_read_text(session_id: String, path: String) -> Result<String, String> {
    crate::webdav::read_text(&session_id, &path).await
}

#[tauri::command]
pub async fn webdav_download(session_id: String, path: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    crate::webdav::download(&session_id, &path, app_handle).await
}

#[tauri::command]
pub async fn webdav_upload(session_id: String, local_path: String, remote_path: String) -> Result<(), String> {
    crate::webdav::upload(&session_id, &local_path, &remote_path).await
}

#[tauri::command]
pub async fn webdav_create_dir(session_id: String, path: String) -> Result<(), String> {
    crate::webdav::create_dir(&session_id, &path).await
}

#[tauri::command]
pub async fn webdav_remove(session_id: String, path: String) -> Result<(), String> {
    crate::webdav::remove(&session_id, &path).await
}

#[tauri::command]
pub async fn webdav_rename(session_id: String, from: String, to: String) -> Result<(), String> {
    crate::webdav::rename(&session_id, &from, &to).await
}

#[tauri::command]
pub fn create_dir(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| format!("创建目录失败: {}", e))
}

#[tauri::command]
pub fn create_file(path: String) -> Result<(), String> {
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    fs::write(&path, []).map_err(|e| format!("创建文件失败: {}", e))
}

#[tauri::command]
pub fn remove(path: String) -> Result<(), String> {
    let meta = fs::metadata(&path).map_err(|e| format!("获取文件信息失败: {}", e))?;
    if meta.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| format!("删除目录失败: {}", e))
    } else {
        fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))
    }
}

#[tauri::command]
pub fn rename_item(from: String, to: String) -> Result<(), String> {
    if let Some(parent) = Path::new(&to).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目标目录失败: {}", e))?;
    }
    fs::rename(&from, &to).map_err(|e| format!("重命名失败: {}", e))
}

fn copy_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if src.is_dir() {
        fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
        for entry in fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let file_type = entry.file_type().map_err(|e| format!("读取文件类型失败: {}", e))?;
            let new_src = entry.path();
            let new_dst = dst.join(entry.file_name());
            if file_type.is_dir() {
                copy_recursive(&new_src, &new_dst)?;
            } else {
                fs::copy(&new_src, &new_dst).map_err(|e| format!("复制文件失败: {}", e))?;
            }
        }
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
        fs::copy(src, dst).map_err(|e| format!("复制文件失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn copy_item(from: String, to: String) -> Result<(), String> {
    let src = Path::new(&from);
    let dst = Path::new(&to);
    copy_recursive(src, dst)
}

#[tauri::command]
pub fn move_item(from: String, to: String) -> Result<(), String> {
    rename_item(from.clone(), to.clone()).or_else(|_| {
        copy_item(from.clone(), to.clone())?;
        let meta = fs::metadata(&from).map_err(|e| format!("获取文件信息失败: {}", e))?;
        if meta.is_dir() {
            fs::remove_dir_all(&from).map_err(|e| format!("删除源目录失败: {}", e))
        } else {
            fs::remove_file(&from).map_err(|e| format!("删除源文件失败: {}", e))
        }
    })
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    }
    Ok(())
}

fn get_zip_modified(mime_time: Option<zip::DateTime>) -> String {
    match mime_time {
        Some(dt) => {
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second())
        }
        None => String::new(),
    }
}

#[tauri::command]
pub fn zip_list(zip_path: String, inner_path: String) -> Result<Vec<FileEntry>, String> {
    let file = fs::File::open(&zip_path).map_err(|e| format!("无法打开 ZIP 文件: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("无法解析 ZIP 文件: {}", e))?;

    let normalized_inner = if inner_path.is_empty() || inner_path == "/" {
        String::new()
    } else {
        let s = inner_path.trim_start_matches('/').trim_end_matches('/');
        if s.is_empty() { String::new() } else { format!("{}/", s) }
    };

    let mut entries: std::collections::HashMap<String, (bool, u64, String)> = std::collections::HashMap::new();

    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let full_name = entry.name().to_string();
        // Skip entries not under current prefix
        if !full_name.starts_with(&normalized_inner) || full_name == normalized_inner {
            continue;
        }
        let relative = full_name[normalized_inner.len()..].to_string();
        if relative.is_empty() { continue; }

        // Check if this is a direct child or nested deeper
        let child_name = if let Some(slash_pos) = relative.find('/') {
            let dir_name = &relative[..slash_pos];
            let key = format!("{}{}/", normalized_inner, dir_name);
            entries.entry(key).or_insert_with(|| (true, 0, String::new()));
            continue;
        } else {
            relative
        };

        let key = format!("{}{}", normalized_inner, child_name);
        let is_dir = entry.is_dir();
        let size = entry.size();
        let modified = get_zip_modified(entry.last_modified());
        entries.entry(key).or_insert_with(|| (is_dir, size, modified));
    }

    let mut files: Vec<FileEntry> = entries.into_iter().map(|(path, (is_dir, size, modified))| {
        let name = path.trim_end_matches('/').rsplit('/').next().unwrap_or(&path).to_string();
        let ext = if is_dir {
            String::new()
        } else {
            name.rsplit('.').next().map(|e| e.to_lowercase()).unwrap_or_default()
        };
        FileEntry {
            name,
            path: format!("zip://{}/{}", zip_path, path),
            is_dir,
            ext,
            size,
            modified,
        }
    }).collect();

    files.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });
    Ok(files)
}

#[tauri::command]
pub fn zip_read_binary(zip_path: String, inner_path: String) -> Result<Vec<u8>, String> {
    let file = fs::File::open(&zip_path).map_err(|e| format!("无法打开 ZIP 文件: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("无法解析 ZIP 文件: {}", e))?;
    let inner = inner_path.trim_start_matches('/');
    let mut entry = archive.by_name(inner).map_err(|e| format!("ZIP 中未找到文件: {}", e))?;
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry.read_to_end(&mut buf).map_err(|e| format!("读取 ZIP 文件失败: {}", e))?;
    Ok(buf)
}

#[tauri::command]
pub fn zip_read_text(zip_path: String, inner_path: String) -> Result<String, String> {
    let bytes = zip_read_binary(zip_path, inner_path)?;
    String::from_utf8(bytes).map_err(|e| format!("文件不是有效文本: {}", e))
}

// ── Window controls ──

#[tauri::command]
pub fn window_minimize(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("主窗口未找到")?
        .minimize()
        .map_err(|e| format!("最小化失败: {}", e))
}

#[tauri::command]
pub fn window_toggle_maximize(app: tauri::AppHandle) -> Result<(), String> {
    let w = app.get_webview_window("main").ok_or("主窗口未找到")?;
    if w.is_maximized().unwrap_or(false) {
        w.unmaximize().map_err(|e| format!("还原失败: {}", e))
    } else {
        w.maximize().map_err(|e| format!("最大化失败: {}", e))
    }
}

#[tauri::command]
pub fn window_close(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("主窗口未找到")?
        .close()
        .map_err(|e| format!("关闭失败: {}", e))
}

#[tauri::command]
pub fn window_start_drag(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("主窗口未找到")?
        .start_dragging()
        .map_err(|e| format!("拖拽失败: {}", e))
}

#[tauri::command]
pub fn window_is_maximized(app: tauri::AppHandle) -> Result<bool, String> {
    Ok(app.get_webview_window("main")
        .ok_or("主窗口未找到")?
        .is_maximized()
        .map_err(|e| format!("读取状态失败: {}", e))?)
}

// ── Temp file ──

#[tauri::command]
pub fn save_temp_file(data: Vec<u8>, name: String) -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join("tafiles").join("extracted");
    fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;
    let file_path = temp_dir.join(&name);
    // Avoid overwriting: add timestamp if exists
    let file_path = if file_path.exists() {
        let stem = file_path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = file_path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        temp_dir.join(format!("{}_{}{}", stem, ts, ext))
    } else {
        file_path
    };
    fs::write(&file_path, &data).map_err(|e| format!("写入临时文件失败: {}", e))?;
    Ok(file_path.to_string_lossy().to_string())
}