// src-tauri/src/commands/clipboard_cmd.rs
use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::state::AppState;

#[tauri::command]
pub async fn copy_to_clipboard(
    app: tauri::AppHandle,
    session_token: String,
    text: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    app.clipboard()
        .write_text(text)
        .map_err(|_| "写入剪贴板失败".to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clear_clipboard(
    app: tauri::AppHandle,
    session_token: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    app.clipboard()
        .clear()
        .map_err(|_| "清空剪贴板失败".to_string())?;
    Ok(())
}
