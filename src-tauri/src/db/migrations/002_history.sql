-- 密码历史记录（加密存储）
CREATE TABLE IF NOT EXISTS field_history (
    id          TEXT PRIMARY KEY,
    entry_id    TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    field_key   TEXT NOT NULL,
    enc_value   TEXT NOT NULL,
    changed_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_field_history_entry ON field_history(entry_id, field_key, changed_at);
