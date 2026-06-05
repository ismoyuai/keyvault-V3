mod commands;
mod crypto;
mod db;
mod error;
mod import;
mod native_messaging;
mod state;
mod sync;
mod tray;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            let data_dir = app_handle
                .path()
                .app_data_dir()
                .expect("无法获取应用数据目录");

            std::fs::create_dir_all(&data_dir).expect("无法创建数据目录");

            let app_state = AppState::new(data_dir);
            app_handle.manage(app_state);

            tray::setup_tray(app).expect("无法初始化系统托盘");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::is_initialized,
            commands::auth::get_unlock_status,
            commands::auth::setup,
            commands::auth::unlock,
            commands::auth::lock,
            commands::auth::change_password,
            commands::auth::emergency_wipe,
            commands::export_cmd::export_vault,
            commands::export_cmd::import_vault,
            commands::export_cmd::export_vault_to_file,
            commands::export_cmd::import_vault_from_file,
            commands::vault::list_entries,
            commands::vault::search_entries,
            commands::vault::get_entry_secrets,
            commands::vault::create_entry,
            commands::vault::update_entry,
            commands::vault::delete_entry,
            commands::vault::list_trash_entries,
            commands::vault::restore_entry,
            commands::vault::purge_entry,
            commands::vault::empty_trash,
            commands::vault::toggle_favorite,
            commands::generator::generate_password,
            commands::breach::check_password_breach,
            commands::clipboard_cmd::copy_to_clipboard,
            commands::clipboard_cmd::clear_clipboard,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::sync::get_sync_config,
            commands::sync::set_sync_config,
            commands::sync::test_webdav_connection,
            commands::sync::sync_push,
            commands::sync::sync_pull,
            commands::sync::get_sync_status,
            commands::groups::list_groups,
            commands::groups::create_group,
            commands::groups::update_group,
            commands::groups::delete_group,
            commands::window::minimize_window,
            commands::window::toggle_maximize,
            commands::window::close_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_native_messaging() {
    native_messaging::host::run();
}
