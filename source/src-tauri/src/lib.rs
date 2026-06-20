mod commands;

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
            commands::open_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
