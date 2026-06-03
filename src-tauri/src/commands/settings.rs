// src-tauri/src/commands/settings.rs
use tauri::State;
use crate::db::queries;
use crate::state::AppState;

#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    queries::get_config(&state.db, &key)
        .await
        .map_err(|e| e.to_string())
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
    queries::set_config(&state.db, &key, &value)
        .await
        .map_err(|e| e.to_string())
}
