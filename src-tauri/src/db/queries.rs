use sqlx::SqlitePool;

use super::schema::{EntryMeta, FieldRow, GroupRow};

// ============================================
// 条目查询
// ============================================

pub async fn list_entries(pool: &SqlitePool) -> Result<Vec<EntryMeta>, sqlx::Error> {
    sqlx::query_as::<_, EntryMeta>(
        "SELECT id, entry_type, title, subtitle, tags, favorited, group_id, updated_at
         FROM entries ORDER BY favorited DESC, updated_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn search_entries(pool: &SqlitePool, query: &str) -> Result<Vec<EntryMeta>, sqlx::Error> {
    let pattern = format!("%{}%", query);
    sqlx::query_as::<_, EntryMeta>(
        "SELECT id, entry_type, title, subtitle, tags, favorited, group_id, updated_at
         FROM entries
         WHERE title LIKE ?1 OR subtitle LIKE ?1 OR tags LIKE ?1
         ORDER BY favorited DESC, updated_at DESC",
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await
}

pub async fn get_entry_fields(pool: &SqlitePool, entry_id: &str) -> Result<Vec<FieldRow>, sqlx::Error> {
    sqlx::query_as::<_, FieldRow>(
        "SELECT field_key, field_type, enc_value, is_sensitive
         FROM fields WHERE entry_id = ? ORDER BY sort_order",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
}

pub async fn entry_exists(pool: &SqlitePool, entry_id: &str) -> Result<bool, sqlx::Error> {
    let result: Option<(String,)> =
        sqlx::query_as("SELECT id FROM entries WHERE id = ?")
            .bind(entry_id)
            .fetch_optional(pool)
            .await?;
    Ok(result.is_some())
}

// ============================================
// 分组查询
// ============================================

pub async fn list_groups(pool: &SqlitePool) -> Result<Vec<GroupRow>, sqlx::Error> {
    sqlx::query_as::<_, GroupRow>(
        "SELECT id, name, icon, color, sort_order, created_at, updated_at
         FROM groups ORDER BY sort_order ASC",
    )
    .fetch_all(pool)
    .await
}

// ============================================
// 配置查询
// ============================================

pub async fn get_config(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let result: Option<(String,)> =
        sqlx::query_as("SELECT value FROM config WHERE key = ?")
            .bind(key)
            .fetch_optional(pool)
            .await?;
    Ok(result.map(|(v,)| v))
}

pub async fn set_config(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}

// ============================================
// 审计日志
// ============================================

pub async fn write_audit_log(
    pool: &SqlitePool,
    action: &str,
    entry_id: Option<&str>,
    field_key: Option<&str>,
    metadata: Option<&str>,
) -> Result<(), sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO audit_log (id, action, entry_id, field_key, metadata, occurred_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(action)
    .bind(entry_id)
    .bind(field_key)
    .bind(metadata)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}
