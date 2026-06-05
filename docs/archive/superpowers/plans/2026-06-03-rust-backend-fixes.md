# Rust 后端审查修复实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 Rust 后端审查中发现的全部 5 Critical + 8 Important + 4 Minor 问题

**Architecture:** 按优先级分 4 阶段：Phase 1 阻塞应用运行的 Critical 问题，Phase 2 数据一致性 Critical 问题，Phase 3 安全加固 Important 问题，Phase 4 质量改进

**Tech Stack:** Rust, sqlx, tokio, zeroize, rand, tauri

---

## Phase 1: 阻塞应用运行的 Critical 问题

### Task 1: 实现 write_audit_log 函数 (C-1)

**Files:**
- Modify: `src-tauri/src/db/queries.rs`

- [ ] **Step 1: 实现 write_audit_log**

在 `src-tauri/src/db/queries.rs` 末尾添加：

```rust
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
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/db/queries.rs
git commit -m "fix: implement write_audit_log function (C-1)"
```

---

### Task 2: 创建 window 命令模块 (C-5)

**Files:**
- Create: `src-tauri/src/commands/window.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 window.rs**

```rust
// src-tauri/src/commands/window.rs
use tauri::Manager;

#[tauri::command]
pub async fn minimize_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_maximize(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_maximized().unwrap_or(false) {
            window.unmaximize().map_err(|e| e.to_string())?;
        } else {
            window.maximize().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn close_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

- [ ] **Step 2: 更新 commands/mod.rs**

```rust
pub mod auth;
pub mod breach;
pub mod clipboard_cmd;
pub mod export_cmd;
pub mod generator;
pub mod groups;
pub mod native_ext;
pub mod settings;
pub mod vault;
pub mod window;
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/commands/window.rs src-tauri/src/commands/mod.rs
git commit -m "feat: add window control commands (C-5 part 1)"
```

---

### Task 3: 创建 clipboard_cmd 命令模块 (C-5)

**Files:**
- Create: `src-tauri/src/commands/clipboard_cmd.rs`

- [ ] **Step 1: 创建 clipboard_cmd.rs**

```rust
// src-tauri/src/commands/clipboard_cmd.rs
use tauri::Manager;

#[tauri::command]
pub async fn copy_to_clipboard(app: tauri::AppHandle, text: String) -> Result<(), String> {
    use tauri::ClipboardKind;
    app.clipboard()
        .write_text(text, ClipboardKind::Clipboard)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clear_clipboard(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::ClipboardKind;
    app.clipboard()
        .write_text(String::new(), ClipboardKind::Clipboard)
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/clipboard_cmd.rs
git commit -m "feat: add clipboard commands (C-5 part 2)"
```

---

### Task 4: 创建 settings 命令模块 (C-5)

**Files:**
- Create: `src-tauri/src/commands/settings.rs`

- [ ] **Step 1: 创建 settings.rs**

```rust
// src-tauri/src/commands/settings.rs
use tauri::State;
use crate::db::queries;
use crate::state::AppState;

#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    queries::get_config(&state.db, &key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    session_token: String,
    key: String,
    value: String,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    queries::set_config(&state.db, &key, &value)
        .await
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/settings.rs
git commit -m "feat: add settings commands (C-5 part 3)"
```

---

### Task 5: 创建 groups 命令模块 (C-5)

**Files:**
- Create: `src-tauri/src/commands/groups.rs`

- [ ] **Step 1: 创建 groups.rs**

```rust
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
    let rows = queries::list_groups(&state.db)
        .await
        .map_err(|e| e.to_string())?;
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
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;
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
    let now = chrono::Utc::now().timestamp();
    sqlx::query("UPDATE groups SET name = ?, icon = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(&icon)
        .bind(now)
        .bind(&group_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
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
    sqlx::query("DELETE FROM groups WHERE id = ?")
        .bind(&group_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/groups.rs
git commit -m "feat: add group CRUD commands (C-5 part 4)"
```

---

### Task 6: 创建 export_cmd 命令模块 (C-5)

**Files:**
- Create: `src-tauri/src/commands/export_cmd.rs`

- [ ] **Step 1: 创建 export_cmd.rs**

```rust
// src-tauri/src/commands/export_cmd.rs
use tauri::State;
use serde::{Serialize, Deserialize};
use crate::state::AppState;
use crate::crypto::cipher;
use crate::db::queries;

#[derive(Serialize)]
struct ExportEntry {
    entry_type: String,
    title: String,
    subtitle: Option<String>,
    tags: Option<String>,
    favorited: bool,
    group_name: Option<String>,
    fields: Vec<ExportField>,
}

#[derive(Serialize)]
struct ExportField {
    field_key: String,
    field_type: String,
    value: String,
    is_sensitive: bool,
}

#[derive(Serialize)]
struct ExportData {
    version: String,
    exported_at: i64,
    entries: Vec<ExportEntry>,
}

#[tauri::command]
pub async fn export_vault(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<String, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let entries = queries::list_entries(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let groups = queries::list_groups(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let group_map: std::collections::HashMap<String, String> = groups
        .into_iter()
        .map(|g| (g.id, g.name))
        .collect();

    let mut export_entries = Vec::new();

    for entry in entries {
        let fields = queries::get_entry_fields(&state.db, &entry.id)
            .await
            .map_err(|e| e.to_string())?;

        let mut export_fields = Vec::new();
        for field in fields {
            let value = if field.is_sensitive != 0 {
                let decrypted = cipher::decrypt_field(key, &field.enc_value)
                    .map_err(|e| e.to_string())?;
                String::from_utf8(decrypted.to_vec())
                    .map_err(|_| "解码失败".to_string())?
            } else {
                field.enc_value
            };
            export_fields.push(ExportField {
                field_key: field.field_key,
                field_type: field.field_type,
                value,
                is_sensitive: field.is_sensitive != 0,
            });
        }

        export_entries.push(ExportEntry {
            entry_type: entry.entry_type,
            title: entry.title,
            subtitle: entry.subtitle,
            tags: entry.tags,
            favorited: entry.favorited != 0,
            group_name: entry.group_id.and_then(|gid| group_map.get(&gid).cloned()),
            fields: export_fields,
        });
    }

    let data = ExportData {
        version: "3.0.0".to_string(),
        exported_at: chrono::Utc::now().timestamp(),
        entries: export_entries,
    };

    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct ImportEntry {
    entry_type: String,
    title: String,
    subtitle: Option<String>,
    tags: Option<String>,
    favorited: Option<bool>,
    group_name: Option<String>,
    fields: Vec<ImportField>,
}

#[derive(Deserialize)]
struct ImportField {
    field_key: String,
    field_type: Option<String>,
    value: String,
    is_sensitive: Option<bool>,
}

#[derive(Deserialize)]
struct ImportData {
    entries: Vec<ImportEntry>,
}

#[tauri::command]
pub async fn import_vault(
    state: State<'_, AppState>,
    session_token: String,
    json: String,
) -> Result<usize, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let data: ImportData = serde_json::from_str(&json)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    let mut count = 0;

    for import_entry in data.entries {
        let entry_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let tags_json = import_entry.tags.unwrap_or_else(|| "[]".to_string());

        let group_id = if let Some(ref name) = import_entry.group_name {
            let groups = queries::list_groups(&state.db)
                .await
                .map_err(|e| e.to_string())?;
            groups.iter().find(|g| &g.name == name).map(|g| g.id.clone())
        } else {
            None
        };

        sqlx::query(
            "INSERT INTO entries (id, group_id, entry_type, title, subtitle, tags, favorited, sort_order, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)"
        )
        .bind(&entry_id)
        .bind(&group_id)
        .bind(&import_entry.entry_type)
        .bind(&import_entry.title)
        .bind(&import_entry.subtitle)
        .bind(&tags_json)
        .bind(import_entry.favorited.unwrap_or(false) as i32)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        for (i, import_field) in import_entry.fields.iter().enumerate() {
            let is_sensitive = import_field.is_sensitive.unwrap_or(true);
            let enc_value = if is_sensitive {
                cipher::encrypt_field(key, import_field.value.as_bytes())
                    .map_err(|e| e.to_string())?
            } else {
                import_field.value.clone()
            };
            let field_id = uuid::Uuid::new_v4().to_string();
            let field_type = import_field.field_type.clone().unwrap_or_else(|| "text".to_string());

            sqlx::query(
                "INSERT INTO fields (id, entry_id, field_key, field_type, enc_value, is_sensitive, sort_order)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&field_id)
            .bind(&entry_id)
            .bind(&import_field.field_key)
            .bind(&field_type)
            .bind(&enc_value)
            .bind(is_sensitive as i32)
            .bind(i as i32)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
        }

        count += 1;
    }

    Ok(count)
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/export_cmd.rs
git commit -m "feat: add export/import commands (C-5 part 5)"
```

---

### Task 7: 注册所有新命令到 lib.rs (C-5)

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 更新 generate_handler![]**

在 `src-tauri/src/lib.rs` 的 `generate_handler![]` 中添加所有缺失命令：

```rust
.invoke_handler(tauri::generate_handler![
    commands::auth::is_initialized,
    commands::auth::setup,
    commands::auth::unlock,
    commands::auth::lock,
    commands::auth::change_password,
    commands::vault::list_entries,
    commands::vault::search_entries,
    commands::vault::get_entry_secrets,
    commands::vault::create_entry,
    commands::vault::update_entry,
    commands::vault::delete_entry,
    commands::vault::toggle_favorite,
    commands::generator::generate_password,
    commands::breach::check_password_breach,
    commands::window::minimize_window,
    commands::window::toggle_maximize,
    commands::window::close_window,
    commands::clipboard_cmd::copy_to_clipboard,
    commands::clipboard_cmd::clear_clipboard,
    commands::settings::get_setting,
    commands::settings::set_setting,
    commands::groups::list_groups,
    commands::groups::create_group,
    commands::groups::update_group,
    commands::groups::delete_group,
    commands::export_cmd::export_vault,
    commands::export_cmd::import_vault,
])
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: register all 27 IPC commands in generate_handler (C-5)"
```

---

## Phase 2: 数据一致性 Critical 问题

### Task 8: setup 防重复初始化 (C-3)

**Files:**
- Modify: `src-tauri/src/commands/auth.rs`

- [ ] **Step 1: 在 setup 函数开头添加检查**

在 `src-tauri/src/commands/auth.rs` 的 `setup` 函数中，在 `let password_bytes = Zeroizing::new(password.into_bytes());` 之前添加：

```rust
    // 检查是否已初始化
    let existing = queries::get_config(&state.db, "password_hash")
        .await
        .map_err(|e| e.to_string())?;
    if existing.is_some() {
        return Err("密码管理器已初始化，请勿重复设置".to_string());
    }
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/auth.rs
git commit -m "fix: prevent duplicate setup in auth command (C-3)"
```

---

### Task 9: change_password 原子性 (C-4)

**Files:**
- Modify: `src-tauri/src/commands/auth.rs`

- [ ] **Step 1: 用事务包装数据库操作**

将 `change_password` 函数中的数据库操作用事务包装。替换从 `// 重新加密所有字段` 到 `// 更新内存中的密钥` 之间的代码：

```rust
    // 在事务中执行所有数据库操作
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    // 重新加密所有字段
    let fields = sqlx::query_as::<_, (String, String)>("SELECT id, enc_value FROM fields")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for (id, enc_value) in fields {
        let plaintext =
            crate::crypto::cipher::decrypt_field(old_key, &enc_value).map_err(|e| e.to_string())?;
        let new_enc =
            crate::crypto::cipher::encrypt_field(&new_key, &plaintext).map_err(|e| e.to_string())?;
        sqlx::query("UPDATE fields SET enc_value = ? WHERE id = ?")
            .bind(&new_enc)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 重新加密 field_history
    let history = sqlx::query_as::<_, (String, String)>("SELECT id, enc_value FROM field_history")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for (id, enc_value) in history {
        let plaintext =
            crate::crypto::cipher::decrypt_field(old_key, &enc_value).map_err(|e| e.to_string())?;
        let new_enc =
            crate::crypto::cipher::encrypt_field(&new_key, &plaintext).map_err(|e| e.to_string())?;
        sqlx::query("UPDATE field_history SET enc_value = ? WHERE id = ?")
            .bind(&new_enc)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 更新密码哈希和 salt
    let new_hash = kdf::hash_master_password(&new_bytes).map_err(|e| e.to_string())?;
    sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)")
        .bind("password_hash")
        .bind(&new_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)")
        .bind("kdf_salt")
        .bind(&hex::encode(new_salt))
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 提交事务
    tx.commit().await.map_err(|e| e.to_string())?;

    drop(key_guard);

    // 更新内存中的密钥
    *state.encryption_key.write().await = Some(new_key);
    *state.kdf_salt.write().await = Some(new_salt);
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/auth.rs
git commit -m "fix: make change_password atomic with transaction (C-4)"
```

---

### Task 10: update_entry 历史保存顺序修复 (I-7)

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`

- [ ] **Step 1: 修复 update_entry 中的历史保存逻辑**

将 `update_entry` 函数中的字段更新部分改为：先查询旧字段保存历史，再 DELETE，再 INSERT。

替换从 `// 删除旧字段，插入新字段` 到函数末尾的代码：

```rust
    // 先查询旧字段用于保存历史
    let old_fields = queries::get_entry_fields(&state.db, &entry_id)
        .await
        .unwrap_or_default();

    // 删除旧字段
    sqlx::query("DELETE FROM fields WHERE entry_id = ?")
        .bind(&entry_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    // 插入新字段并保存历史
    for (i, field) in input.fields.iter().enumerate() {
        // 保存旧值到历史
        for old in &old_fields {
            if old.field_key == field.field_key {
                let history_id = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO field_history (id, entry_id, field_key, enc_value, changed_at)
                     VALUES (?, ?, ?, ?, ?)",
                )
                .bind(&history_id)
                .bind(&entry_id)
                .bind(&old.field_key)
                .bind(&old.enc_value)
                .bind(now)
                .execute(&state.db)
                .await
                .ok();
            }
        }

        let enc_value =
            cipher::encrypt_field(key, field.value.as_bytes()).map_err(|e| e.to_string())?;
        let field_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO fields (id, entry_id, field_key, field_type, enc_value, is_sensitive, sort_order)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&field_id)
        .bind(&entry_id)
        .bind(&field.field_key)
        .bind(&field.field_type)
        .bind(&enc_value)
        .bind(field.is_sensitive as i32)
        .bind(i as i32)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/vault.rs
git commit -m "fix: correct update_entry history save order (I-7)"
```

---

## Phase 3: 安全加固 Important 问题

### Task 11: lock() 原子性修复 (C-2)

**Files:**
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: 修改 lock() 方法顺序**

将 `lock()` 方法改为先清零密钥再销毁 session：

```rust
    pub async fn lock(&self) {
        // 先清零加密密钥（高优先级）
        let mut key = self.encryption_key.write().await;
        *key = None;
        drop(key); // 释放写锁

        // 再销毁所有会话
        self.sessions.destroy_all().await;

        // 清零 salt
        let mut salt = self.kdf_salt.write().await;
        *salt = None;
    }
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/state.rs
git commit -m "fix: make lock() atomic - zero key before destroying sessions (C-2)"
```

---

### Task 12: session token 改用 OsRng + 清零 token (I-1, I-2)

**Files:**
- Modify: `src-tauri/src/crypto/session.rs`

- [ ] **Step 1: 修改 session.rs**

替换整个文件内容：

```rust
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::Rng;
use tokio::sync::RwLock;
use zeroize::Zeroizing;

const SESSION_TTL_SECS: u64 = 1800; // 30 分钟

pub struct SessionManager {
    sessions: RwLock<HashMap<String, SystemTime>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create(&self) -> String {
        let token: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let mut sessions = self.sessions.write().await;
        sessions.insert(token.clone(), SystemTime::now());
        token
    }

    pub async fn validate(&self, token: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        match sessions.get_mut(token) {
            Some(created_at) => {
                if created_at
                    .elapsed()
                    .unwrap_or_default()
                    > Duration::from_secs(SESSION_TTL_SECS)
                {
                    sessions.remove(token);
                    false
                } else {
                    // 滑动窗口：刷新时间
                    *created_at = SystemTime::now();
                    true
                }
            }
            None => false,
        }
    }

    pub async fn destroy_all(&self) {
        let mut sessions = self.sessions.write().await;
        // 清零每个 token 的内存
        for (token, _) in sessions.drain() {
            let mut token_bytes = Zeroizing::new(token.into_bytes());
            for b in token_bytes.iter_mut() {
                *b = 0;
            }
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/crypto/session.rs
git commit -m "fix: use OsRng for session tokens + zeroize on destroy_all (I-1, I-2)"
```

---

### Task 13: kdf_salt 用 Zeroizing 包装 (I-4)

**Files:**
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/commands/auth.rs`

- [ ] **Step 1: 修改 state.rs 中 kdf_salt 类型**

```rust
pub struct AppState {
    /// 加密密钥，仅在解锁后存在，锁定时清零
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
    /// SQLCipher 连接池（数据库始终保持加密）
    pub db: SqlitePool,
    /// 会话令牌管理器
    pub sessions: SessionManager,
    /// KDF 的 salt（存储在 DB 的 config 表中）
    pub kdf_salt: RwLock<Option<Zeroizing<[u8; 16]>>>,
}
```

- [ ] **Step 2: 更新 auth.rs 中的 salt 赋值**

在 `setup` 和 `unlock` 函数中，将 `*state.kdf_salt.write().await = Some(salt);` 改为：

```rust
    *state.kdf_salt.write().await = Some(Zeroizing::new(salt));
```

在 `change_password` 中同样修改。

- [ ] **Step 3: 更新 lib.rs 初始化**

```rust
            let app_state = AppState {
                encryption_key: RwLock::new(None),
                db: pool,
                sessions: crypto::session::SessionManager::new(),
                kdf_salt: RwLock::new(None),
            };
```

- [ ] **Step 4: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/state.rs src-tauri/src/commands/auth.rs
git commit -m "fix: wrap kdf_salt with Zeroizing for memory safety (I-4)"
```

---

### Task 14: AppError::Database 隐藏 SQL 细节 (I-5)

**Files:**
- Modify: `src-tauri/src/error.rs`

- [ ] **Step 1: 修改 AppError::Database 实现**

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误")]
    Database(String),
    #[error("加密错误: {0}")]
    Crypto(#[from] CryptoError),
    #[error("会话已过期")]
    SessionExpired,
    #[error("密码管理器已锁定")]
    Locked,
    #[error("密码管理器未初始化")]
    NotInitialized,
    #[error("密码错误")]
    WrongPassword,
    #[error("网络请求失败: {0}")]
    Network(String),
    #[error("{0}")]
    BadRequest(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        // 不泄露 SQL 细节，只记录内部日志
        tracing::error!("数据库错误: {:?}", e);
        AppError::Database("数据库操作失败".to_string())
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/error.rs
git commit -m "fix: hide SQL details in AppError serialization (I-5)"
```

---

### Task 15: LIKE 通配符转义 (I-3)

**Files:**
- Modify: `src-tauri/src/db/queries.rs`

- [ ] **Step 1: 修改 search_entries 函数**

```rust
pub async fn search_entries(pool: &SqlitePool, query: &str) -> Result<Vec<EntryMeta>, sqlx::Error> {
    // 转义 LIKE 通配符
    let escaped = query.replace('%', "\\%").replace('_', "\\_");
    let pattern = format!("%{}%", escaped);
    sqlx::query_as::<_, EntryMeta>(
        "SELECT id, entry_type, title, subtitle, tags, favorited, group_id, updated_at
         FROM entries
         WHERE title LIKE ?1 ESCAPE '\\' OR subtitle LIKE ?1 ESCAPE '\\' OR tags LIKE ?1 ESCAPE '\\'
         ORDER BY favorited DESC, updated_at DESC",
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/db/queries.rs
git commit -m "fix: escape LIKE wildcards in search_entries (I-3)"
```

---

## Phase 4: 质量改进

### Task 16: 创建系统托盘 (I-8)

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 tray.rs**

```rust
// src-tauri/src/tray.rs
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let lock_item = MenuItem::with_id(app, "lock", "锁定", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &lock_item, &quit_item])?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("KeyVault")
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "lock" => {
                let state = app.state::<crate::state::AppState>();
                let rt = tokio::runtime::Handle::current();
                rt.block_on(state.lock());
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

- [ ] **Step 2: 在 lib.rs 中注册 tray**

在 `lib.rs` 的 `mod` 声明中添加 `mod tray;`，在 setup 闭包中添加：

```rust
    tray::setup_tray(app).expect("无法初始化系统托盘");
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "feat: add system tray with show/lock/quit menu (I-8)"
```

---

### Task 17: 查询分页支持 (M-1)

**Files:**
- Modify: `src-tauri/src/db/queries.rs`

- [ ] **Step 1: 添加分页参数**

```rust
pub async fn list_entries(pool: &SqlitePool, limit: i64, offset: i64) -> Result<Vec<EntryMeta>, sqlx::Error> {
    sqlx::query_as::<_, EntryMeta>(
        "SELECT id, entry_type, title, subtitle, tags, favorited, group_id, updated_at
         FROM entries ORDER BY favorited DESC, updated_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
```

- [ ] **Step 2: 更新调用方**

在 `commands/vault.rs` 中更新 `list_entries` 调用：

```rust
    let rows = queries::list_entries(&state.db, 100, 0)
        .await
        .map_err(|e| e.to_string())?;
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/db/queries.rs src-tauri/src/commands/vault.rs
git commit -m "feat: add pagination support to list_entries (M-1)"
```

---

### Task 18: 统一错误消息为中文 (M-4)

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`

- [ ] **Step 1: 统一错误消息**

在 `vault.rs` 中，将所有 `.map_err(|e| e.to_string())` 改为中文错误消息：

```rust
    .map_err(|e| {
        tracing::error!("操作失败: {:?}", e);
        "操作失败，请重试".to_string()
    })?;
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/vault.rs
git commit -m "fix: unify error messages to Chinese (M-4)"
```

---

## 最终验证

### Task 19: 全面编译检查

- [ ] **Step 1: 运行 cargo check**

Run: `cd src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 2: 运行 cargo test**

Run: `cd src-tauri && cargo test`
Expected: 所有测试通过

- [ ] **Step 3: 运行 npm type-check**

Run: `npm run type-check`
Expected: TypeScript 编译通过

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "chore: complete Rust backend audit fixes (5 Critical, 8 Important, 4 Minor)"
```
