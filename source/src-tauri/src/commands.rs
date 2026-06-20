use image::GenericImageView;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub ext: String,
    pub size: u64,
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