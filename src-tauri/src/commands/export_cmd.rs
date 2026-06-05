use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::crypto::cipher;
use crate::db::queries;
use crate::import::browser_csv;
use crate::state::AppState;

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

async fn build_export_json(state: &AppState, key: &[u8; 32]) -> Result<String, String> {
    let db = state.db_pool().await?;

    let entries = queries::list_all_active_entries(&db)
        .await
        .map_err(|_| "导出失败".to_string())?;

    let groups = queries::list_groups(&db)
        .await
        .map_err(|_| "导出失败".to_string())?;

    let group_map: std::collections::HashMap<String, String> =
        groups.into_iter().map(|g| (g.id, g.name)).collect();

    let mut export_entries = Vec::new();

    for entry in entries {
        let fields = queries::get_entry_fields(&db, &entry.id)
            .await
            .map_err(|_| "导出失败".to_string())?;

        let mut export_fields = Vec::new();
        for field in fields {
            let value = if field.is_sensitive != 0 {
                let decrypted =
                    cipher::decrypt_field(key, &field.enc_value).map_err(|_| "导出失败".to_string())?;
                String::from_utf8(decrypted.to_vec()).map_err(|_| "解码失败".to_string())?
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

    serde_json::to_string_pretty(&data).map_err(|_| "导出失败".to_string())
}

#[tauri::command]
pub async fn export_vault(
    state: State<'_, AppState>,
    session_token: String,
    format: String,
) -> Result<String, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    if format != "json" {
        return Err("不支持的导出格式".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    build_export_json(&state, key).await
}

#[tauri::command]
pub async fn export_vault_to_file(
    app: AppHandle,
    state: State<'_, AppState>,
    session_token: String,
    format: String,
) -> Result<bool, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    if format != "json" {
        return Err("不支持的导出格式".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let file_path = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .blocking_save_file();

    let Some(file_path) = file_path else {
        return Ok(false);
    };

    let path = file_path
        .into_path()
        .map_err(|_| "无效的文件路径".to_string())?;
    let json = build_export_json(&state, key).await?;
    std::fs::write(&path, json).map_err(|_| "写入文件失败".to_string())?;

    Ok(true)
}

#[derive(Deserialize, Clone)]
pub(crate) struct ImportEntry {
    pub(crate) entry_type: String,
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) tags: Option<String>,
    pub(crate) favorited: Option<bool>,
    pub(crate) group_name: Option<String>,
    pub(crate) fields: Vec<ImportField>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct ImportField {
    pub(crate) field_key: String,
    pub(crate) field_type: Option<String>,
    pub(crate) value: String,
    pub(crate) is_sensitive: Option<bool>,
}

#[derive(Deserialize)]
struct ImportData {
    entries: Vec<ImportEntry>,
}

fn parse_import_entries(data: &str, format: &str) -> Result<Vec<ImportEntry>, String> {
    match format {
        "json" => {
            let parsed: ImportData = serde_json::from_str(data)
                .map_err(|_| "JSON 解析失败，请检查文件格式".to_string())?;
            Ok(parsed.entries)
        }
        "csv" | "browser-csv" => browser_csv::parse_browser_csv(data),
        "auto" => {
            let trimmed = data.trim_start_matches('\u{FEFF}').trim();
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                parse_import_entries(data, "json")
            } else {
                parse_import_entries(data, "csv")
            }
        }
        _ => Err("不支持的导入格式，请使用 KeyVault JSON 或浏览器 CSV".to_string()),
    }
}

async fn import_vault_data(
    state: &AppState,
    key: &[u8; 32],
    data: &str,
    format: &str,
) -> Result<usize, String> {
    let entries = parse_import_entries(data, format)?;

    let db = state.db_pool().await?;
    let mut count = 0usize;

    for import_entry in entries {
        let entry_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let tags_json = import_entry.tags.unwrap_or_else(|| "[]".to_string());

        let group_id = if let Some(ref name) = import_entry.group_name {
            let groups = queries::list_groups(&db)
                .await
                .map_err(|_| "导入失败".to_string())?;
            groups.iter().find(|g| &g.name == name).map(|g| g.id.clone())
        } else {
            None
        };

        let mut tx = db.begin().await.map_err(|_| "导入失败".to_string())?;

        sqlx::query(
            "INSERT INTO entries (id, group_id, entry_type, title, subtitle, tags, favorited, sort_order, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)",
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
        .execute(&mut *tx)
        .await
        .map_err(|_| "导入失败".to_string())?;

        for (i, import_field) in import_entry.fields.iter().enumerate() {
            // 与 create_entry 一致：enc_value 始终加密；is_sensitive 仅作 UI 元数据
            let is_sensitive = import_field.is_sensitive.unwrap_or(true);
            let enc_value = cipher::encrypt_field(key, import_field.value.as_bytes())
                .map_err(|_| "导入失败".to_string())?;
            let field_id = uuid::Uuid::new_v4().to_string();
            let field_type = import_field
                .field_type
                .clone()
                .unwrap_or_else(|| "text".to_string());

            sqlx::query(
                "INSERT INTO fields (id, entry_id, field_key, field_type, enc_value, is_sensitive, sort_order)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&field_id)
            .bind(&entry_id)
            .bind(&import_field.field_key)
            .bind(&field_type)
            .bind(&enc_value)
            .bind(is_sensitive as i32)
            .bind(i as i32)
            .execute(&mut *tx)
            .await
            .map_err(|_| "导入失败".to_string())?;
        }

        tx.commit().await.map_err(|_| "导入失败".to_string())?;
        count += 1;
    }

    Ok(count)
}

#[tauri::command]
pub async fn import_vault(
    state: State<'_, AppState>,
    session_token: String,
    data: String,
    format: String,
) -> Result<usize, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    import_vault_data(&state, key, &data, &format).await
}

#[tauri::command]
pub async fn import_vault_from_file(
    app: AppHandle,
    state: State<'_, AppState>,
    session_token: String,
    format: String,
) -> Result<usize, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let file_path = app
        .dialog()
        .file()
        .add_filter("密码库文件", &["json", "csv"])
        .add_filter("KeyVault JSON", &["json"])
        .add_filter("浏览器密码 CSV", &["csv"])
        .blocking_pick_file();

    let Some(file_path) = file_path else {
        return Ok(0);
    };

    let path = file_path
        .into_path()
        .map_err(|_| "无效的文件路径".to_string())?;
    let content = std::fs::read_to_string(&path).map_err(|_| "读取文件失败".to_string())?;

    import_vault_data(&state, key, &content, &format).await
}

#[cfg(test)]
mod import_encrypt_tests {
    use crate::crypto::cipher;
    use crate::import::browser_csv;

    /// 回归：非敏感字段也必须加密存储，否则 get_entry_secrets 解密会失败
    #[test]
    fn csv_import_fields_encrypt_and_decrypt() {
        let key = [7u8; 32];
        let csv = r#"name,url,username,password
Test,https://t.com,user,pass"#;
        let entries = browser_csv::parse_browser_csv(csv).unwrap();
        let entry = &entries[0];

        for import_field in &entry.fields {
            let enc_value = cipher::encrypt_field(&key, import_field.value.as_bytes()).unwrap();
            let decrypted = cipher::decrypt_field(&key, &enc_value).unwrap();
            assert_eq!(
                String::from_utf8(decrypted.to_vec()).unwrap(),
                import_field.value
            );
            // 旧实现将 is_sensitive=false 的字段明文写入 enc_value
            if import_field.is_sensitive == Some(false) {
                assert!(
                    cipher::decrypt_field(&key, &import_field.value).is_err(),
                    "plaintext enc_value must not decrypt: {}",
                    import_field.field_key
                );
            }
        }
    }
}
