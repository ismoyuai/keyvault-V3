use serde::{Deserialize, Serialize};
use tauri::State;

use crate::crypto::cipher;
use crate::db::queries;
use crate::state::AppState;
use crate::sync::{
    engine::{self, SyncEntry, SyncField, REMOTE_PATH},
    webdav::{self, WebDavConfig},
};

const KEY_URL: &str = "sync_webdav_url";
const KEY_USERNAME: &str = "sync_webdav_username";
const KEY_PASSWORD: &str = "sync_webdav_password";
const KEY_DEVICE_ID: &str = "sync_device_id";
const KEY_LAST_SYNC: &str = "sync_last_sync_time";
const ENC_CREDENTIAL_PREFIX: &str = "enc:";

macro_rules! audit_log {
    ($db:expr, $action:expr) => {
        let _ = queries::write_audit_log($db, $action, None, None, None).await;
    };
}

#[derive(Serialize)]
pub struct SyncConfig {
    pub url: String,
    pub username: String,
    pub configured: bool,
    #[serde(rename = "lastSyncTime")]
    pub last_sync_time: Option<i64>,
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
}

#[derive(Deserialize)]
pub struct SetSyncConfigInput {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SyncPushResult {
    #[serde(rename = "entriesSent")]
    pub entries_sent: usize,
    #[serde(rename = "exportedAt")]
    pub exported_at: String,
}

#[derive(Serialize)]
pub struct SyncPullResult {
    #[serde(rename = "entriesMerged")]
    pub entries_merged: usize,
    #[serde(rename = "remoteTime")]
    pub remote_time: Option<String>,
}

#[derive(Serialize)]
pub struct SyncStatus {
    #[serde(rename = "lastRemoteSync")]
    pub last_remote_sync: Option<String>,
    #[serde(rename = "lastLocalSync")]
    pub last_local_sync: Option<i64>,
}

fn encrypt_sync_credential(key: &[u8; 32], plaintext: &str) -> Result<String, String> {
    let ciphertext = cipher::encrypt_field(key, plaintext.as_bytes()).map_err(|e| e.to_string())?;
    Ok(format!("{ENC_CREDENTIAL_PREFIX}{ciphertext}"))
}

fn decrypt_sync_credential(key: &[u8; 32], stored: &str) -> Result<String, String> {
    if let Some(ciphertext) = stored.strip_prefix(ENC_CREDENTIAL_PREFIX) {
        let decrypted = cipher::decrypt_field(key, ciphertext).map_err(|e| e.to_string())?;
        String::from_utf8(decrypted.to_vec())
            .map_err(|_| "WebDAV 凭据解密失败，请重新保存同步配置".to_string())
    } else {
        // 兼容旧版明文存储
        Ok(stored.to_string())
    }
}

async fn load_webdav_config(state: &AppState, key: &[u8; 32]) -> Result<WebDavConfig, String> {
    let url = queries::get_config(&state.db, KEY_URL)
        .await
        .map_err(|e| e.to_string())?
        .filter(|s| !s.is_empty())
        .ok_or("请先配置 WebDAV 同步")?;
    let username = queries::get_config(&state.db, KEY_USERNAME)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let stored_password = queries::get_config(&state.db, KEY_PASSWORD)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("请先配置 WebDAV 同步")?;
    let password = decrypt_sync_credential(key, &stored_password)?;
    Ok(WebDavConfig {
        url,
        username,
        password,
    })
}

async fn get_or_create_device_id(state: &AppState) -> Result<String, String> {
    if let Some(id) = queries::get_config(&state.db, KEY_DEVICE_ID)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(id);
    }
    let id = uuid::Uuid::new_v4().to_string();
    queries::set_config(&state.db, KEY_DEVICE_ID, &id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(id)
}

async fn export_sync_entries(state: &AppState, key: &[u8; 32]) -> Result<Vec<SyncEntry>, String> {
    let entries = queries::list_all_entries_for_sync(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let groups = queries::list_groups(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let group_map: std::collections::HashMap<String, String> =
        groups.into_iter().map(|g| (g.id, g.name)).collect();

    let mut result = Vec::new();
    for entry in entries {
        let fields = queries::get_entry_fields(&state.db, &entry.id)
            .await
            .map_err(|e| e.to_string())?;
        let mut sync_fields = Vec::new();
        for field in fields {
            let value = if field.is_sensitive != 0 {
                let decrypted =
                    cipher::decrypt_field(key, &field.enc_value).map_err(|e| e.to_string())?;
                String::from_utf8(decrypted.to_vec()).map_err(|_| "解码失败".to_string())?
            } else {
                field.enc_value
            };
            sync_fields.push(SyncField {
                field_key: field.field_key,
                field_type: field.field_type,
                value,
                is_sensitive: field.is_sensitive != 0,
            });
        }
        result.push(SyncEntry {
            id: entry.id,
            entry_type: entry.entry_type,
            title: entry.title,
            subtitle: entry.subtitle,
            tags: entry.tags,
            favorited: entry.favorited != 0,
            group_name: entry.group_id.and_then(|gid| group_map.get(&gid).cloned()),
            updated_at: entry.updated_at,
            deleted_at: entry.deleted_at,
            fields: sync_fields,
        });
    }
    Ok(result)
}

async fn upsert_sync_entry(
    state: &AppState,
    key: &[u8; 32],
    entry: &SyncEntry,
) -> Result<(), String> {
    if entry.fields.is_empty() && entry.deleted_at.is_none() {
        return Ok(());
    }

    let group_id = if let Some(ref name) = entry.group_name {
        let groups = queries::list_groups(&state.db)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(g) = groups.iter().find(|g| &g.name == name) {
            Some(g.id.clone())
        } else {
            let gid = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().timestamp();
            sqlx::query(
                "INSERT INTO groups (id, name, sort_order, created_at, updated_at) VALUES (?, ?, 0, ?, ?)",
            )
            .bind(&gid)
            .bind(name)
            .bind(now)
            .bind(now)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
            Some(gid)
        }
    } else {
        None
    };

    let tags = entry.tags.clone().unwrap_or_else(|| "[]".to_string());
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entries WHERE id = ?")
        .bind(&entry.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    if exists > 0 {
        sqlx::query(
            "UPDATE entries SET group_id = ?, entry_type = ?, title = ?, subtitle = ?, tags = ?, favorited = ?, updated_at = ?, deleted_at = ? WHERE id = ?",
        )
        .bind(&group_id)
        .bind(&entry.entry_type)
        .bind(&entry.title)
        .bind(&entry.subtitle)
        .bind(&tags)
        .bind(entry.favorited as i32)
        .bind(entry.updated_at)
        .bind(entry.deleted_at)
        .bind(&entry.id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        if !entry.fields.is_empty() {
            sqlx::query("DELETE FROM fields WHERE entry_id = ?")
                .bind(&entry.id)
                .execute(&state.db)
                .await
                .map_err(|e| e.to_string())?;
        }
    } else if entry.deleted_at.is_none() {
        let now = entry.updated_at;
        sqlx::query(
            "INSERT INTO entries (id, group_id, entry_type, title, subtitle, tags, favorited, sort_order, created_at, updated_at, deleted_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?, NULL)",
        )
        .bind(&entry.id)
        .bind(&group_id)
        .bind(&entry.entry_type)
        .bind(&entry.title)
        .bind(&entry.subtitle)
        .bind(&tags)
        .bind(entry.favorited as i32)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        let now = entry.updated_at;
        sqlx::query(
            "INSERT INTO entries (id, group_id, entry_type, title, subtitle, tags, favorited, sort_order, created_at, updated_at, deleted_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?)",
        )
        .bind(&entry.id)
        .bind(&group_id)
        .bind(&entry.entry_type)
        .bind(&entry.title)
        .bind(&entry.subtitle)
        .bind(&tags)
        .bind(entry.favorited as i32)
        .bind(now)
        .bind(now)
        .bind(entry.deleted_at)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    }

    for (i, field) in entry.fields.iter().enumerate() {
        let enc_value = if field.is_sensitive {
            cipher::encrypt_field(key, field.value.as_bytes()).map_err(|e| e.to_string())?
        } else {
            field.value.clone()
        };
        let field_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO fields (id, entry_id, field_key, field_type, enc_value, is_sensitive, sort_order)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&field_id)
        .bind(&entry.id)
        .bind(&field.field_key)
        .bind(&field.field_type)
        .bind(&enc_value)
        .bind(field.is_sensitive as i32)
        .bind(i as i32)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_sync_config(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<SyncConfig, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let url = queries::get_config(&state.db, KEY_URL)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let username = queries::get_config(&state.db, KEY_USERNAME)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let password = queries::get_config(&state.db, KEY_PASSWORD)
        .await
        .map_err(|e| e.to_string())?;
    let last_sync = queries::get_config(&state.db, KEY_LAST_SYNC)
        .await
        .map_err(|e| e.to_string())?
        .and_then(|s| s.parse().ok());
    let device_id = queries::get_config(&state.db, KEY_DEVICE_ID)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SyncConfig {
        configured: !url.is_empty() && password.is_some(),
        url,
        username,
        last_sync_time: last_sync,
        device_id,
    })
}

#[tauri::command]
pub async fn set_sync_config(
    session_token: String,
    input: SetSyncConfigInput,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    queries::set_config(&state.db, KEY_URL, input.url.trim())
        .await
        .map_err(|e| e.to_string())?;
    queries::set_config(&state.db, KEY_USERNAME, &input.username)
        .await
        .map_err(|e| e.to_string())?;
    if !input.password.is_empty() {
        let key_guard = state.encryption_key.read().await;
        let key = key_guard.as_ref().ok_or("请先解锁密码管理器")?;
        let encrypted = encrypt_sync_credential(key, &input.password)?;
        queries::set_config(&state.db, KEY_PASSWORD, &encrypted)
            .await
            .map_err(|e| e.to_string())?;
    }

    let _ = get_or_create_device_id(&state).await?;
    audit_log!(&state.db, "sync_config");
    Ok(())
}

#[tauri::command]
pub async fn test_webdav_connection(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("请先解锁密码管理器")?;
    let config = load_webdav_config(&state, key).await?;
    webdav::test_connection(&config).await
}

#[tauri::command]
pub async fn sync_push(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<SyncPushResult, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let config = load_webdav_config(&state, key).await?;
    let device_id = get_or_create_device_id(&state).await?;

    let local_entries = export_sync_entries(&state, key).await?;

    let merged_entries = match webdav::download(&config, REMOTE_PATH).await {
        Ok(Some(remote_content)) => {
            let remote_payload = engine::deserialize_payload(key, &remote_content).map_err(|e| {
                format!(
                    "无法解密远程同步文件，已中止上传以防覆盖远程数据。若刚修改过主密码，请在所有设备用新密码重新推送。详情: {}",
                    e
                )
            })?;
            engine::merge_entries(local_entries.clone(), remote_payload.data.entries)
        }
        Ok(None) => local_entries.clone(),
        Err(e) => return Err(e),
    };

    let payload = engine::build_payload(&device_id, merged_entries)?;
    let encrypted = engine::serialize_encrypted(key, &payload)?;

    webdav::upload(&config, REMOTE_PATH, &encrypted).await?;

    let now = chrono::Utc::now().timestamp();
    queries::set_config(&state.db, KEY_LAST_SYNC, &now.to_string())
        .await
        .map_err(|e| e.to_string())?;

    audit_log!(&state.db, "sync_push");

    Ok(SyncPushResult {
        entries_sent: payload.data.entries.len(),
        exported_at: payload.exported_at,
    })
}

#[tauri::command]
pub async fn sync_pull(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<SyncPullResult, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let config = load_webdav_config(&state, key).await?;
    let content = webdav::download(&config, REMOTE_PATH).await?;

    let Some(content) = content else {
        return Ok(SyncPullResult {
            entries_merged: 0,
            remote_time: None,
        });
    };

    let payload = engine::deserialize_payload(key, &content)?;

    let mut applied = 0usize;
    for remote_entry in &payload.data.entries {
        let local_updated: Option<i64> = sqlx::query_scalar("SELECT updated_at FROM entries WHERE id = ?")
            .bind(&remote_entry.id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| e.to_string())?;

        // 与 merge_entries 一致：updated_at 相等时保留本地
        let should_apply = match local_updated {
            None => true,
            Some(local_ts) => remote_entry.updated_at > local_ts,
        };

        if should_apply {
            upsert_sync_entry(&state, key, remote_entry).await?;
            applied += 1;
        }
    }

    let now = chrono::Utc::now().timestamp();
    queries::set_config(&state.db, KEY_LAST_SYNC, &now.to_string())
        .await
        .map_err(|e| e.to_string())?;

    audit_log!(&state.db, "sync_pull");

    Ok(SyncPullResult {
        entries_merged: applied,
        remote_time: Some(payload.exported_at),
    })
}

#[tauri::command]
pub async fn get_sync_status(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<SyncStatus, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("请先解锁密码管理器")?;
    let config = load_webdav_config(&state, key).await?;
    let last_remote = webdav::get_last_modified(&config, REMOTE_PATH).await.ok().flatten();
    let last_local = queries::get_config(&state.db, KEY_LAST_SYNC)
        .await
        .map_err(|e| e.to_string())?
        .and_then(|s| s.parse().ok());

    Ok(SyncStatus {
        last_remote_sync: last_remote,
        last_local_sync: last_local,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_credential_roundtrip() {
        let key = [0x11u8; 32];
        let encrypted = encrypt_sync_credential(&key, "nas-secret").unwrap();
        assert!(encrypted.starts_with(ENC_CREDENTIAL_PREFIX));
        let decrypted = decrypt_sync_credential(&key, &encrypted).unwrap();
        assert_eq!(decrypted, "nas-secret");
    }

    #[test]
    fn sync_credential_legacy_plaintext() {
        let key = [0x11u8; 32];
        let decrypted = decrypt_sync_credential(&key, "legacy-plain").unwrap();
        assert_eq!(decrypted, "legacy-plain");
    }
}
