-- 配置表（KDF salt、主密码哈希等）
CREATE TABLE IF NOT EXISTS config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 分组/文件夹
CREATE TABLE IF NOT EXISTS groups (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    icon       TEXT,
    color      TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 条目元数据（不含敏感字段）
CREATE TABLE IF NOT EXISTS entries (
    id          TEXT PRIMARY KEY,
    group_id    TEXT REFERENCES groups(id) ON DELETE SET NULL,
    entry_type  TEXT NOT NULL,
    title       TEXT NOT NULL,
    subtitle    TEXT,
    tags        TEXT,
    favorited   INTEGER NOT NULL DEFAULT 0,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

-- 加密字段（每个条目可有多个加密字段）
CREATE TABLE IF NOT EXISTS fields (
    id           TEXT PRIMARY KEY,
    entry_id     TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    field_key    TEXT NOT NULL,
    field_type   TEXT NOT NULL DEFAULT 'text',
    enc_value    TEXT NOT NULL,
    is_sensitive INTEGER NOT NULL DEFAULT 1,
    sort_order   INTEGER NOT NULL DEFAULT 0
);

-- 泄露检测缓存
CREATE TABLE IF NOT EXISTS breach_cache (
    hash_prefix TEXT PRIMARY KEY,
    result_json TEXT NOT NULL,
    cached_at   INTEGER NOT NULL
);

-- 审计日志
CREATE TABLE IF NOT EXISTS audit_log (
    id          TEXT PRIMARY KEY,
    action      TEXT NOT NULL,
    entry_id    TEXT,
    field_key   TEXT,
    metadata    TEXT,
    occurred_at INTEGER NOT NULL
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_entries_type      ON entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_entries_group     ON entries(group_id);
CREATE INDEX IF NOT EXISTS idx_entries_favorited ON entries(favorited);
CREATE INDEX IF NOT EXISTS idx_fields_entry      ON fields(entry_id);
