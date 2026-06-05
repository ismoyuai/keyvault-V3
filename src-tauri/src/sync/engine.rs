use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const REMOTE_PATH: &str = "/keyvault/data.kv";

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

pub fn compute_checksum(data: &SyncData) -> String {
    let json = serde_json::to_string(data).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn build_payload(device_id: &str, entries: Vec<SyncEntry>) -> SyncPayload {
    let data = SyncData { entries };
    let checksum = compute_checksum(&data);
    SyncPayload {
        version: 2,
        device_id: device_id.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        data,
        checksum,
    }
}

pub fn verify_payload(payload: &SyncPayload) -> Result<(), String> {
    let expected = compute_checksum(&payload.data);
    if payload.checksum != expected {
        return Err("数据校验失败，远程文件可能被篡改".to_string());
    }
    Ok(())
}

/// 按 updated_at 合并条目（较新者胜出）
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_entries_newer_wins() {
        let local = vec![SyncEntry {
            id: "a".into(),
            entry_type: "login".into(),
            title: "Local".into(),
            subtitle: None,
            tags: None,
            favorited: false,
            group_name: None,
            updated_at: 100,
            fields: vec![],
        }];
        let remote = vec![SyncEntry {
            id: "a".into(),
            entry_type: "login".into(),
            title: "Remote".into(),
            subtitle: None,
            tags: None,
            favorited: false,
            group_name: None,
            updated_at: 200,
            fields: vec![],
        }];
        let merged = merge_entries(local, remote);
        assert_eq!(merged[0].title, "Remote");
    }
}
