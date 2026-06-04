use serde::Serialize;

/// 条目元数据（列表显示用，不含敏感数据）
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EntryMeta {
    pub id: String,
    pub entry_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Option<String>,
    pub favorited: i32,
    pub group_id: Option<String>,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

/// 加密字段行
#[derive(Debug, sqlx::FromRow)]
pub struct FieldRow {
    pub field_key: String,
    pub field_type: String,
    pub enc_value: String,
    pub is_sensitive: i32,
}

/// 分组
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GroupRow {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}
