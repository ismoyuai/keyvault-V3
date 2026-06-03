use serde::{Deserialize, Serialize};
use tauri::State;

use crate::crypto::cipher;
use crate::db::queries;
use crate::state::AppState;

#[derive(Serialize)]
pub struct EntryMeta {
    pub id: String,
    #[serde(rename = "entryType")]
    pub entry_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
    pub favorited: bool,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct EntrySecrets {
    pub fields: Vec<DecryptedField>,
}

#[derive(Serialize)]
pub struct DecryptedField {
    #[serde(rename = "fieldKey")]
    pub field_key: String,
    #[serde(rename = "fieldType")]
    pub field_type: String,
    pub value: String,
    #[serde(rename = "isSensitive")]
    pub is_sensitive: bool,
}

#[derive(Deserialize)]
pub struct CreateEntryInput {
    #[serde(rename = "entryType")]
    pub entry_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    pub fields: Vec<FieldInput>,
}

#[derive(Deserialize)]
pub struct FieldInput {
    #[serde(rename = "fieldKey")]
    pub field_key: String,
    #[serde(rename = "fieldType")]
    pub field_type: String,
    pub value: String,
    #[serde(rename = "isSensitive")]
    pub is_sensitive: bool,
}

fn parse_tags(tags_json: Option<String>) -> Vec<String> {
    tags_json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default()
}

fn row_to_meta(
    row: crate::db::schema::EntryMeta,
) -> EntryMeta {
    EntryMeta {
        id: row.id,
        entry_type: row.entry_type,
        title: row.title,
        subtitle: row.subtitle,
        tags: parse_tags(row.tags),
        favorited: row.favorited != 0,
        group_id: row.group_id,
        updated_at: row.updated_at,
    }
}

/// 获取所有条目元数据
#[tauri::command]
pub async fn list_entries(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntryMeta>, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let rows = queries::list_entries(&state.db, 100, 0)
        .await
        .map_err(|e| {
            tracing::error!("操作失败: {:?}", e);
            "操作失败，请重试".to_string()
        })?;

    Ok(rows.into_iter().map(row_to_meta).collect())
}

/// 搜索条目
#[tauri::command]
pub async fn search_entries(
    session_token: String,
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntryMeta>, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let rows = queries::search_entries(&state.db, &query)
        .await
        .map_err(|e| {
            tracing::error!("操作失败: {:?}", e);
            "操作失败，请重试".to_string()
        })?;

    Ok(rows.into_iter().map(row_to_meta).collect())
}

/// 按需获取单个条目的解密字段
#[tauri::command]
pub async fn get_entry_secrets(
    session_token: String,
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<EntrySecrets, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let rows = queries::get_entry_fields(&state.db, &entry_id)
        .await
        .map_err(|e| {
            tracing::error!("操作失败: {:?}", e);
            "操作失败，请重试".to_string()
        })?;

    let mut fields = Vec::new();
    for row in rows {
        let decrypted = cipher::decrypt_field(key, &row.enc_value).map_err(|e| {
            tracing::error!("操作失败: {:?}", e);
            "操作失败，请重试".to_string()
        })?;
        let value =
            String::from_utf8(decrypted.to_vec()).map_err(|_| "解码失败".to_string())?;
        fields.push(DecryptedField {
            field_key: row.field_key,
            field_type: row.field_type,
            value,
            is_sensitive: row.is_sensitive != 0,
        });
    }

    Ok(EntrySecrets { fields })
}

/// 创建条目
#[tauri::command]
pub async fn create_entry(
    session_token: String,
    input: CreateEntryInput,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let entry_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let tags_json = serde_json::to_string(&input.tags).unwrap_or_default();

    // 插入条目元数据
    sqlx::query(
        "INSERT INTO entries (id, group_id, entry_type, title, subtitle, tags, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&entry_id)
    .bind(&input.group_id)
    .bind(&input.entry_type)
    .bind(&input.title)
    .bind(&input.subtitle)
    .bind(&tags_json)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("操作失败: {:?}", e);
        "操作失败，请重试".to_string()
    })?;

