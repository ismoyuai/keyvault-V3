use serde::Serialize;
use sha1::{Digest, Sha1};
use tauri::State;
use zeroize::Zeroizing;

use crate::error::ipc_db_err;
use crate::state::AppState;

#[derive(Serialize)]
pub struct BreachResult {
    pub breached: bool,
    pub count: u64,
}

/// k-匿名性：只发送 SHA1 前 5 字符，在本地比对完整哈希
/// 密码原文永远不离开设备
#[tauri::command]
pub async fn check_password_breach(
    session_token: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<BreachResult, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let password_bytes = Zeroizing::new(password.into_bytes());
    let db = state.db_pool().await?;

    let mut hasher = Sha1::new();
    hasher.update(password_bytes.as_slice());
    let hash = format!("{:X}", hasher.finalize());

    let prefix = &hash[..5];
    let suffix = &hash[5..];

    let cached: Option<(String, i64)> = sqlx::query_as(
        "SELECT result_json, cached_at FROM breach_cache WHERE hash_prefix = ?",
    )
    .bind(prefix)
    .fetch_optional(&db)
    .await
    .map_err(ipc_db_err)?;

    let now = chrono::Utc::now().timestamp();

    if let Some((result_json, cached_at)) = cached {
        if now - cached_at < 7 * 24 * 3600 {
            return parse_breach_result(&result_json, suffix);
        }
    }

    let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);
    let response = reqwest::get(&url)
        .await
        .map_err(|_| "网络请求失败，跳过泄露检测".to_string())?
        .text()
        .await
        .map_err(|_| "网络请求失败".to_string())?;

    let cache_json = serde_json::json!({ "lines": response });
    sqlx::query(
        "INSERT OR REPLACE INTO breach_cache (hash_prefix, result_json, cached_at) VALUES (?, ?, ?)",
    )
    .bind(prefix)
    .bind(cache_json.to_string())
    .bind(now)
    .execute(&db)
    .await
    .ok();

    for line in response.lines() {
        if let Some((s, count)) = line.split_once(':') {
            if s.trim().eq_ignore_ascii_case(suffix) {
                let count = count.trim().parse::<u64>().unwrap_or(1);
                return Ok(BreachResult {
                    breached: true,
                    count,
                });
            }
        }
    }

    Ok(BreachResult {
        breached: false,
        count: 0,
    })
}

fn parse_breach_result(json: &str, suffix: &str) -> Result<BreachResult, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|_| "缓存数据无效".to_string())?;
    let lines = v["lines"].as_str().unwrap_or("");
    for line in lines.lines() {
        if let Some((s, count)) = line.split_once(':') {
            if s.trim().eq_ignore_ascii_case(suffix) {
                let count = count.trim().parse::<u64>().unwrap_or(1);
                return Ok(BreachResult {
                    breached: true,
                    count,
                });
            }
        }
    }
    Ok(BreachResult {
        breached: false,
        count: 0,
    })
}
