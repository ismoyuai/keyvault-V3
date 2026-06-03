use serde::Serialize;
use sha1::{Digest, Sha1};
use tauri::State;

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

    // 1. 计算 SHA1 哈希
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    let hash = format!("{:X}", hasher.finalize());

    let prefix = &hash[..5];
    let suffix = &hash[5..];

    // 2. 检查缓存
    let cached: Option<(String, i64)> = sqlx::query_as(
        "SELECT result_json, cached_at FROM breach_cache WHERE hash_prefix = ?",
    )
    .bind(prefix)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().timestamp();

    // 缓存有效期 7 天
    if let Some((result_json, cached_at)) = cached {
        if now - cached_at < 7 * 24 * 3600 {
            return parse_breach_result(&result_json, suffix);
        }
    }

    // 3. 查询 HIBP API（仅发送前 5 字符）
    let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);
    let response = reqwest::get(&url)
        .await
        .map_err(|_| "网络请求失败，跳过泄露检测".to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    // 4. 缓存结果
    let cache_json = serde_json::json!({ "lines": response });
    sqlx::query(
        "INSERT OR REPLACE INTO breach_cache (hash_prefix, result_json, cached_at) VALUES (?, ?, ?)",
    )
    .bind(prefix)
    .bind(cache_json.to_string())
    .bind(now)
    .execute(&state.db)
    .await
    .ok();

    // 5. 本地比对
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
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
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
