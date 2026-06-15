mod commands;
mod tray;

use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".envtools")
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_groups,
            commands::get_group_detail,
            commands::create_group,
            commands::delete_group,
            commands::enable_group,
            commands::disable_group,
            commands::set_variable,
            commands::remove_variable,
            commands::get_status,
            commands::export_config,
            commands::import_config,
        ])
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
