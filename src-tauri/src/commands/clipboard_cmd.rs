// src-tauri/src/commands/clipboard_cmd.rs
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn copy_to_clipboard(app: tauri::AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clear_clipboard(app: tauri::AppHandle) -> Result<(), String> {
    app.clipboard()
        .clear()
        .map_err(|e| e.to_string())?;
    Ok(())
}
