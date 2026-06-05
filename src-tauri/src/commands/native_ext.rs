use serde::Serialize;

use crate::crypto::cipher;
use crate::db::queries;
use crate::state::AppState;

#[derive(Serialize)]
pub struct CredentialMatch {
    pub id: String,
    pub title: String,
    pub username: Option<String>,
}

#[derive(Serialize)]
pub struct FillResult {
    pub username: Option<String>,
    pub password: String,
}

/// 按 URL 查找匹配凭据（不含密码）
pub async fn find_credentials_by_url(
    session_token: &str,
    url: &str,
    state: &AppState,
) -> Result<Vec<CredentialMatch>, String> {
    if !state.sessions.validate(session_token).await {
        return Err("会话已过期".to_string());
    }

    let hostname = extract_hostname(url).ok_or("无效的 URL")?;

    let db = state.db_pool().await?;
    let entries = queries::list_entries(&db, 100, 0)
        .await
        .map_err(|e| e.to_string())?;

    let mut matches: Vec<CredentialMatch> = Vec::new();

    // 按 subtitle 匹配
    for entry in &entries {
        let subtitle_match = entry
            .subtitle
            .as_ref()
            .map(|s| s.contains(&hostname))
            .unwrap_or(false);

        if subtitle_match {
            let username = get_username_for_entry(state, &entry.id).await;
            matches.push(CredentialMatch {
                id: entry.id.clone(),
                title: entry.title.clone(),
                username,
            });
        }
    }

    // 如果 subtitle 没匹配到，尝试解密 url 字段匹配
    if matches.is_empty() {
        let key_guard = state.encryption_key.read().await;
        if let Some(key) = key_guard.as_ref() {
            for entry in &entries {
                let fields = queries::get_entry_fields(&db, &entry.id)
                    .await
                    .unwrap_or_default();

                for field in &fields {
                    if field.field_key == "url" {
                        if let Ok(decrypted) = cipher::decrypt_field(key, &field.enc_value) {
                            if let Ok(url_val) = String::from_utf8(decrypted.to_vec()) {
                                if url_val.contains(&hostname) {
                                    let username = get_username_for_entry(state, &entry.id).await;
                                    matches.push(CredentialMatch {
                                        id: entry.id.clone(),
                                        title: entry.title.clone(),
                                        username,
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(matches)
}

/// 获取条目的 username 字段值
async fn get_username_for_entry(state: &AppState, entry_id: &str) -> Option<String> {
    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref()?;
    let db = state.db_pool().await.ok()?;

    let fields = queries::get_entry_fields(&db, entry_id)
        .await
        .ok()?;

    for field in &fields {
        if field.field_key == "username" {
            if let Ok(decrypted) = cipher::decrypt_field(key, &field.enc_value) {
                return String::from_utf8(decrypted.to_vec()).ok();
            }
        }
    }

    None
}

/// 获取条目的完整凭据（用于填充）
pub async fn get_entry_for_fill(
    session_token: &str,
    entry_id: &str,
    state: &AppState,
) -> Result<FillResult, String> {
    if !state.sessions.validate(session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;
    let db = state.db_pool().await?;

    let fields = queries::get_entry_fields(&db, entry_id)
        .await
        .map_err(|e| e.to_string())?;

    let mut username: Option<String> = None;
    let mut password: Option<String> = None;

    for field in &fields {
        let decrypted = cipher::decrypt_field(key, &field.enc_value)
            .map_err(|e| e.to_string())?;
        let value = String::from_utf8(decrypted.to_vec())
            .map_err(|_| "解码失败".to_string())?;

        match field.field_key.as_str() {
            "username" => username = Some(value),
            "password" => password = Some(value),
            _ => {}
        }
    }

    Ok(FillResult {
        username,
        password: password.ok_or("未找到密码字段")?,
    })
}

/// 从 URL 提取 hostname
fn extract_hostname(url: &str) -> Option<String> {
    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    let hostname = without_protocol.split('/').next()?;
    let hostname = hostname.split(':').next()?;
    let hostname = hostname.strip_prefix("www.").unwrap_or(hostname);

    Some(hostname.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hostname() {
        assert_eq!(extract_hostname("https://github.com/login"), Some("github.com".to_string()));
        assert_eq!(extract_hostname("http://www.google.com:8080/path"), Some("google.com".to_string()));
        assert_eq!(extract_hostname("https://app.example.com"), Some("app.example.com".to_string()));
    }
}
