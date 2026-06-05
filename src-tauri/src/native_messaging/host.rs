use std::io::{self, Read, Write};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::commands::native_ext;
use crate::state::AppState;

#[derive(Deserialize)]
struct Request {
    #[serde(rename = "requestId")]
    request_id: u32,
    action: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(rename = "entryId", default)]
    entry_id: Option<String>,
    #[serde(default)]
    query: Option<String>,
}

#[derive(Serialize)]
struct Response {
    #[serde(rename = "requestId")]
    request_id: u32,
    #[serde(flatten)]
    data: ResponseData,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ResponseData {
    Entries { entries: Vec<serde_json::Value> },
    Fill { username: Option<String>, password: String },
    Status { status: String },
    Error { error: String },
}

pub fn run() {
    let rt = Runtime::new().expect("无法创建 tokio 运行时");

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("keyvault");
    std::fs::create_dir_all(&data_dir).ok();

    let state = AppState::new(data_dir);

    // Native Messaging Host 自身的 session token（与 GUI 解锁态不共享，见 C-6）
    let host_session_token = rt.block_on(state.sessions.create());

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    loop {
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).is_err() {
            break;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        if len > 1024 * 1024 {
            break;
        }

        let mut payload = vec![0u8; len];
        if reader.read_exact(&mut payload).is_err() {
            break;
        }

        let request: Request = match serde_json::from_slice(&payload) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let response_data = rt.block_on(handle_request(&request, &state, &host_session_token));

        let response = Response {
            request_id: request.request_id,
            data: response_data,
        };

        let response_json = serde_json::to_vec(&response).unwrap();
        let len = (response_json.len() as u32).to_le_bytes();
        let _ = writer.write_all(&len);
        let _ = writer.write_all(&response_json);
        let _ = writer.flush();
    }
}

async fn handle_request(request: &Request, state: &AppState, host_token: &str) -> ResponseData {
    let token = host_token;

    match request.action.as_str() {
        "get_status" => {
            if state.is_unlocked() {
                ResponseData::Status {
                    status: "connected".to_string(),
                }
            } else {
                ResponseData::Status {
                    status: "locked".to_string(),
                }
            }
        }

        "find_credentials" => {
            let url = match &request.url {
                Some(u) => u.clone(),
                None => {
                    return ResponseData::Error {
                        error: "缺少 url 参数".to_string(),
                    }
                }
            };

            match native_ext::find_credentials_by_url(token, &url, state).await {
                Ok(entries) => {
                    let entries_json: Vec<serde_json::Value> = entries
                        .iter()
                        .map(|e| serde_json::to_value(e).unwrap())
                        .collect();
                    ResponseData::Entries {
                        entries: entries_json,
                    }
                }
                Err(e) => ResponseData::Error { error: e },
            }
        }

        "get_entry_for_fill" => {
            let entry_id = match &request.entry_id {
                Some(id) => id.clone(),
                None => {
                    return ResponseData::Error {
                        error: "缺少 entryId 参数".to_string(),
                    }
                }
            };

            match native_ext::get_entry_for_fill(token, &entry_id, state).await {
                Ok(result) => ResponseData::Fill {
                    username: result.username,
                    password: result.password,
                },
                Err(e) => ResponseData::Error { error: e },
            }
        }

        "search" => {
            let query = match &request.query {
                Some(q) => q.clone(),
                None => {
                    return ResponseData::Error {
                        error: "缺少 query 参数".to_string(),
                    }
                }
            };

            if !state.sessions.validate(token).await {
                return ResponseData::Error {
                    error: "会话已过期".to_string(),
                };
            }

            let db = match state.db_pool().await {
                Ok(db) => db,
                Err(e) => return ResponseData::Error { error: e },
            };

            match crate::db::queries::search_entries(&db, &query).await {
                Ok(entries) => {
                    let entries_json: Vec<serde_json::Value> = entries
                        .iter()
                        .map(|e| {
                            serde_json::json!({
                                "id": e.id,
                                "title": e.title,
                                "username": Option::<String>::None,
                            })
                        })
                        .collect();
                    ResponseData::Entries {
                        entries: entries_json,
                    }
                }
                Err(e) => ResponseData::Error {
                    error: e.to_string(),
                },
            }
        }

        _ => ResponseData::Error {
            error: format!("未知操作: {}", request.action),
        },
    }
}
