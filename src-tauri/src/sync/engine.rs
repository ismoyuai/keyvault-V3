use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::crypto::cipher;

pub const REMOTE_PATH: &str = "/keyvault/data.kv";
pub const SYNC_ENCRYPTED_VERSION: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncField {
    pub field_key: String,
    pub field_type: String,
    pub value: String,
    pub is_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntry {
    pub id: String,
    pub entry_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Option<String>,
    pub favorited: bool,
    pub group_name: Option<String>,
    pub updated_at: i64,
    #[serde(rename = "deletedAt", skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    pub fields: Vec<SyncField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncData {
    pub entries: Vec<SyncEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPayload {
    pub version: u32,
    pub device_id: String,
    pub exported_at: String,
    pub data: SyncData,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedSyncFile {
    pub version: u32,
    pub ciphertext: String,
}

pub fn compute_checksum(data: &SyncData) -> Result<String, String> {
    let json = serde_json::to_string(data).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}

pub fn build_payload(device_id: &str, entries: Vec<SyncEntry>) -> Result<SyncPayload, String> {
    let data = SyncData { entries };
    let checksum = compute_checksum(&data)?;
    Ok(SyncPayload {
        version: 2,
        device_id: device_id.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        data,
        checksum,
    })
}

pub fn verify_payload(payload: &SyncPayload) -> Result<(), String> {
    let expected = compute_checksum(&payload.data)?;
    if payload.checksum != expected {
        return Err("数据校验失败，远程文件可能被篡改".to_string());
    }
    Ok(())
}

/// 按 updated_at 合并条目（较新者胜出，含 deleted_at）
pub fn merge_entries(local: Vec<SyncEntry>, remote: Vec<SyncEntry>) -> Vec<SyncEntry> {
    let mut merged: std::collections::HashMap<String, SyncEntry> =
        local.into_iter().map(|e| (e.id.clone(), e)).collect();

    for remote_entry in remote {
        match merged.get(&remote_entry.id) {
            Some(local_entry) if local_entry.updated_at >= remote_entry.updated_at => {}
            _ => {
                merged.insert(remote_entry.id.clone(), remote_entry);
            }
        }
    }

    merged.into_values().collect()
}

/// AES-256-GCM 加密后上传（v3 格式）
pub fn serialize_encrypted(key: &[u8; 32], payload: &SyncPayload) -> Result<String, String> {
    let json = serde_json::to_string(payload).map_err(|e| e.to_string())?;
    let ciphertext = cipher::encrypt_field(key, json.as_bytes()).map_err(|e| e.to_string())?;
    let blob = EncryptedSyncFile {
        version: SYNC_ENCRYPTED_VERSION,
        ciphertext,
    };
    serde_json::to_string(&blob).map_err(|e| e.to_string())
}

/// 解密同步文件（生产拒绝明文 v2 降级）
pub fn deserialize_payload(key: &[u8; 32], content: &str) -> Result<SyncPayload, String> {
    let blob = serde_json::from_str::<EncryptedSyncFile>(content)
        .map_err(|_| "远程数据不是 v3 加密格式（拒绝明文 v2 回退）".to_string())?;

    if blob.version != SYNC_ENCRYPTED_VERSION {
        return Err("远程同步文件版本不支持".to_string());
    }

    let decrypted = cipher::decrypt_field(key, &blob.ciphertext).map_err(|e| e.to_string())?;
    let payload: SyncPayload =
        serde_json::from_slice(&decrypted).map_err(|e| format!("解密数据解析失败: {}", e))?;
    verify_payload(&payload)?;
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(id: &str, title: &str, updated_at: i64) -> SyncEntry {
        SyncEntry {
            id: id.into(),
            entry_type: "login".into(),
            title: title.into(),
            subtitle: None,
            tags: None,
            favorited: false,
            group_name: None,
            updated_at,
            deleted_at: None,
            fields: vec![],
        }
    }

    #[test]
    fn test_merge_entries_newer_wins() {
        let local = vec![sample_entry("a", "Local", 100)];
        let remote = vec![sample_entry("a", "Remote", 200)];
        let merged = merge_entries(local, remote);
        assert_eq!(merged[0].title, "Remote");
    }

    #[test]
    fn test_merge_entries_equal_timestamp_keeps_local() {
        let local = vec![sample_entry("a", "Local", 100)];
        let remote = vec![sample_entry("a", "Remote", 100)];
        let merged = merge_entries(local, remote);
        assert_eq!(merged[0].title, "Local");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let entries = vec![sample_entry("x", "Secret", 1)];
        let payload = build_payload("device-1", entries).unwrap();
        let encrypted = serialize_encrypted(&key, &payload).unwrap();
        let restored = deserialize_payload(&key, &encrypted).unwrap();
        assert_eq!(restored.data.entries[0].title, "Secret");
    }

    #[test]
    fn test_reject_plaintext_v2_sync_payload() {
        let key = [0x42u8; 32];
        let entries = vec![sample_entry("x", "Secret", 1)];
        let payload = build_payload("device-1", entries).unwrap();
        let plaintext = serde_json::to_string(&payload).unwrap();

        let err = deserialize_payload(&key, &plaintext).unwrap_err();
        assert!(
            err.contains("拒绝") || err.contains("加密"),
            "unexpected error: {err}"
        );
    }
}
