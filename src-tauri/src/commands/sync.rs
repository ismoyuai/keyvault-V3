use serde::{Deserialize, Serialize};
use tauri::State;

use crate::crypto::cipher;
use crate::db::queries;
use crate::error::{ipc_crypto_err, ipc_db_err, ipc_sync_err};
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
    let ciphertext =
        cipher::encrypt_field(key, plaintext.as_bytes()).map_err(ipc_crypto_err)?;
    Ok(format!("{ENC_CREDENTIAL_PREFIX}{ciphertext}"))
}

fn decrypt_sync_credential(key: &[u8; 32], stored: &str) -> Result<String, String> {
    let ciphertext = stored
        .strip_prefix(ENC_CREDENTIAL_PREFIX)
        .ok_or_else(|| "检测到旧版明文 WebDAV 凭据，已标记需迁移".to_string())?;

    let decrypted = cipher::decrypt_field(key, ciphertext).map_err(ipc_crypto_err)?;
    String::from_utf8(decrypted.to_vec())
        .map_err(|_| "WebDAV 凭据解密失败，请重新保存同步配置".to_string())
}

async fn load_webdav_config(state: &AppState, key: &[u8; 32]) -> Result<WebDavConfig, String> {
    let db = state.db_pool().await?;
    let url = queries::get_config(&db, KEY_URL)
        .await
        .map_err(ipc_db_err)?
        .filter(|s| !s.is_empty())
        .ok_or("请先配置 WebDAV 同步")?;
    let username = queries::get_config(&db, KEY_USERNAME)
        .await
        .map_err(ipc_db_err)?
        .unwrap_or_default();
    let stored_password = queries::get_config(&db, KEY_PASSWORD)
        .await
        .map_err(ipc_db_err)?
        .ok_or("请先配置 WebDAV 同步")?;

    // I-21：迁移旧版明文凭据为 `enc:` 格式；生产不再长期兼容明文回退
    let password = if stored_password.starts_with(ENC_CREDENTIAL_PREFIX) {
        decrypt_sync_credential(key, &stored_password)?
    } else {
        let encrypted = encrypt_sync_credential(key, &stored_password)?;
        queries::set_config(&db, KEY_PASSWORD, &encrypted)
            .await
            .map_err(ipc_db_err)?;
        stored_password
    };

    Ok(WebDavConfig {
        url,
        username,
        password,
    })
}

async fn get_or_create_device_id(state: &AppState) -> Result<String, String> {
    let db = state.db_pool().await?;
    if let Some(id) = queries::get_config(&db, KEY_DEVICE_ID)
        .await
        .map_err(ipc_db_err)?
    {
        return Ok(id);
    }
    let id = uuid::Uuid::new_v4().to_string();
    queries::set_config(&db, KEY_DEVICE_ID, &id)
        .await
        .map_err(ipc_db_err)?;
    Ok(id)
}

