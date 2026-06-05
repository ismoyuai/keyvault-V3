use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Method;

#[derive(Clone, Debug)]
pub struct WebDavConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

impl WebDavConfig {
    fn base_url(&self) -> String {
        self.url.trim_end_matches('/').to_string()
    }

    fn full_url(&self, remote_path: &str) -> String {
        format!("{}{}", self.base_url(), remote_path)
    }

    fn auth_header(&self) -> String {
        use base64::Engine;
        let creds = format!("{}:{}", self.username, self.password);
        format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(creds.as_bytes())
        )
    }

    fn client(&self) -> Result<reqwest::Client, String> {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())
    }
}

pub async fn test_connection(config: &WebDavConfig) -> Result<(), String> {
    let client = config.client()?;
    let response = client
        .request(Method::from_bytes(b"PROPFIND").map_err(|e| e.to_string())?, config.full_url("/"))
        .header(AUTHORIZATION, config.auth_header())
        .header("Depth", "0")
        .header(CONTENT_TYPE, "application/xml")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<propfind xmlns="DAV:"><prop><resourcetype/></prop></propfind>"#,
        )
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    if response.status().is_success() || response.status().as_u16() == 207 {
        Ok(())
    } else {
        Err(format!("WebDAV 响应异常: {}", response.status()))
    }
}

async fn ensure_directory(config: &WebDavConfig, remote_path: &str) -> Result<(), String> {
    let dir = remote_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    if dir.is_empty() {
        return Ok(());
    }

    let client = config.client()?;
    let response = client
        .request(Method::from_bytes(b"MKCOL").map_err(|e| e.to_string())?, config.full_url(dir))
        .header(AUTHORIZATION, config.auth_header())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() || response.status().as_u16() == 405 {
        Ok(())
    } else {
        Err(format!("创建目录失败: {}", response.status()))
    }
}

pub async fn upload(config: &WebDavConfig, remote_path: &str, data: &str) -> Result<(), String> {
    ensure_directory(config, remote_path).await?;
    let client = config.client()?;
    let response = client
        .put(config.full_url(remote_path))
        .header(AUTHORIZATION, config.auth_header())
        .header(CONTENT_TYPE, "application/json")
        .body(data.to_string())
        .send()
        .await
        .map_err(|e| format!("上传失败: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("上传失败: {}", response.status()))
    }
}

pub async fn download(config: &WebDavConfig, remote_path: &str) -> Result<Option<String>, String> {
    let client = config.client()?;
    let response = client
        .get(config.full_url(remote_path))
        .header(AUTHORIZATION, config.auth_header())
        .send()
        .await
        .map_err(|e| format!("下载失败: {}", e))?;

    if response.status().as_u16() == 404 {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!("下载失败: {}", response.status()));
    }

    response
        .text()
        .await
        .map(Some)
        .map_err(|e| e.to_string())
}

pub async fn get_last_modified(config: &WebDavConfig, remote_path: &str) -> Result<Option<String>, String> {
    let client = config.client()?;
    let response = client
        .request(Method::HEAD, config.full_url(remote_path))
        .header(AUTHORIZATION, config.auth_header())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().as_u16() == 404 {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!("获取远程状态失败: {}", response.status()));
    }

    Ok(response
        .headers()
        .get("last-modified")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string()))
}
