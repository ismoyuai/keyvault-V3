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

    Ok(pool)
}