async fn export_sync_entries(state: &AppState, key: &[u8; 32]) -> Result<Vec<SyncEntry>, String> {
    let db = state.db_pool().await?;
    let entries = queries::list_all_entries_for_sync(&db)
        .await
        .map_err(ipc_db_err)?;
    let groups = queries::list_groups(&db)
        .await
        .map_err(ipc_db_err)?;
    let group_map: std::collections::HashMap<String, String> =
        groups.into_iter().map(|g| (g.id, g.name)).collect();

    let mut result = Vec::new();
    for entry in entries {
        let fields = queries::get_entry_fields(&db, &entry.id)
            .await
            .map_err(ipc_db_err)?;
        let mut sync_fields = Vec::new();
        for field in fields {
            let value = if field.is_sensitive != 0 {
                let decrypted =
                    cipher::decrypt_field(key, &field.enc_value).map_err(ipc_crypto_err)?;
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
    let db = state.db_pool().await?;
    if entry.fields.is_empty() && entry.deleted_at.is_none() {
        return Ok(());
    }

    let group_id = if let Some(ref name) = entry.group_name {
        let groups = queries::list_groups(&db)
            .await
            .map_err(ipc_db_err)?;
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
            .execute(&db)
            .await
            .map_err(ipc_db_err)?;
            Some(gid)
        }
    } else {
        None
    };

    let tags = entry.tags.clone().unwrap_or_else(|| "[]".to_string());
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entries WHERE id = ?")
        .bind(&entry.id)
        .fetch_one(&db)
        .await
        .map_err(ipc_db_err)?;

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
        .execute(&db)
        .await
        .map_err(ipc_db_err)?;

        if !entry.fields.is_empty() {
            sqlx::query("DELETE FROM fields WHERE entry_id = ?")
                .bind(&entry.id)
                .execute(&db)
                .await
                .map_err(ipc_db_err)?;
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
        .execute(&db)
        .await
        .map_err(ipc_db_err)?;
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
        .execute(&db)
        .await
        .map_err(ipc_db_err)?;
    }

    for (i, field) in entry.fields.iter().enumerate() {
        let enc_value = if field.is_sensitive {
            cipher::encrypt_field(key, field.value.as_bytes()).map_err(ipc_crypto_err)?
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
        .execute(&db)
        .await
        .map_err(ipc_db_err)?;
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

    let db = state.db_pool().await?;
    let url = queries::get_config(&db, KEY_URL)
        .await
        .map_err(ipc_db_err)?
        .unwrap_or_default();
    let username = queries::get_config(&db, KEY_USERNAME)
        .await
        .map_err(ipc_db_err)?
        .unwrap_or_default();
    let password = queries::get_config(&db, KEY_PASSWORD)
        .await
        .map_err(ipc_db_err)?;
    let last_sync = queries::get_config(&db, KEY_LAST_SYNC)
        .await
        .map_err(ipc_db_err)?
        .and_then(|s| s.parse().ok());
    let device_id = queries::get_config(&db, KEY_DEVICE_ID)
        .await
        .map_err(ipc_db_err)?;

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

    let db = state.db_pool().await?;
    queries::set_config(&db, KEY_URL, input.url.trim())
        .await
        .map_err(ipc_db_err)?;
    queries::set_config(&db, KEY_USERNAME, &input.username)
        .await
        .map_err(ipc_db_err)?;
    if !input.password.is_empty() {
        let key_guard = state.encryption_key.read().await;
        let key = key_guard.as_ref().ok_or("请先解锁密码管理器")?;
        let encrypted = encrypt_sync_credential(key, &input.password)?;
        queries::set_config(&db, KEY_PASSWORD, &encrypted)
            .await
            .map_err(ipc_db_err)?;
    }

    let _ = get_or_create_device_id(&state).await?;
    audit_log!(&db, "sync_config");
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
                ipc_sync_err("远程同步文件解密失败", e);
                "无法解密远程同步文件，已中止上传以防覆盖远程数据。若刚修改过主密码，请在所有设备用新密码重新推送。".to_string()
            })?;
            engine::merge_entries(local_entries.clone(), remote_payload.data.entries)
        }
        Ok(None) => local_entries.clone(),
        Err(e) => return Err(e),
    };

    let payload = engine::build_payload(&device_id, merged_entries)?;
    let encrypted = engine::serialize_encrypted(key, &payload)?;

    webdav::upload(&config, REMOTE_PATH, &encrypted).await?;

    let db = state.db_pool().await?;
    let now = chrono::Utc::now().timestamp();
    queries::set_config(&db, KEY_LAST_SYNC, &now.to_string())
        .await
        .map_err(ipc_db_err)?;

    audit_log!(&db, "sync_push");

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

    let db = state.db_pool().await?;
    let mut applied = 0usize;
    for remote_entry in &payload.data.entries {
        let local_updated: Option<i64> = sqlx::query_scalar("SELECT updated_at FROM entries WHERE id = ?")
            .bind(&remote_entry.id)
            .fetch_optional(&db)
            .await
            .map_err(ipc_db_err)?;

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
    queries::set_config(&db, KEY_LAST_SYNC, &now.to_string())
        .await
        .map_err(ipc_db_err)?;

    audit_log!(&db, "sync_pull");

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
    let db = state.db_pool().await?;
    let last_local = queries::get_config(&db, KEY_LAST_SYNC)
        .await
        .map_err(ipc_db_err)?
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

    #[tokio::test]
    async fn sync_credential_legacy_plaintext_migrates_to_enc() {
        use crate::db;

        let key = [0x11u8; 32];

        // 构造一个已迁移的临时数据库
        let dir = std::env::temp_dir().join(format!(
            "kv-sync-test-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        let db_path = db::db_file_path(&dir);
        let pool = db::open_encrypted_pool(&db_path, &key).await.unwrap();
        db::run_migrations(&pool).await.unwrap();

        // 注入到 AppState 以供 load_webdav_config 读取
        let state = AppState::new(dir);
        *state.db.write().await = Some(pool.clone());

        // 写入旧版明文配置
        queries::set_config(&pool, KEY_URL, "https://example.com").await.unwrap();
        queries::set_config(&pool, KEY_USERNAME, "u").await.unwrap();
        queries::set_config(&pool, KEY_PASSWORD, "legacy-plain").await.unwrap();

        let config = load_webdav_config(&state, &key).await.unwrap();
        assert_eq!(config.password, "legacy-plain");

        // 再次读取：应该已经迁移为 enc: 前缀
        let stored = queries::get_config(&pool, KEY_PASSWORD).await.unwrap().unwrap();
        assert!(stored.starts_with(ENC_CREDENTIAL_PREFIX));
        let decrypted = decrypt_sync_credential(&key, &stored).unwrap();
        assert_eq!(decrypted, "legacy-plain");
    }
}
