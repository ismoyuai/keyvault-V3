use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::State;
use zeroize::Zeroizing;

use crate::crypto::kdf;
use crate::db::{self, queries};
use crate::error::{ipc_crypto_err, ipc_db_err};
use crate::state::AppState;

macro_rules! audit_log {
    ($db:expr, $action:expr, $entry_id:expr, $field_key:expr, $metadata:expr) => {
        let _ = queries::write_audit_log($db, $action, $entry_id, $field_key, $metadata).await;
    };
    ($db:expr, $action:expr) => {
        audit_log!($db, $action, None, None, None);
    };
}

#[derive(Serialize)]
pub struct UnlockStatus {
    #[serde(rename = "failureCount")]
    pub failure_count: u32,
    pub locked: bool,
    #[serde(rename = "secondsRemaining")]
    pub seconds_remaining: u64,
}

#[tauri::command]
pub async fn get_unlock_status(state: State<'_, AppState>) -> Result<UnlockStatus, String> {
    let failures = state.unlock_failures.load(Ordering::SeqCst);
    let locked = state.is_unlock_locked();
    let seconds_remaining = if locked {
        let last = state.last_failure_time.load(Ordering::SeqCst);
        let elapsed = chrono::Utc::now().timestamp() - last;
        (300 - elapsed).max(0) as u64
    } else {
        0
    };
    Ok(UnlockStatus {
        failure_count: failures,
        locked,
        seconds_remaining,
    })
}

#[tauri::command]
pub async fn is_initialized(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(db::is_vault_initialized(&state.data_dir))
}

#[tauri::command]
pub async fn setup(password: String, state: State<'_, AppState>) -> Result<String, String> {
    if db::is_vault_initialized(&state.data_dir) {
        return Err("密码管理器已初始化，请勿重复设置".to_string());
    }

    let password_bytes = Zeroizing::new(password.into_bytes());

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let key = kdf::derive_key(&password_bytes, &salt).map_err(ipc_crypto_err)?;
    let hash = kdf::hash_master_password(&password_bytes).map_err(ipc_crypto_err)?;

    state.open_database(key, salt).await?;

    let db = state.db_pool().await?;
    queries::set_config(&db, "kdf_salt", &hex::encode(salt))
        .await
        .map_err(ipc_db_err)?;
    queries::set_config(&db, "password_hash", &hash)
        .await
        .map_err(ipc_db_err)?;

    audit_log!(&db, "setup");

    Ok(state.sessions.create().await)
}

#[tauri::command]
pub async fn unlock(password: String, state: State<'_, AppState>) -> Result<String, String> {
    if state.is_unlock_locked() {
        let failures = state.unlock_failures.load(Ordering::SeqCst);
        return Err(format!(
            "密码错误次数过多（{}次），请5分钟后重试",
            failures
        ));
    }

    if !db::is_vault_initialized(&state.data_dir) {
        return Err("密码管理器未初始化".to_string());
    }

    let password_bytes = Zeroizing::new(password.into_bytes());
    let db_path = db::db_file_path(&state.data_dir);

    let legacy_pool = if db::read_sidecar_salt(&state.data_dir)
        .ok()
        .flatten()
        .is_none()
        && db_path.exists()
    {
        db::open_plaintext_pool(&db_path).await.ok()
    } else {
        None
    };

    let salt = db::resolve_kdf_salt(&state.data_dir, legacy_pool.as_ref())
        .await?
        .ok_or("KDF salt 缺失")?;

    if let Some(pool) = legacy_pool.as_ref() {
        db::close_pool(pool.clone()).await;
    }

    let field_key = kdf::derive_key(&password_bytes, &salt).map_err(ipc_crypto_err)?;

    if state.open_database(field_key, salt).await.is_err() {
        state.lock().await;
        state.record_unlock_failure();
        let failures = state.unlock_failures.load(Ordering::SeqCst);
        return Err(format!(
            "密码错误（已失败{}次，超过5次将锁定5分钟）",
            failures
        ));
    }

    let db = state.db_pool().await?;
    let hash = queries::get_config(&db, "password_hash")
        .await
        .map_err(ipc_db_err)?
        .ok_or("密码管理器未初始化")?;

    let valid = kdf::verify_master_password(&password_bytes, &hash).map_err(ipc_crypto_err)?;
    if !valid {
        state.lock().await;
        state.record_unlock_failure();
        let failures = state.unlock_failures.load(Ordering::SeqCst);
        return Err(format!(
            "密码错误（已失败{}次，超过5次将锁定5分钟）",
            failures
        ));
    }

    state.reset_unlock_failures();
    audit_log!(&db, "unlock");

    Ok(state.sessions.create().await)
}

