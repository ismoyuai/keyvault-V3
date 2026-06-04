pub mod queries;
pub mod schema;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

/// 初始化 SQLCipher 数据库连接池并执行迁移
pub async fn init_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_path)
        .await?;

    // 执行迁移
    sqlx::query(include_str!("migrations/001_init.sql"))
        .execute(&pool)
        .await?;

    sqlx::query(include_str!("migrations/002_history.sql"))
        .execute(&pool)
        .await?;

    // 003: 软删除列（幂等，避免重复 ALTER 失败）
    let has_deleted_at: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('entries') WHERE name = 'deleted_at'",
    )
    .fetch_one(&pool)
    .await?;
    if has_deleted_at == 0 {
        sqlx::query("ALTER TABLE entries ADD COLUMN deleted_at INTEGER")
            .execute(&pool)
            .await?;
    }
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_entries_deleted_at ON entries(deleted_at)",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
