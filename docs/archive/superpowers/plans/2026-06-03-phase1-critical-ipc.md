# Phase 1: Critical IPC Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement all Rust IPC commands that the frontend bridge (`src/bridge/tauri.ts`) calls but don't exist yet, making the app fully functional.

**Architecture:** Each command module follows the existing pattern: `#[tauri::command]` functions that take `State<'_, AppState>`, validate session tokens where needed, and return `Result<T, String>`. Commands are registered in `lib.rs` via `tauri::generate_handler![]`.

**Tech Stack:** Rust, Tauri 2.0, sqlx, tokio

---

## File Structure

```
src-tauri/src/
├── commands/
│   ├── mod.rs              # Add new module declarations
│   ├── window.rs           # NEW: minimize, maximize, close
│   ├── clipboard_cmd.rs    # NEW: copy_to_clipboard, clear_clipboard
│   ├── settings.rs         # NEW: get_setting, set_setting
│   └── groups.rs           # NEW: list_groups, create_group, update_group, delete_group
├── lib.rs                  # Register new commands in generate_handler![]
└── tray.rs                 # NEW: system tray setup
```

---

### Task 1: Window Control Commands

**Files:**
- Create: `src-tauri/src/commands/window.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create window.rs with minimize, maximize, close commands**

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

- [ ] **Step 2: Add module declaration to commands/mod.rs**

Add `pub mod window;` to `src-tauri/src/commands/mod.rs`.

- [ ] **Step 3: Register commands in lib.rs**

Add to the `generate_handler![]` array in `src-tauri/src/lib.rs`:
```rust
commands::window::minimize_window,
commands::window::toggle_maximize,
commands::window::close_window,
```

- [ ] **Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/window.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add window control IPC commands (minimize, maximize, close)"
```

---

### Task 2: Clipboard Commands

**Files:**
- Create: `src-tauri/src/commands/clipboard_cmd.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`

- [ ] **Step 1: Create clipboard_cmd.rs**

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

- [ ] **Step 2: Add module declaration to commands/mod.rs**

Add `pub mod clipboard_cmd;` to `src-tauri/src/commands/mod.rs`.

- [ ] **Step 3: Register commands in lib.rs**

Add to the `generate_handler![]` array:
```rust
commands::clipboard_cmd::copy_to_clipboard,
commands::clipboard_cmd::clear_clipboard,
```

- [ ] **Step 4: Add clipboard permission to capabilities**

Add `"core:clipboard:allow-write-text"` and `"core:clipboard:allow-read-text"` to `src-tauri/capabilities/default.json` permissions array. If these permissions don't exist in Tauri 2.0, the clipboard API may work without them — verify by testing.

- [ ] **Step 5: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/clipboard_cmd.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/capabilities/default.json
git commit -m "feat: add clipboard IPC commands (copy, clear)"
```

---

### Task 3: Settings Commands

**Files:**
- Create: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create settings.rs**

```rust
// src-tauri/src/commands/settings.rs
use tauri::State;
use crate::db::queries;
use crate::state::AppState;

/// Get a setting value. Returns null if not set.
/// Does NOT require session (non-sensitive settings like theme, auto-lock timeout).
#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    queries::get_config(&state.db, &key)
        .await
        .map_err(|e| e.to_string())
}

/// Set a setting value. Requires session token.
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

- [ ] **Step 2: Add module declaration and register commands**

