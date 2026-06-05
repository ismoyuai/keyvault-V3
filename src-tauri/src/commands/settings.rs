// src-tauri/src/commands/settings.rs
use tauri::State;

use crate::db::{self, queries};
use crate::state::AppState;

/// 未解锁也可读取的 UI 偏好项（不含凭据、哈希等）
const PUBLIC_SETTING_KEYS: &[&str] = &[
    "auto_lock_minutes",
    "clipboard_clear_seconds",
    "theme",
];

fn is_public_setting_key(key: &str) -> bool {
    PUBLIC_SETTING_KEYS.contains(&key)
}

#[tauri::command]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
    session_token: Option<String>,
) -> Result<Option<String>, String> {
    if is_public_setting_key(&key) {
        if let Ok(db) = state.db_pool().await {
            if let Ok(Some(value)) = queries::get_config(&db, &key).await {
                return Ok(Some(value));
            }
        }
        return Ok(db::read_preference(&state.data_dir, &key));
    }

    let token = session_token.ok_or("需要解锁后访问此设置")?;
    if !state.sessions.validate(&token).await {
        return Err("会话已过期".to_string());
    }

    let db = state.db_pool().await?;
    queries::get_config(&db, &key)
        .await
        .map_err(|_| "读取设置失败".to_string())
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    session_token: String,
    key: String,
    value: String,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    if is_public_setting_key(&key) {
        db::write_preference(&state.data_dir, &key, &value)
            .map_err(|_| "保存设置失败".to_string())?;
    }

    let db = state.db_pool().await?;
    queries::set_config(&db, &key, &value)
        .await
        .map_err(|_| "保存设置失败".to_string())
}
