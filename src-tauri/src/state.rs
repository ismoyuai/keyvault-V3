use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};

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
    pub kdf_salt: RwLock<Option<Zeroizing<[u8; 16]>>>,
    /// 密码验证失败次数
    pub unlock_failures: AtomicU32,
    /// 上次失败时间戳（用于自动重置）
    pub last_failure_time: AtomicI64,
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

        // 重置失败计数
        self.unlock_failures.store(0, Ordering::SeqCst);
    }

    pub fn is_unlocked(&self) -> bool {
        self.encryption_key
            .try_read()
            .map(|k| k.is_some())
            .unwrap_or(false)
    }

    /// 记录一次解锁失败
    pub fn record_unlock_failure(&self) {
        self.unlock_failures.fetch_add(1, Ordering::SeqCst);
        self.last_failure_time
            .store(chrono::Utc::now().timestamp(), Ordering::SeqCst);
    }

    /// 重置失败计数
    pub fn reset_unlock_failures(&self) {
        self.unlock_failures.store(0, Ordering::SeqCst);
    }

    /// 检查是否被锁定（超过 5 次失败，5 分钟内禁止尝试）
    pub fn is_unlock_locked(&self) -> bool {
        let failures = self.unlock_failures.load(Ordering::SeqCst);
        if failures < 5 {
            return false;
        }
        let last = self.last_failure_time.load(Ordering::SeqCst);
        let now = chrono::Utc::now().timestamp();
        // 5 分钟锁定
        now - last < 300
    }
}
