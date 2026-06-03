use tauri::State;

use crate::crypto::kdf;
use crate::db::queries;
use crate::state::AppState;

/// 审计日志写入（失败静默忽略，不阻塞主操作）
macro_rules! audit_log {
    ($db:expr, $action:expr, $entry_id:expr, $field_key:expr, $metadata:expr) => {
        let _ = queries::write_audit_log($db, $action, $entry_id, $field_key, $metadata).await;
    };
    ($db:expr, $action:expr) => {
        audit_log!($db, $action, None, None, None);
    };
}

#[tauri::command]
pub async fn is_initialized(state: State<'_, AppState>) -> Result<bool, String> {
    let result = queries::get_config(&state.db, "password_hash")
        .await
        .map_err(|e| e.to_string())?;
    Ok(result.is_some())
}

#[tauri::command]
pub async fn setup(password: String, state: State<'_, AppState>) -> Result<String, String> {
    use rand::RngCore;
    use zeroize::Zeroizing;

    // 检查是否已初始化
    let existing = queries::get_config(&state.db, "password_hash")
        .await
        .map_err(|e| e.to_string())?;
    if existing.is_some() {
        return Err("密码管理器已初始化，请勿重复设置".to_string());
    }

    let password_bytes = Zeroizing::new(password.into_bytes());

    // 1. 生成 KDF salt
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);

    // 2. 派生加密密钥
    let key = kdf::derive_key(&password_bytes, &salt).map_err(|e| e.to_string())?;

    // 3. 哈希主密码用于验证
    let hash = kdf::hash_master_password(&password_bytes).map_err(|e| e.to_string())?;

    // 4. 存入数据库
    queries::set_config(&state.db, "kdf_salt", &hex::encode(salt))
        .await
        .map_err(|e| e.to_string())?;
    queries::set_config(&state.db, "password_hash", &hash)
        .await
        .map_err(|e| e.to_string())?;

    // 5. 激活加密密钥
    *state.encryption_key.write().await = Some(key);
    *state.kdf_salt.write().await = Some(Zeroizing::new(salt));

    // 6. 创建会话
    let token = state.sessions.create().await;

    // 7. 审计日志
    audit_log!(&state.db, "setup");

    Ok(token)
}

#[tauri::command]
pub async fn unlock(password: String, state: State<'_, AppState>) -> Result<String, String> {
    use std::sync::atomic::Ordering;
    use zeroize::Zeroizing;

    // 检查暴力破解防护
    if state.is_unlock_locked() {
        let failures = state.unlock_failures.load(Ordering::SeqCst);
        return Err(format!(
            "密码错误次数过多（{}次），请5分钟后重试",
            failures
        ));
    }

    let password_bytes = Zeroizing::new(password.into_bytes());

    // 1. 获取存储的哈希
    let hash = queries::get_config(&state.db, "password_hash")
        .await
        .map_err(|e| e.to_string())?
        .ok_or("密码管理器未初始化")?;

    // 2. 验证密码
    let valid = kdf::verify_master_password(&password_bytes, &hash).map_err(|e| e.to_string())?;

    if !valid {
        state.record_unlock_failure();
        let failures = state.unlock_failures.load(Ordering::SeqCst);
        return Err(format!(
            "密码错误（已失败{}次，超过5次将锁定5分钟）",
            failures
        ));
    }

    // 密码验证成功，重置失败计数
    state.reset_unlock_failures();

    // 3. 获取 KDF salt
    let salt_hex = queries::get_config(&state.db, "kdf_salt")
        .await
        .map_err(|e| e.to_string())?
        .ok_or("KDF salt 缺失")?;

    let salt_bytes = hex::decode(&salt_hex).map_err(|e| e.to_string())?;
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&salt_bytes[..16]);

    // 4. 派生密钥
    let key = kdf::derive_key(&password_bytes, &salt).map_err(|e| e.to_string())?;

    // 5. 激活密钥
    *state.encryption_key.write().await = Some(key);
    *state.kdf_salt.write().await = Some(Zeroizing::new(salt));

    // 6. 创建会话
    let token = state.sessions.create().await;

    // 7. 审计日志
    audit_log!(&state.db, "unlock");

    Ok(token)
}

#[tauri::command]
pub async fn lock(state: State<'_, AppState>) -> Result<(), String> {
    // 审计日志（在锁定前写入，因为锁定后 db 可能不可用）
    audit_log!(&state.db, "lock");

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

    use zeroize::Zeroizing;

    let old_bytes = Zeroizing::new(old_password.into_bytes());
    let new_bytes = Zeroizing::new(new_password.into_bytes());

    // 验证旧密码
    let hash = queries::get_config(&state.db, "password_hash")
        .await
        .map_err(|e| e.to_string())?
        .ok_or("密码管理器未初始化")?;

    let valid = kdf::verify_master_password(&old_bytes, &hash).map_err(|e| e.to_string())?;
    if !valid {
        return Err("旧密码错误".to_string());
    }

    // 获取当前加密密钥
    let key_guard = state.encryption_key.read().await;
    let old_key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    // 生成新 salt 和密钥
    use rand::RngCore;
    let mut new_salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut new_salt);
    let new_key = kdf::derive_key(&new_bytes, &new_salt).map_err(|e| e.to_string())?;

    // 在事务中执行所有数据库操作
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    // 重新加密所有字段
    let fields = sqlx::query_as::<_, (String, String)>("SELECT id, enc_value FROM fields")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for (id, enc_value) in fields {
        let plaintext =
            crate::crypto::cipher::decrypt_field(old_key, &enc_value).map_err(|e| e.to_string())?;
        let new_enc =
            crate::crypto::cipher::encrypt_field(&new_key, &plaintext).map_err(|e| e.to_string())?;
        sqlx::query("UPDATE fields SET enc_value = ? WHERE id = ?")
            .bind(&new_enc)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 重新加密 field_history
    let history = sqlx::query_as::<_, (String, String)>("SELECT id, enc_value FROM field_history")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for (id, enc_value) in history {
        let plaintext =
            crate::crypto::cipher::decrypt_field(old_key, &enc_value).map_err(|e| e.to_string())?;
        let new_enc =
            crate::crypto::cipher::encrypt_field(&new_key, &plaintext).map_err(|e| e.to_string())?;
        sqlx::query("UPDATE field_history SET enc_value = ? WHERE id = ?")
            .bind(&new_enc)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 更新密码哈希和 salt
    let new_hash = kdf::hash_master_password(&new_bytes).map_err(|e| e.to_string())?;
    sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)")
        .bind("password_hash")
        .bind(&new_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)")
        .bind("kdf_salt")
        .bind(&hex::encode(new_salt))
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 提交事务
    tx.commit().await.map_err(|e| e.to_string())?;

    drop(key_guard);

    // 更新内存中的密钥
    *state.encryption_key.write().await = Some(new_key);
    *state.kdf_salt.write().await = Some(Zeroizing::new(new_salt));

    // 审计日志
    audit_log!(&state.db, "change_password");

    Ok(())
}
