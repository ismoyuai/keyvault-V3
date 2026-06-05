use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};

use sqlx::SqlitePool;
use tokio::sync::RwLock;
use zeroize::Zeroizing;

use crate::crypto::kdf;
use crate::crypto::session::SessionManager;
use crate::db;

pub struct AppState {
    pub data_dir: PathBuf,
    /// 加密密钥，仅在解锁后存在，锁定时清零
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
    /// SQLCipher 连接池，仅在解锁后存在
    pub db: RwLock<Option<SqlitePool>>,
    pub sessions: SessionManager,
    pub kdf_salt: RwLock<Option<Zeroizing<[u8; 16]>>>,
    pub unlock_failures: AtomicU32,
    pub last_failure_time: AtomicI64,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            encryption_key: RwLock::new(None),
            db: RwLock::new(None),
            sessions: SessionManager::new(),
            kdf_salt: RwLock::new(None),
            unlock_failures: AtomicU32::new(0),
            last_failure_time: AtomicI64::new(0),
        }
    }

    pub async fn db_pool(&self) -> Result<SqlitePool, String> {
        self.db
            .read()
            .await
            .clone()
            .ok_or_else(|| "密码管理器已锁定".to_string())
    }

    /// 打开或迁移数据库（setup / unlock 成功后调用）
    pub async fn open_database(
        &self,
        field_key: Zeroizing<[u8; 32]>,
        salt: [u8; 16],
    ) -> Result<(), String> {
        let db_path = db::db_file_path(&self.data_dir);
        let db_key = kdf::derive_db_key(&field_key).map_err(|_| "密钥派生失败".to_string())?;

        if let Ok(pool) = db::open_encrypted_pool(&db_path, &db_key).await {
            db::run_migrations(&pool)
                .await
                .map_err(|_| "数据库初始化失败".to_string())?;
            *self.db.write().await = Some(pool);
        } else if db_path.exists() {
            let legacy = db::open_plaintext_pool(&db_path)
                .await
                .map_err(|_| "无法打开数据库".to_string())?;
            db::run_migrations(&legacy)
                .await
                .map_err(|_| "数据库迁移失败".to_string())?;
            db::rekey_pool(&legacy, &db_key)
                .await
                .map_err(|_| "数据库加密迁移失败".to_string())?;
            db::close_pool(legacy).await;

            let pool = db::open_encrypted_pool(&db_path, &db_key)
                .await
                .map_err(|_| "无法打开加密数据库".to_string())?;
            *self.db.write().await = Some(pool);
        } else {
            let pool = db::open_encrypted_pool(&db_path, &db_key)
                .await
                .map_err(|_| "无法创建数据库".to_string())?;
            db::run_migrations(&pool)
                .await
                .map_err(|_| "数据库初始化失败".to_string())?;
            *self.db.write().await = Some(pool);
        }

        db::write_sidecar_salt(&self.data_dir, &salt).map_err(|_| "保存 salt 失败".to_string())?;
        *self.encryption_key.write().await = Some(field_key);
        *self.kdf_salt.write().await = Some(Zeroizing::new(salt));

        Ok(())
    }

    /// 修改主密码后轮换 SQLCipher 密钥
    pub async fn rekey_database(&self, new_field_key: &Zeroizing<[u8; 32]>) -> Result<(), String> {
        let db_key =
            kdf::derive_db_key(new_field_key).map_err(|_| "密钥派生失败".to_string())?;
        let pool = self.db_pool().await?;
        db::rekey_pool(&pool, &db_key)
            .await
            .map_err(|_| "数据库密钥轮换失败".to_string())?;
        Ok(())
    }

    pub async fn lock(&self) {
        self.sessions.destroy_all().await;

        // 优先销毁 session，避免出现“session 仍有效但 key 已清空”的短暂窗口
        let mut key = self.encryption_key.write().await;
        *key = None;
        drop(key);

        let mut salt = self.kdf_salt.write().await;
        *salt = None;

        if let Some(pool) = self.db.write().await.take() {
            db::close_pool(pool).await;
        }

        self.unlock_failures.store(0, Ordering::SeqCst);
    }

    pub fn is_unlocked(&self) -> bool {
        self.encryption_key
            .try_read()
            .map(|k| k.is_some())
            .unwrap_or(false)
    }

    pub fn record_unlock_failure(&self) {
        self.unlock_failures.fetch_add(1, Ordering::SeqCst);
        self.last_failure_time
            .store(chrono::Utc::now().timestamp(), Ordering::SeqCst);
    }

    pub fn reset_unlock_failures(&self) {
        self.unlock_failures.store(0, Ordering::SeqCst);
    }

    pub fn is_unlock_locked(&self) -> bool {
        let failures = self.unlock_failures.load(Ordering::SeqCst);
        if failures < 5 {
            return false;
        }
        let last = self.last_failure_time.load(Ordering::SeqCst);
        let now = chrono::Utc::now().timestamp();
        now - last < 300
    }
}