#[tauri::command]
pub async fn lock(session_token: String, state: State<'_, AppState>) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    if let Ok(db) = state.db_pool().await {
        audit_log!(&db, "lock");
    }

    state.lock().await;
    Ok(())
}

#[tauri::command]
pub async fn change_password(
    session_token: String,
    old_password: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let old_bytes = Zeroizing::new(old_password.into_bytes());
    let new_bytes = Zeroizing::new(new_password.into_bytes());

    let db = state.db_pool().await?;
    let hash = queries::get_config(&db, "password_hash")
        .await
        .map_err(ipc_db_err)?
        .ok_or("密码管理器未初始化")?;

    let valid = kdf::verify_master_password(&old_bytes, &hash).map_err(ipc_crypto_err)?;
    if !valid {
        return Err("旧密码错误".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let old_key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let mut new_salt = [0u8; 16];
    OsRng.fill_bytes(&mut new_salt);
    let new_key = kdf::derive_key(&new_bytes, &new_salt).map_err(ipc_crypto_err)?;

    let mut tx = db.begin().await.map_err(ipc_db_err)?;

    let fields = sqlx::query_as::<_, (String, String)>("SELECT id, enc_value FROM fields")
        .fetch_all(&mut *tx)
        .await
        .map_err(ipc_db_err)?;

    for (id, enc_value) in fields {
        let plaintext =
            crate::crypto::cipher::decrypt_field(old_key, &enc_value).map_err(ipc_crypto_err)?;
        let new_enc = crate::crypto::cipher::encrypt_field(&new_key, &plaintext)
            .map_err(ipc_crypto_err)?;
        sqlx::query("UPDATE fields SET enc_value = ? WHERE id = ?")
            .bind(&new_enc)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(ipc_db_err)?;
    }

    let history = sqlx::query_as::<_, (String, String)>("SELECT id, enc_value FROM field_history")
        .fetch_all(&mut *tx)
        .await
        .map_err(ipc_db_err)?;

    for (id, enc_value) in history {
        let plaintext =
            crate::crypto::cipher::decrypt_field(old_key, &enc_value).map_err(ipc_crypto_err)?;
        let new_enc = crate::crypto::cipher::encrypt_field(&new_key, &plaintext)
            .map_err(ipc_crypto_err)?;
        sqlx::query("UPDATE field_history SET enc_value = ? WHERE id = ?")
            .bind(&new_enc)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(ipc_db_err)?;
    }

    let new_hash = kdf::hash_master_password(&new_bytes).map_err(ipc_crypto_err)?;
    sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)")
        .bind("password_hash")
        .bind(&new_hash)
        .execute(&mut *tx)
        .await
        .map_err(ipc_db_err)?;
    sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)")
        .bind("kdf_salt")
        .bind(&hex::encode(new_salt))
        .execute(&mut *tx)
        .await
        .map_err(ipc_db_err)?;

    tx.commit().await.map_err(ipc_db_err)?;
    drop(key_guard);

    *state.encryption_key.write().await = Some(new_key);
    *state.kdf_salt.write().await = Some(Zeroizing::new(new_salt));
    db::write_sidecar_salt(&state.data_dir, &new_salt).map_err(|_| "保存 salt 失败".to_string())?;

    let new_key_guard = state.encryption_key.read().await;
    let new_field_key = new_key_guard.as_ref().ok_or("密码管理器已锁定")?;
    state.rekey_database(new_field_key).await?;

    audit_log!(&db, "change_password");

    Ok(())
}

#[tauri::command]
pub async fn emergency_wipe(
    session_token: String,
    password: String,
    confirmation: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if confirmation != "DELETE" {
        return Err("请输入 DELETE 以确认擦除".to_string());
    }

    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let password_bytes = Zeroizing::new(password.into_bytes());
    let db = state.db_pool().await?;

    let hash = queries::get_config(&db, "password_hash")
        .await
        .map_err(ipc_db_err)?
        .ok_or("密码管理器未初始化")?;

    let valid = kdf::verify_master_password(&password_bytes, &hash).map_err(ipc_crypto_err)?;
    if !valid {
        return Err("主密码错误".to_string());
    }

    audit_log!(&db, "emergency_wipe");

    let tables = [
        "field_history",
        "fields",
        "entries",
        "groups",
        "audit_log",
        "breach_cache",
        "config",
    ];
    for table in tables {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&db)
            .await
            .map_err(ipc_db_err)?;
    }

    state.lock().await;

    let db_path = db::db_file_path(&state.data_dir);
    let salt_path = db::salt_file_path(&state.data_dir);
    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(salt_path);

    Ok(())
}
