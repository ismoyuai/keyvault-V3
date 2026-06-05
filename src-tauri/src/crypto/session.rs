use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::Rng;
use tokio::sync::RwLock;
use zeroize::Zeroizing;

const SESSION_TTL_SECS: u64 = 1800; // 30 分钟

pub struct SessionManager {
    sessions: RwLock<HashMap<String, SystemTime>>,
    ttl: Duration,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(SESSION_TTL_SECS))
    }

    #[cfg(test)]
    pub fn with_ttl_secs(secs: u64) -> Self {
        Self::with_ttl(Duration::from_secs(secs))
    }

    fn with_ttl(ttl: Duration) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    pub async fn create(&self) -> String {
        let token: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let mut sessions = self.sessions.write().await;
        sessions.insert(token.clone(), SystemTime::now());
        token
    }

    pub async fn validate(&self, token: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        match sessions.get_mut(token) {
            Some(created_at) => {
                if created_at.elapsed().unwrap_or_default() > self.ttl
                {
                    sessions.remove(token);
                    false
                } else {
                    // 滑动窗口：刷新时间
                    *created_at = SystemTime::now();
                    true
                }
            }
            None => false,
        }
    }

    pub async fn destroy_all(&self) {
        let mut sessions = self.sessions.write().await;
        // 清零每个 token 的内存
        for (token, _) in sessions.drain() {
            let mut token_bytes = Zeroizing::new(token.into_bytes());
            for b in token_bytes.iter_mut() {
                *b = 0;
            }
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_validate() {
        let mgr = SessionManager::new();
        let token = mgr.create().await;
        assert!(mgr.validate(&token).await);
    }

    #[tokio::test]
    async fn test_validate_invalid_token() {
        let mgr = SessionManager::new();
        assert!(!mgr.validate("nonexistent").await);
    }

    #[tokio::test]
    async fn test_destroy_all() {
        let mgr = SessionManager::new();
        let token = mgr.create().await;
        assert!(mgr.validate(&token).await);
        mgr.destroy_all().await;
        assert!(!mgr.validate(&token).await);
    }

    #[tokio::test]
    async fn test_sliding_window_refresh() {
        let mgr = SessionManager::new();
        let token = mgr.create().await;
        // Validate multiple times — each should succeed and refresh
        assert!(mgr.validate(&token).await);
        assert!(mgr.validate(&token).await);
        assert!(mgr.validate(&token).await);
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        let mgr = SessionManager::new();
        let token1 = mgr.create().await;
        let token2 = mgr.create().await;
        assert_ne!(token1, token2);
        assert!(mgr.validate(&token1).await);
        assert!(mgr.validate(&token2).await);
    }

    #[tokio::test]
    async fn test_session_expires_after_ttl() {
        let mgr = SessionManager::with_ttl_secs(1);
        let token = mgr.create().await;
        assert!(mgr.validate(&token).await);
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(!mgr.validate(&token).await);
    }
}