    // 插入加密字段
    for (i, field) in input.fields.iter().enumerate() {
        let enc_value =
            cipher::encrypt_field(key, field.value.as_bytes()).map_err(|e| {
                tracing::error!("操作失败: {:?}", e);
                "操作失败，请重试".to_string()
            })?;
        let field_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO fields (id, entry_id, field_key, field_type, enc_value, is_sensitive, sort_order)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&field_id)
        .bind(&entry_id)
        .bind(&field.field_key)
        .bind(&field.field_type)
        .bind(&enc_value)
        .bind(field.is_sensitive as i32)
        .bind(i as i32)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("操作失败: {:?}", e);
            "操作失败，请重试".to_string()
        })?;
    }

    Ok(entry_id)
}

/// 更新条目
#[tauri::command]
pub async fn update_entry(
    session_token: String,
    entry_id: String,
    input: CreateEntryInput,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let now = chrono::Utc::now().timestamp();
    let tags_json = serde_json::to_string(&input.tags).unwrap_or_default();

    // 更新元数据
    sqlx::query(
        "UPDATE entries SET group_id = ?, entry_type = ?, title = ?, subtitle = ?, tags = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&input.group_id)
    .bind(&input.entry_type)
    .bind(&input.title)
    .bind(&input.subtitle)
    .bind(&tags_json)
    .bind(now)
    .bind(&entry_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("操作失败: {:?}", e);
        "操作失败，请重试".to_string()
    })?;

    // 先查询旧字段用于保存历史
    let old_fields = queries::get_entry_fields(&state.db, &entry_id)
        .await
        .unwrap_or_default();

    // 删除旧字段
    sqlx::query("DELETE FROM fields WHERE entry_id = ?")
        .bind(&entry_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("操作失败: {:?}", e);
            "操作失败，请重试".to_string()
        })?;

    // 插入新字段并保存历史
    for (i, field) in input.fields.iter().enumerate() {
        // 保存旧值到历史
        for old in &old_fields {
            if old.field_key == field.field_key {
                let history_id = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO field_history (id, entry_id, field_key, enc_value, changed_at)
                     VALUES (?, ?, ?, ?, ?)",
                )
                .bind(&history_id)
                .bind(&entry_id)
                .bind(&old.field_key)
                .bind(&old.enc_value)
                .bind(now)
                .execute(&state.db)
                .await
                .ok();
            }
        }

        let enc_value =
            cipher::encrypt_field(key, field.value.as_bytes()).map_err(|e| {
                tracing::error!("操作失败: {:?}", e);
                "操作失败，请重试".to_string()
            })?;
        let field_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO fields (id, entry_id, field_key, field_type, enc_value, is_sensitive, sort_order)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&field_id)
        .bind(&entry_id)
        .bind(&field.field_key)
        .bind(&field.field_type)
        .bind(&enc_value)
        .bind(field.is_sensitive as i32)
        .bind(i as i32)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("操作失败: {:?}", e);
            "操作失败，请重试".to_string()
        })?;
    }

    Ok(())
}

/// 删除条目
#[tauri::command]
pub async fn delete_entry(
    session_token: String,
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    sqlx::query("DELETE FROM entries WHERE id = ?")
        .bind(&entry_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("操作失败: {:?}", e);
            "操作失败，请重试".to_string()
        })?;

    Ok(())
}

/// 切换收藏状态
#[tauri::command]
pub async fn toggle_favorite(
    session_token: String,
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    sqlx::query("UPDATE entries SET favorited = CASE WHEN favorited = 1 THEN 0 ELSE 1 END WHERE id = ?")
        .bind(&entry_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("操作失败: {:?}", e);
            "操作失败，请重试".to_string()
        })?;

    Ok(())
}
