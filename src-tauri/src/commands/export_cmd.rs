// src-tauri/src/commands/export_cmd.rs
use tauri::State;
use serde::{Serialize, Deserialize};
use crate::state::AppState;
use crate::crypto::cipher;
use crate::db::queries;

#[derive(Serialize)]
struct ExportEntry {
    entry_type: String,
    title: String,
    subtitle: Option<String>,
    tags: Option<String>,
    favorited: bool,
    group_name: Option<String>,
    fields: Vec<ExportField>,
}

#[derive(Serialize)]
struct ExportField {
    field_key: String,
    field_type: String,
    value: String,
    is_sensitive: bool,
}

#[derive(Serialize)]
struct ExportData {
    version: String,
    exported_at: i64,
    entries: Vec<ExportEntry>,
}

#[tauri::command]
pub async fn export_vault(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<String, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let entries = queries::list_entries(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let groups = queries::list_groups(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let group_map: std::collections::HashMap<String, String> = groups
        .into_iter()
        .map(|g| (g.id, g.name))
        .collect();

    let mut export_entries = Vec::new();

    for entry in entries {
        let fields = queries::get_entry_fields(&state.db, &entry.id)
            .await
            .map_err(|e| e.to_string())?;

        let mut export_fields = Vec::new();
        for field in fields {
            let value = if field.is_sensitive != 0 {
                let decrypted = cipher::decrypt_field(key, &field.enc_value)
                    .map_err(|e| e.to_string())?;
                String::from_utf8(decrypted.to_vec())
                    .map_err(|_| "解码失败".to_string())?
            } else {
                field.enc_value
            };
            export_fields.push(ExportField {
                field_key: field.field_key,
                field_type: field.field_type,
                value,
                is_sensitive: field.is_sensitive != 0,
            });
        }

        export_entries.push(ExportEntry {
            entry_type: entry.entry_type,
            title: entry.title,
            subtitle: entry.subtitle,
            tags: entry.tags,
            favorited: entry.favorited != 0,
            group_name: entry.group_id.and_then(|gid| group_map.get(&gid).cloned()),
            fields: export_fields,
        });
    }

    let data = ExportData {
        version: "3.0.0".to_string(),
        exported_at: chrono::Utc::now().timestamp(),
        entries: export_entries,
    };

    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct ImportEntry {
    entry_type: String,
    title: String,
    subtitle: Option<String>,
    tags: Option<String>,
    favorited: Option<bool>,
    group_name: Option<String>,
    fields: Vec<ImportField>,
}

#[derive(Deserialize)]
struct ImportField {
    field_key: String,
    field_type: Option<String>,
    value: String,
    is_sensitive: Option<bool>,
}

#[derive(Deserialize)]
struct ImportData {
    entries: Vec<ImportEntry>,
}

#[tauri::command]
pub async fn import_vault(
    state: State<'_, AppState>,
    session_token: String,
    json: String,
) -> Result<usize, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let data: ImportData = serde_json::from_str(&json)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    let mut count = 0;

    for import_entry in data.entries {
        let entry_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let tags_json = import_entry.tags.unwrap_or_else(|| "[]".to_string());

        let group_id = if let Some(ref name) = import_entry.group_name {
            let groups = queries::list_groups(&state.db)
                .await
                .map_err(|e| e.to_string())?;
            groups.iter().find(|g| &g.name == name).map(|g| g.id.clone())
        } else {
            None
        };

        sqlx::query(
            "INSERT INTO entries (id, group_id, entry_type, title, subtitle, tags, favorited, sort_order, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)"
        )
        .bind(&entry_id)
        .bind(&group_id)
        .bind(&import_entry.entry_type)
        .bind(&import_entry.title)
        .bind(&import_entry.subtitle)
        .bind(&tags_json)
        .bind(import_entry.favorited.unwrap_or(false) as i32)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        for (i, import_field) in import_entry.fields.iter().enumerate() {
            let is_sensitive = import_field.is_sensitive.unwrap_or(true);
            let enc_value = if is_sensitive {
                cipher::encrypt_field(key, import_field.value.as_bytes())
                    .map_err(|e| e.to_string())?
            } else {
                import_field.value.clone()
            };
            let field_id = uuid::Uuid::new_v4().to_string();
            let field_type = import_field.field_type.clone().unwrap_or_else(|| "text".to_string());

            sqlx::query(
                "INSERT INTO fields (id, entry_id, field_key, field_type, enc_value, is_sensitive, sort_order)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&field_id)
            .bind(&entry_id)
            .bind(&import_field.field_key)
            .bind(&field_type)
            .bind(&enc_value)
            .bind(is_sensitive as i32)
            .bind(i as i32)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
        }

        count += 1;
    }

    Ok(count)
}
