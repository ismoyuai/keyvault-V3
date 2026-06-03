use sqlx::SqlitePool;
use tokio::sync::RwLock;
use zeroize::Zeroizing;

use crate::crypto::session::SessionManager;

pub struct AppState {
    /// 加密密钥，仅在解锁后存在，锁定时清零
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
    /// SQLCipher 连接池（数据库始终保持加密）
    pub db: SqlitePool,
    /// 会话令牌管理器
    pub sessions: SessionManager,
    /// KDF 的 salt（存储在 DB 的 config 表中）
    pub kdf_salt: RwLock<Option<[u8; 16]>>,
}

impl AppState {
    pub async fn lock(&self) {
        // 先清零加密密钥（高优先级）
        let mut key = self.encryption_key.write().await;
        *key = None;
        drop(key); // 释放写锁

        // 再销毁所有会话
        self.sessions.destroy_all().await;

        // 清零 salt
        let mut salt = self.kdf_salt.write().await;
        *salt = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.encryption_key
            .try_read()
            .map(|k| k.is_some())
            .unwrap_or(false)
    }
}
