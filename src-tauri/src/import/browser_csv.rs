//! 解析 Chrome / Edge / Firefox 等浏览器导出的 CSV 密码文件

use std::collections::HashMap;

use crate::commands::export_cmd::{ImportEntry, ImportField};

/// 去掉 UTF-8 BOM 与首尾空白
fn normalize_content(content: &str) -> String {
    content.trim_start_matches('\u{FEFF}').trim().to_string()
}

fn header_index(headers: &csv::StringRecord) -> HashMap<String, usize> {
    headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.trim().to_lowercase(), i))
        .collect()
}

fn field_at(record: &csv::StringRecord, index: &HashMap<String, usize>, key: &str) -> String {
    index
        .get(key)
        .and_then(|&i| record.get(i))
        .unwrap_or("")
        .trim()
        .to_string()
}

fn host_from_url(url: &str) -> Option<String> {
    let rest = url.split("://").nth(1).unwrap_or(url);
    let host_port = rest.split('/').next().unwrap_or(rest);
    let host = host_port.split('@').next_back().unwrap_or(host_port);
    let host = host.split(':').next().unwrap_or(host).trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

fn title_from_row(name: &str, url: &str, username: &str) -> String {
    if !name.is_empty() {
        return name.to_string();
    }
    if let Some(host) = host_from_url(url) {
        return host;
    }
    if !url.is_empty() {
        return url.to_string();
    }
    if !username.is_empty() {
        return username.to_string();
    }
    "未命名".to_string()
}

fn row_to_entry(record: &csv::StringRecord, index: &HashMap<String, usize>) -> Option<ImportEntry> {
    let name = field_at(record, index, "name");
    let url = field_at(record, index, "url");
    let username = field_at(record, index, "username");
    let password = field_at(record, index, "password");
    let note = field_at(record, index, "note");
    let notes = field_at(record, index, "notes");
    let note_text = if !note.is_empty() { note } else { notes };

    if url.is_empty() && username.is_empty() && password.is_empty() {
        return None;
    }

    let title = title_from_row(&name, &url, &username);
    let subtitle = if url.is_empty() { None } else { Some(url.clone()) };

    let mut fields = Vec::new();

    if !username.is_empty() {
        fields.push(ImportField {
            field_key: "username".to_string(),
            field_type: Some("text".to_string()),
            value: username,
            is_sensitive: Some(false),
        });
    }

    if !password.is_empty() {
        fields.push(ImportField {
            field_key: "password".to_string(),
            field_type: Some("password".to_string()),
            value: password,
            is_sensitive: Some(true),
        });
    }

    if !url.is_empty() {
        fields.push(ImportField {
            field_key: "url".to_string(),
            field_type: Some("url".to_string()),
            value: url.clone(),
            is_sensitive: Some(false),
        });
    }

    if !note_text.is_empty() {
        fields.push(ImportField {
            field_key: "notes".to_string(),
            field_type: Some("textarea".to_string()),
            value: note_text,
            is_sensitive: Some(false),
        });
    }

    if fields.is_empty() {
        return None;
    }

    Some(ImportEntry {
        entry_type: "login".to_string(),
        title,
        subtitle,
        tags: None,
        favorited: None,
        group_name: None,
        fields,
    })
}

/// 解析浏览器导出的 CSV（Chrome / Edge / Firefox 等）
pub fn parse_browser_csv(content: &str) -> Result<Vec<ImportEntry>, String> {
    let normalized = normalize_content(content);
    if normalized.is_empty() {
        return Err("CSV 文件为空".to_string());
    }

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(normalized.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|_| "无法读取 CSV 表头".to_string())?
        .clone();
    let index = header_index(&headers);

    let has_password_col = index.contains_key("password");
    let has_username_col = index.contains_key("username");
    let has_url_col = index.contains_key("url");

    if !has_password_col && !(has_username_col && has_url_col) {
        return Err(
            "不支持的 CSV 格式：需要包含 password 列，或 url + username 列（Chrome/Edge/Firefox 导出）"
                .to_string(),
        );
    }

    let mut entries = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|_| "CSV 行解析失败，请确认文件为浏览器导出的密码 CSV".to_string())?;
        if let Some(entry) = row_to_entry(&record, &index) {
            entries.push(entry);
        }
    }

    if entries.is_empty() {
        return Err("CSV 中未找到可导入的密码记录".to_string());
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chrome_csv() {
        let csv = r#"name,url,username,password
Google,https://accounts.google.com,user@gmail.com,SecretPass1
GitHub,https://github.com,devuser,ghp_xxx"#;

        let entries = parse_browser_csv(csv).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "Google");
        assert_eq!(entries[0].subtitle.as_deref(), Some("https://accounts.google.com"));
        assert_eq!(entries[0].entry_type, "login");
        assert!(entries[0]
            .fields
            .iter()
            .any(|f| f.field_key == "password" && f.value == "SecretPass1"));
    }

    #[test]
    fn parse_chrome_csv_quoted_fields() {
        let csv = r#"name,url,username,password
"Example, Inc.","https://example.com","user","pass""#;
        let entries = parse_browser_csv(csv).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Example, Inc.");
    }

    #[test]
    fn parse_edge_csv_with_note() {
        let csv = r#"name,url,username,password,note
Edge Site,https://edge.example.com,admin,pass123,work account"#;
        let entries = parse_browser_csv(csv).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .fields
            .iter()
            .any(|f| f.field_key == "notes" && f.value.contains("work")));
    }

    #[test]
    fn parse_firefox_csv() {
        let csv = r#"url,username,password,httpRealm,formActionOrigin,guid,timeCreated,timeLastUsed,timePasswordChanged
https://mozilla.org,fox,firefox-pass,,,,,,"#;
        let entries = parse_browser_csv(csv).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "mozilla.org");
    }

    #[test]
    fn parse_csv_with_bom() {
        let csv = "\u{FEFF}name,url,username,password\nTest,https://t.com,u,p";
        let entries = parse_browser_csv(csv).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn reject_unknown_csv() {
        let csv = "foo,bar\n1,2";
        assert!(parse_browser_csv(csv).is_err());
    }
}
