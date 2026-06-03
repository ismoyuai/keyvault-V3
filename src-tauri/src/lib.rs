mod commands;
mod crypto;
mod db;
mod error;
mod native_messaging;
mod state;

use state::AppState;
use tauri::Manager;
use tokio::sync::RwLock;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 获取应用数据目录
            let data_dir = app_handle
                .path()
                .app_data_dir()
                .expect("无法获取应用数据目录");

            // 确保目录存在
            std::fs::create_dir_all(&data_dir).expect("无法创建数据目录");

            let db_path = data_dir.join("keyvault.db");
            let db_path_str = format!("sqlite:{}?mode=rwc", db_path.display());

            // 在异步上下文中初始化数据库
            let rt = tokio::runtime::Handle::current();
            let pool = rt.block_on(async {
                db::init_db(&db_path_str)
                    .await
                    .expect("无法初始化数据库")
            });

            // 创建 AppState
            let app_state = AppState {
                encryption_key: RwLock::new(None),
                db: pool,
                sessions: crypto::session::SessionManager::new(),
                kdf_salt: RwLock::new(None),
            };

            app_handle.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::is_initialized,
            commands::auth::setup,
            commands::auth::unlock,
            commands::auth::lock,
            commands::auth::change_password,
            commands::vault::list_entries,
            commands::vault::search_entries,
            commands::vault::get_entry_secrets,
            commands::vault::create_entry,
            commands::vault::update_entry,
            commands::vault::delete_entry,
            commands::vault::toggle_favorite,
            commands::generator::generate_password,
            commands::breach::check_password_breach,
            commands::clipboard_cmd::copy_to_clipboard,
            commands::clipboard_cmd::clear_clipboard,
            commands::settings::get_setting,
            commands::settings::set_setting,
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
