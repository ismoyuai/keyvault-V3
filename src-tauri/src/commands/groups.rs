// src-tauri/src/commands/groups.rs
use tauri::State;
use serde::Serialize;
use crate::state::AppState;
use crate::db::queries;

#[derive(Serialize)]
pub struct GroupInfo {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[tauri::command]
pub async fn list_groups(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<GroupInfo>, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    let db = state.db_pool().await?;
    let rows = queries::list_groups(&db)
        .await
        .map_err(|_| "操作失败".to_string())?;
    Ok(rows.into_iter().map(|r| GroupInfo {
        id: r.id,
        name: r.name,
        icon: r.icon,
        color: r.color,
        sort_order: r.sort_order,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }).collect())
}

#[tauri::command]
pub async fn create_group(
    state: State<'_, AppState>,
    session_token: String,
    name: String,
    icon: Option<String>,
) -> Result<String, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    let db = state.db_pool().await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO groups (id, name, icon, sort_order, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)"
    )
    .bind(&id)
    .bind(&name)
    .bind(&icon)
    .bind(now)
    .bind(now)
    .execute(&db)
    .await
    .map_err(|_| "操作失败".to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn update_group(
    state: State<'_, AppState>,
    session_token: String,
    group_id: String,
    name: String,
    icon: Option<String>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    let db = state.db_pool().await?;
    let now = chrono::Utc::now().timestamp();
    sqlx::query("UPDATE groups SET name = ?, icon = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(&icon)
        .bind(now)
        .bind(&group_id)
        .execute(&db)
        .await
        .map_err(|_| "操作失败".to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_group(
    state: State<'_, AppState>,
    session_token: String,
    group_id: String,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    let db = state.db_pool().await?;
    sqlx::query("DELETE FROM groups WHERE id = ?")
        .bind(&group_id)
        .execute(&db)
        .await
        .map_err(|_| "操作失败".to_string())?;
    Ok(())
}
