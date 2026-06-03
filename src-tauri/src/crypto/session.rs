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
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
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
                if created_at
                    .elapsed()
                    .unwrap_or_default()
                    > Duration::from_secs(SESSION_TTL_SECS)
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
