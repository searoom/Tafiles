mod commands;
mod webdav;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::read_dir,
            commands::read_file_binary,
            commands::read_file_text,
            commands::write_file,
            commands::pick_folder,
            commands::get_image_dimensions,
            commands::get_thumbnail,
            commands::rotate_image,
            commands::crop_image,
            commands::load_annotations,
            commands::save_annotations,
            commands::list_drives,
            commands::get_home_dir,
            commands::list_dirs,
            commands::get_file_icon,
            commands::create_dir,
            commands::create_file,
            commands::remove,
            commands::rename_item,
            commands::copy_item,
            commands::move_item,
            commands::open_file,
            commands::webdav_connect,
            commands::webdav_disconnect,
            commands::webdav_list_sessions,
            commands::webdav_list,
            commands::webdav_read_binary,
            commands::webdav_read_text,
            commands::webdav_download,
            commands::webdav_upload,
            commands::webdav_create_dir,
            commands::webdav_remove,
            commands::webdav_rename,
            commands::zip_list,
            commands::zip_read_binary,
            commands::zip_read_text,
            commands::save_temp_file,
            commands::window_minimize,
            commands::window_toggle_maximize,
            commands::window_close,
            commands::window_start_drag,
            commands::window_is_maximized,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