Add `pub mod settings;` to `src-tauri/src/commands/mod.rs`. Add to `generate_handler![]`:
```rust
commands::settings::get_setting,
commands::settings::set_setting,
```

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/settings.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add settings IPC commands (get_setting, set_setting)"
```

---

### Task 4: Group CRUD Commands

**Files:**
- Create: `src-tauri/src/commands/groups.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create groups.rs**

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
    // Entries in this group will have group_id set to NULL (ON DELETE SET NULL)
    sqlx::query("DELETE FROM groups WHERE id = ?")
        .bind(&group_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 2: Add module declaration and register commands**

Add `pub mod groups;` to `src-tauri/src/commands/mod.rs`. Add to `generate_handler![]`:
```rust
commands::groups::list_groups,
commands::groups::create_group,
commands::groups::update_group,
commands::groups::delete_group,
```

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/groups.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add group CRUD IPC commands"
```

---

### Task 5: Audit Log Helper

**Files:**
- Modify: `src-tauri/src/db/queries.rs`

- [ ] **Step 1: Add audit_log helper function to queries.rs**

Append to `src-tauri/src/db/queries.rs`:

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
        "INSERT INTO audit_log (id, action, entry_id, field_key, metadata, occurred_at) VALUES (?, ?, ?, ?, ?, ?)"
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

- [ ] **Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/queries.rs
git commit -m "feat: add audit_log helper to queries"
```

---

### Task 6: System Tray

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create tray.rs**

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

- [ ] **Step 2: Wire tray into lib.rs**

In the `setup` closure of `src-tauri/src/lib.rs`, add after `app_handle.manage(app_state);`:
```rust
tray::setup_tray(app).expect("无法初始化系统托盘");
```

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "feat: add system tray with show/lock/quit menu"
```

---

### Task 7: Final Integration Verification

- [ ] **Step 1: Full compilation check**

Run: `cd src-tauri && cargo check`
Expected: Compiles with only the existing unused-code warnings.

- [ ] **Step 2: Verify all frontend bridge calls have matching Rust commands**

Check that every `invoke<...>('command_name')` in `src/bridge/tauri.ts` has a corresponding `#[tauri::command]` registered in `generate_handler![]`.

Expected mapping:
| Frontend invoke | Rust command |
|---|---|
| `is_initialized` | `commands::auth::is_initialized` |
| `setup` | `commands::auth::setup` |
| `unlock` | `commands::auth::unlock` |
| `lock` | `commands::auth::lock` |
| `change_password` | `commands::auth::change_password` |
| `list_entries` | `commands::vault::list_entries` |
| `search_entries` | `commands::vault::search_entries` |
| `get_entry_secrets` | `commands::vault::get_entry_secrets` |
| `create_entry` | `commands::vault::create_entry` |
| `update_entry` | `commands::vault::update_entry` |
| `delete_entry` | `commands::vault::delete_entry` |
| `toggle_favorite` | `commands::vault::toggle_favorite` |
| `generate_password` | `commands::generator::generate_password` |
| `check_password_breach` | `commands::breach::check_password_breach` |
| `list_groups` | `commands::groups::list_groups` |
| `create_group` | `commands::groups::create_group` |
| `update_group` | `commands::groups::update_group` |
| `delete_group` | `commands::groups::delete_group` |
| `minimize_window` | `commands::window::minimize_window` |
| `toggle_maximize` | `commands::window::toggle_maximize` |
| `close_window` | `commands::window::close_window` |
| `copy_to_clipboard` | `commands::clipboard_cmd::copy_to_clipboard` |
| `clear_clipboard` | `commands::clipboard_cmd::clear_clipboard` |
| `get_setting` | `commands::settings::get_setting` |
| `set_setting` | `commands::settings::set_setting` |
| `tray_lock` | NOT IMPLEMENTED (tray handles via menu directly) |
| `tray_show` | NOT IMPLEMENTED (tray handles via menu directly) |

- [ ] **Step 3: Remove tray_lock and tray_show from frontend bridge**

Since the tray handles lock/show directly via menu events (not IPC), remove the unused `tray` namespace from `src/bridge/tauri.ts`:

```typescript
// DELETE the tray section:
// export const tray = {
//   lock: () => invoke<void>('tray_lock'),
//   show: () => invoke<void>('tray_show'),
// }
```

- [ ] **Step 4: Verify TypeScript still compiles**

Run: `npm run type-check`
Expected: Clean compilation.

- [ ] **Step 5: Commit**

```bash
git add src/bridge/tauri.ts
git commit -m "fix: remove unused tray IPC calls (tray uses menu events directly)"
```
