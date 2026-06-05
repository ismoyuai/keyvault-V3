# Phase 4: Remaining Features Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement remaining planned features: data export/import, browser extension save-prompt, change password UI, and theme switching.

**Architecture:** Data export uses Rust `serde_json` to serialize the vault, import parses JSON/CSV. The save-prompt content script detects form submissions and offers to save credentials.

**Tech Stack:** Rust, serde_json, Vue 3, TypeScript

---

### Task 1: Data Export Command

**Files:**
- Create: `src-tauri/src/commands/export_cmd.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create export_cmd.rs**

```rust
// src-tauri/src/commands/export_cmd.rs
use tauri::State;
use serde::Serialize;
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

/// Export all vault data as JSON (decrypted). Requires session token.
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
                field.enc_value // Non-sensitive fields stored as plaintext
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
```

- [ ] **Step 2: Add module and register command**

Add `pub mod export_cmd;` to `src-tauri/src/commands/mod.rs`. Add to `generate_handler![]`:
```rust
commands::export_cmd::export_vault,
```

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/export_cmd.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add vault export command (JSON, decrypted)"
```

---

### Task 2: Data Import Command

**Files:**
- Modify: `src-tauri/src/commands/export_cmd.rs`

- [ ] **Step 1: Add import structures and command**

Append to `src-tauri/src/commands/export_cmd.rs`:

```rust
use serde::Deserialize;

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

/// Import vault data from JSON. Requires session token.
/// Returns the number of entries imported.
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

        // Resolve group_id from group_name
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

- [ ] **Step 2: Register import command**

Add to `generate_handler![]` in `lib.rs`:
```rust
commands::export_cmd::import_vault,
```

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/export_cmd.rs src-tauri/src/lib.rs
git commit -m "feat: add vault import command (JSON format)"
```

---

### Task 3: Export/Import Frontend Bridge

**Files:**
- Modify: `src/bridge/tauri.ts`

- [ ] **Step 1: Add export/import to bridge**

Add to `src/bridge/tauri.ts`:

```typescript
// ============================================
// 数据导入/导出
// ============================================
export const data = {
  export: () => invoke<string>('export_vault', { sessionToken: getToken() }),
  import: (json: string) => invoke<number>('import_vault', { sessionToken: getToken(), json }),
}
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npm run type-check`

- [ ] **Step 3: Commit**

```bash
git add src/bridge/tauri.ts
git commit -m "feat: add export/import bridge calls"
```

---

### Task 4: Export/Import UI in Settings

**Files:**
- Modify: `src/views/SettingsView.vue`

- [ ] **Step 1: Add export/import buttons to SettingsView.vue**

Add a new section to the settings view:

```vue
<script setup>
import { data } from '@/bridge/tauri'
import { useToast } from '@/composables/useToast'

const toast = useToast()

async function handleExport() {
  try {
    const json = await data.export()
    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `keyvault-export-${new Date().toISOString().slice(0, 10)}.json`
    a.click()
    URL.revokeObjectURL(url)
    toast.success('导出成功')
  } catch (e: any) {
    toast.error('导出失败: ' + e.message)
  }
}

async function handleImport() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = async () => {
    const file = input.files?.[0]
    if (!file) return
    try {
      const text = await file.text()
      const count = await data.import(text)
      toast.success(`成功导入 ${count} 个条目`)
    } catch (e: any) {
      toast.error('导入失败: ' + e.message)
    }
  }
  input.click()
}
</script>

<!-- Add to template after existing settings sections: -->
<div class="settings-section">
  <h3>数据管理</h3>
  <div class="setting-row">
    <div>
      <div class="setting-label">导出数据</div>
      <div class="setting-desc">将所有密码导出为 JSON 文件（明文）</div>
    </div>
    <KvButton variant="secondary" @click="handleExport">导出</KvButton>
  </div>
  <div class="setting-row">
    <div>
      <div class="setting-label">导入数据</div>
      <div class="setting-desc">从 JSON 文件导入密码</div>
    </div>
    <KvButton variant="secondary" @click="handleImport">导入</KvButton>
  </div>
</div>
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npm run type-check`

- [ ] **Step 3: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "feat: add export/import UI to settings view"
```

---

### Task 5: Browser Extension Save-Prompt

**Files:**
- Create: `extension/content/save-prompt.js`
- Modify: `extension/manifest.json`

- [ ] **Step 1: Create save-prompt.js**

```javascript
// extension/content/save-prompt.js
// Detects form submissions with credentials and offers to save them.

