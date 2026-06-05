use sqlx::SqlitePool;

use super::schema::{EntryMeta, FieldRow, GroupRow};

// ============================================
// 条目查询
// ============================================

const ENTRY_META_COLS: &str =
    "id, entry_type, title, subtitle, tags, favorited, group_id, updated_at, deleted_at";

/// 同步导出用：含回收站条目（全量，无上限）
pub async fn list_all_entries_for_sync(pool: &SqlitePool) -> Result<Vec<EntryMeta>, sqlx::Error> {
    sqlx::query_as::<_, EntryMeta>(&format!(
        "SELECT {ENTRY_META_COLS}
         FROM entries
         ORDER BY updated_at DESC",
    ))
    .fetch_all(pool)
    .await
}

pub async fn list_entries(pool: &SqlitePool, limit: i64, offset: i64) -> Result<Vec<EntryMeta>, sqlx::Error> {
    sqlx::query_as::<_, EntryMeta>(&format!(
        "SELECT {ENTRY_META_COLS}
         FROM entries WHERE deleted_at IS NULL
         ORDER BY favorited DESC, updated_at DESC
         LIMIT ? OFFSET ?",
    ))
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn search_entries(pool: &SqlitePool, query: &str) -> Result<Vec<EntryMeta>, sqlx::Error> {
    // 转义 LIKE 通配符
    let escaped = query.replace('%', "\\%").replace('_', "\\_");
    let pattern = format!("%{}%", escaped);
    sqlx::query_as::<_, EntryMeta>(&format!(
        "SELECT {ENTRY_META_COLS}
         FROM entries
         WHERE deleted_at IS NULL
           AND (title LIKE ?1 ESCAPE '\\' OR subtitle LIKE ?1 ESCAPE '\\' OR tags LIKE ?1 ESCAPE '\\')
         ORDER BY favorited DESC, updated_at DESC",
    ))
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
        sqlx::query_as("SELECT id FROM entries WHERE id = ? AND deleted_at IS NULL")
            .bind(entry_id)
            .fetch_optional(pool)
            .await?;
    Ok(result.is_some())
}

/// 回收站保留天数
pub const TRASH_RETENTION_SECS: i64 = 30 * 24 * 60 * 60;

pub async fn purge_expired_trash(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let cutoff = chrono::Utc::now().timestamp() - TRASH_RETENTION_SECS;
    sqlx::query("DELETE FROM entries WHERE deleted_at IS NOT NULL AND deleted_at < ?")
        .bind(cutoff)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_trash_entries(pool: &SqlitePool, limit: i64) -> Result<Vec<EntryMeta>, sqlx::Error> {
    purge_expired_trash(pool).await?;
    sqlx::query_as::<_, EntryMeta>(&format!(
        "SELECT {ENTRY_META_COLS}
         FROM entries WHERE deleted_at IS NOT NULL
         ORDER BY deleted_at DESC
         LIMIT ?",
    ))
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn soft_delete_entry(pool: &SqlitePool, entry_id: &str, deleted_at: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE entries SET deleted_at = ?, favorited = 0, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(deleted_at)
    .bind(deleted_at)
    .bind(entry_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn restore_entry(pool: &SqlitePool, entry_id: &str, restored_at: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE entries SET deleted_at = NULL, updated_at = ? WHERE id = ? AND deleted_at IS NOT NULL",
    )
    .bind(restored_at)
    .bind(entry_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn purge_entry(pool: &SqlitePool, entry_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM entries WHERE id = ? AND deleted_at IS NOT NULL")
        .bind(entry_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn empty_trash(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM entries WHERE deleted_at IS NOT NULL")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
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

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE config (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE groups (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, icon TEXT, color TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE entries (
                id TEXT PRIMARY KEY, group_id TEXT, entry_type TEXT NOT NULL,
                title TEXT NOT NULL, subtitle TEXT, tags TEXT,
                favorited INTEGER NOT NULL DEFAULT 0, sort_order INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE fields (
                id TEXT PRIMARY KEY, entry_id TEXT NOT NULL, field_key TEXT NOT NULL,
                field_type TEXT NOT NULL DEFAULT 'text', enc_value TEXT NOT NULL,
                is_sensitive INTEGER NOT NULL DEFAULT 1, sort_order INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE audit_log (
                id TEXT PRIMARY KEY, action TEXT NOT NULL, entry_id TEXT,
                field_key TEXT, metadata TEXT, occurred_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn test_get_config_returns_none_for_missing() {
        let pool = setup_db().await;
        let result = get_config(&pool, "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_set_and_get_config() {
        let pool = setup_db().await;
        set_config(&pool, "theme", "dark").await.unwrap();
        let result = get_config(&pool, "theme").await.unwrap();
        assert_eq!(result, Some("dark".to_string()));
    }

    #[tokio::test]
    async fn test_set_config_overwrites() {
        let pool = setup_db().await;
        set_config(&pool, "theme", "dark").await.unwrap();
        set_config(&pool, "theme", "light").await.unwrap();
        let result = get_config(&pool, "theme").await.unwrap();
        assert_eq!(result, Some("light".to_string()));
    }

    #[tokio::test]
    async fn test_list_groups_empty() {
        let pool = setup_db().await;
        let groups = list_groups(&pool).await.unwrap();
        assert!(groups.is_empty());
    }

    #[tokio::test]
    async fn test_list_groups_with_data() {
        let pool = setup_db().await;
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO groups (id, name, icon, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind("g1")
        .bind("Work")
        .bind(Some("briefcase"))
        .bind(0)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        let groups = list_groups(&pool).await.unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "Work");
    }

    #[tokio::test]
    async fn test_write_audit_log() {
        let pool = setup_db().await;
        write_audit_log(&pool, "unlock", None, None, None)
            .await
            .unwrap();
        // Verify it was written
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }
}
