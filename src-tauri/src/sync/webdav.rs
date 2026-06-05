use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Method;

use crate::error::ipc_network_err;

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
            .map_err(|e| ipc_network_err("创建 HTTP 客户端失败", e))
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
        .map_err(|e| ipc_network_err("WebDAV 连接失败", e))?;

    if response.status().is_success() || response.status().as_u16() == 207 {
        Ok(())
    } else {
        tracing::error!("WebDAV 响应异常: {}", response.status());
        Err("WebDAV 连接失败，请检查地址和凭据".to_string())
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
        .map_err(|e| ipc_network_err("WebDAV 创建目录失败", e))?;

    if response.status().is_success() || response.status().as_u16() == 405 {
        Ok(())
    } else {
        tracing::error!("WebDAV 创建目录失败: {}", response.status());
        Err("同步目录创建失败，请重试".to_string())
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
        .map_err(|e| ipc_network_err("WebDAV 上传失败", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        tracing::error!("WebDAV 上传失败: {}", response.status());
        Err("同步上传失败，请重试".to_string())
    }
}

pub async fn download(config: &WebDavConfig, remote_path: &str) -> Result<Option<String>, String> {
    let client = config.client()?;
    let response = client
        .get(config.full_url(remote_path))
        .header(AUTHORIZATION, config.auth_header())
        .send()
        .await
        .map_err(|e| ipc_network_err("WebDAV 下载失败", e))?;

    if response.status().as_u16() == 404 {
        return Ok(None);
    }
    if !response.status().is_success() {
        tracing::error!("WebDAV 下载失败: {}", response.status());
        return Err("同步下载失败，请重试".to_string());
    }

    response
        .text()
        .await
        .map(Some)
        .map_err(|e| ipc_network_err("WebDAV 读取响应失败", e))
}

pub async fn get_last_modified(config: &WebDavConfig, remote_path: &str) -> Result<Option<String>, String> {
    let client = config.client()?;
    let response = client
        .request(Method::HEAD, config.full_url(remote_path))
        .header(AUTHORIZATION, config.auth_header())
        .send()
        .await
        .map_err(|e| ipc_network_err("WebDAV 获取状态失败", e))?;

    if response.status().as_u16() == 404 {
        return Ok(None);
    }
    if !response.status().is_success() {
        tracing::error!("WebDAV 获取状态失败: {}", response.status());
        return Err("获取远程同步状态失败，请重试".to_string());
    }

    Ok(response
        .headers()
        .get("last-modified")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string()))
}