(function () {
  'use strict';

  let pendingCredentials = null;

  // Listen for form submissions
  document.addEventListener('submit', (e) => {
    const form = e.target;
    if (!(form instanceof HTMLFormElement)) return;

    const passwordInput = form.querySelector('input[type="password"]');
    if (!passwordInput || !passwordInput.value) return;

    // Find username
    const usernameInput =
      form.querySelector('input[type="email"]') ||
      form.querySelector('input[autocomplete="username"]') ||
      form.querySelector('input[name*="user" i]') ||
      form.querySelector('input[name*="email" i]') ||
      form.querySelector('input[type="text"]');

    const username = usernameInput?.value || '';
    const password = passwordInput.value;

    if (!password) return;

    pendingCredentials = { username, password, url: location.href };

    // Ask service worker if we should save
    chrome.runtime.sendMessage(
      { action: 'CHECK_SAVE', url: location.href, username },
      (response) => {
        if (response?.shouldSave) {
          showSavePrompt(pendingCredentials);
        }
      }
    );
  });

  function showSavePrompt(creds) {
    // Remove existing prompt if any
    const existing = document.getElementById('kv-save-prompt');
    if (existing) existing.remove();

    const banner = document.createElement('div');
    banner.id = 'kv-save-prompt';
    banner.style.cssText = `
      position: fixed;
      top: 0;
      left: 50%;
      transform: translateX(-50%);
      z-index: 2147483647;
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 10px 16px;
      background: #1c2128;
      border: 1px solid rgba(255,255,255,0.1);
      border-radius: 0 0 8px 8px;
      box-shadow: 0 4px 12px rgba(0,0,0,0.4);
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      font-size: 13px;
      color: #e6edf3;
      animation: kv-slide-down 200ms ease-out;
    `;

    banner.innerHTML = `
      <span style="font-weight:500;">🔐 KeyVault</span>
      <span style="color:#8b949e;">是否保存此密码？</span>
      <button id="kv-save-yes" style="
        padding: 4px 12px; border: none; border-radius: 4px;
        background: #388bfd; color: white; cursor: pointer; font-size: 12px;
      ">保存</button>
      <button id="kv-save-no" style="
        padding: 4px 12px; border: none; border-radius: 4px;
        background: transparent; color: #8b949e; cursor: pointer; font-size: 12px;
      ">不用</button>
    `;

    // Add animation keyframes
    if (!document.getElementById('kv-save-style')) {
      const style = document.createElement('style');
      style.id = 'kv-save-style';
      style.textContent = `@keyframes kv-slide-down { from { transform: translateX(-50%) translateY(-100%); opacity: 0; } to { transform: translateX(-50%) translateY(0); opacity: 1; } }`;
      document.head.appendChild(style);
    }

    document.body.appendChild(banner);

    document.getElementById('kv-save-yes').addEventListener('click', () => {
      chrome.runtime.sendMessage({
        action: 'SAVE_CREDENTIAL',
        url: creds.url,
        username: creds.username,
        password: creds.password,
        title: document.title || location.hostname,
      });
      banner.remove();
    });

    document.getElementById('kv-save-no').addEventListener('click', () => {
      banner.remove();
    });

    // Auto-dismiss after 10 seconds
    setTimeout(() => {
      if (banner.parentNode) banner.remove();
    }, 10000);
  }
})();
```

- [ ] **Step 2: Add save-prompt.js to manifest.json**

Update `extension/manifest.json` content_scripts to include `save-prompt.js`:

```json
"content_scripts": [
  {
    "matches": ["<all_urls>"],
    "js": ["content/detector.js", "content/filler.js", "content/save-prompt.js"],
    "run_at": "document_idle",
    "all_frames": false
  }
]
```

- [ ] **Step 3: Add CHECK_SAVE handler to service worker**

In `extension/background/service-worker.js`, add to the `handleMessage` switch:

```javascript
case 'CHECK_SAVE':
  // Always offer to save for now (could check if already saved)
  return { shouldSave: true }
```

- [ ] **Step 4: Commit**

```bash
git add extension/content/save-prompt.js extension/manifest.json extension/background/service-worker.js
git commit -m "feat: add browser extension save-prompt for new credentials"
```

---

### Task 6: Change Password UI in Settings

**Files:**
- Modify: `src/views/SettingsView.vue`

- [ ] **Step 1: Add change password section to SettingsView.vue**

```vue
<script setup>
import KvModal from '@/components/ui/KvModal.vue'
import KvInput from '@/components/ui/KvInput.vue'
import { auth } from '@/bridge/tauri'

const showChangePassword = ref(false)
const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const changingPassword = ref(false)
const passwordError = ref('')

async function handleChangePassword() {
  passwordError.value = ''
  if (newPassword.value !== confirmPassword.value) {
    passwordError.value = '两次输入的密码不一致'
    return
  }
  if (newPassword.value.length < 8) {
    passwordError.value = '密码至少 8 个字符'
    return
  }
  changingPassword.value = true
  try {
    await auth.changePassword(oldPassword.value, newPassword.value)
    showChangePassword.value = false
    oldPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
    toast.success('密码已更改')
  } catch (e: any) {
    passwordError.value = e.message || '更改失败'
  } finally {
    changingPassword.value = false
  }
}
</script>

<!-- Add to template: -->
<div class="settings-section">
  <h3>安全</h3>
  <div class="setting-row">
    <div>
      <div class="setting-label">更改主密码</div>
      <div class="setting-desc">更改后所有数据将用新密码重新加密</div>
    </div>
    <KvButton variant="secondary" @click="showChangePassword = true">更改</KvButton>
  </div>
</div>

<KvModal v-model:visible="showChangePassword" title="更改主密码">
  <div style="display:flex;flex-direction:column;gap:12px;">
    <KvInput v-model="oldPassword" label="当前密码" type="password" />
    <KvInput v-model="newPassword" label="新密码" type="password" />
    <KvInput v-model="confirmPassword" label="确认新密码" type="password" :error="passwordError" />
    <KvButton variant="primary" :loading="changingPassword" @click="handleChangePassword">
      确认更改
    </KvButton>
  </div>
</KvModal>
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npm run type-check`

- [ ] **Step 3: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "feat: add change password UI to settings"
```
