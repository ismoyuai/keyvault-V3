pub mod queries;
pub mod schema;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
const SALT_FILENAME: &str = "kdf_salt.hex";
const DB_FILENAME: &str = "keyvault.db";

pub fn db_file_path(data_dir: &Path) -> PathBuf {
    data_dir.join(DB_FILENAME)
}

pub fn salt_file_path(data_dir: &Path) -> PathBuf {
    data_dir.join(SALT_FILENAME)
}

/// 金库是否已创建（数据库文件存在且非空）
pub fn is_vault_initialized(data_dir: &Path) -> bool {
    let path = db_file_path(data_dir);
    path.exists()
        && std::fs::metadata(&path)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
}

pub fn read_sidecar_salt(data_dir: &Path) -> Result<Option<[u8; 16]>, std::io::Error> {
    let path = salt_file_path(data_dir);
    if !path.exists() {
        return Ok(None);
    }
    let hex_str = std::fs::read_to_string(path)?.trim().to_string();
    let bytes = hex::decode(hex_str).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;
    if bytes.len() != 16 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "salt 长度无效",
        ));
    }
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&bytes);
    Ok(Some(salt))
}

pub fn write_sidecar_salt(data_dir: &Path, salt: &[u8; 16]) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(salt_file_path(data_dir), hex::encode(salt))?;
    Ok(())
}

fn db_url(path: &Path) -> String {
    format!("sqlite:{}?mode=rwc", path.display())
}

async fn apply_cipher_key(conn: &mut sqlx::SqliteConnection, db_key: &[u8; 32]) -> Result<(), sqlx::Error> {
    let key_hex = hex::encode(db_key);
    sqlx::query(&format!("PRAGMA key = \"x'{}'\"", key_hex))
        .execute(conn)
        .await?;
    Ok(())
}

/// 打开 SQLCipher 加密数据库
pub async fn open_encrypted_pool(
    db_path: &Path,
    db_key: &[u8; 32],
) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let url = db_url(db_path);
    let key = *db_key;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(move |conn, _| {
            let key = key;
            Box::pin(async move {
                apply_cipher_key(conn, &key).await?;
                Ok(())
            })
        })
        .connect(&url)
        .await?;

    sqlx::query("SELECT count(*) FROM sqlite_master")
        .fetch_one(&pool)
        .await?;

    Ok(pool)
}

/// 打开旧版明文 SQLite（仅用于迁移）
pub async fn open_plaintext_pool(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let url = db_url(db_path);
    let options = SqliteConnectOptions::from_str(&url)?.create_if_missing(false);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
}

/// 将明文数据库 rekey 为 SQLCipher
pub async fn rekey_pool(pool: &SqlitePool, db_key: &[u8; 32]) -> Result<(), sqlx::Error> {
    let key_hex = hex::encode(db_key);
    sqlx::query(&format!("PRAGMA rekey = \"x'{}'\"", key_hex))
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("migrations/001_init.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/002_history.sql"))
        .execute(pool)
        .await?;

    let has_deleted_at: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('entries') WHERE name = 'deleted_at'",
    )
    .fetch_one(pool)
    .await?;

    if has_deleted_at == 0 {
        sqlx::query("ALTER TABLE entries ADD COLUMN deleted_at INTEGER")
            .execute(pool)
            .await?;
    }

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_entries_deleted_at ON entries(deleted_at)")
        .execute(pool)
        .await?;

    Ok(())
}

/// 从明文 sidecar 或旧库 config 表读取 KDF salt
pub async fn resolve_kdf_salt(
    data_dir: &Path,
    legacy_pool: Option<&SqlitePool>,
) -> Result<Option<[u8; 16]>, String> {
    if let Some(salt) = read_sidecar_salt(data_dir).map_err(|_| "读取 salt 失败".to_string())? {
        return Ok(Some(salt));
    }

    let Some(pool) = legacy_pool else {
        return Ok(None);
    };

    let salt_hex = queries::get_config(pool, "kdf_salt")
        .await
        .map_err(|_| "读取 salt 失败".to_string())?;

    let Some(salt_hex) = salt_hex else {
        return Ok(None);
    };

    let bytes = hex::decode(&salt_hex).map_err(|_| "salt 格式无效".to_string())?;
    if bytes.len() != 16 {
        return Err("salt 长度无效".to_string());
    }
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&bytes[..16]);
    Ok(Some(salt))
}

/// 关闭连接池
pub async fn close_pool(pool: SqlitePool) {
    pool.close().await;
}

fn preferences_path(data_dir: &Path) -> PathBuf {
    data_dir.join("preferences.json")
}

fn load_preferences(data_dir: &Path) -> HashMap<String, String> {
    let path = preferences_path(data_dir);
    if !path.exists() {
        return HashMap::new();
    }
    let content = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_preferences(data_dir: &Path, prefs: &HashMap<String, String>) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(
        preferences_path(data_dir),
        serde_json::to_string(prefs).unwrap_or_else(|_| "{}".to_string()),
    )?;
    Ok(())
}

/// 未解锁时可读的 UI 偏好（非敏感）
pub fn read_preference(data_dir: &Path, key: &str) -> Option<String> {
    load_preferences(data_dir).get(key).cloned()
}

pub fn write_preference(data_dir: &Path, key: &str, value: &str) -> Result<(), std::io::Error> {
    let mut prefs = load_preferences(data_dir);
    prefs.insert(key.to_string(), value.to_string());
    save_preferences(data_dir, &prefs)
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_vault_initialized_false_when_missing() {
        let dir = std::env::temp_dir().join(format!("kv-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!is_vault_initialized(&dir));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_is_vault_initialized_true_when_db_nonempty() {
        let dir = std::env::temp_dir().join(format!("kv-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = db_file_path(&dir);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"sqlite3").unwrap();
        assert!(is_vault_initialized(&dir));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_is_vault_initialized_false_when_db_empty() {
        let dir = std::env::temp_dir().join(format!("kv-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::File::create(db_file_path(&dir)).unwrap();
        assert!(!is_vault_initialized(&dir));
        std::fs::remove_dir_all(&dir).ok();
    }
}
