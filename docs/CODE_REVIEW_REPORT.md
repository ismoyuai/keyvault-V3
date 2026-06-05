# KeyVault v3 代码审查报告

> **用途：** 全项目审查发现汇总 + 修复任务清单。审查完成后按本文档逐项修复。  
> **审查基线：** commit `9534512`（`master`，工作区干净）  
> **最后更新：** 2026-06-05 — **全项目审查完成（Batch A–E）**

---

## 审查进度

| 批次 | 范围 | 状态 |
|------|------|------|
| **Batch A** | Phase 0 基线 + Crypto/State | ✅ 完成 |
| **Batch B** | DB + IPC 命令层 | ✅ 完成 |
| **Batch C** | 前端 Vue + Bridge | ✅ 完成 |
| **Batch D** | Sync / Export / Native Messaging / Extension / Capabilities | ✅ 完成 |
| **Batch E** | 测试缺口 + 验收对照 + Executive Summary | ✅ 完成 |

### 自动化基线（Phase 0）

| 检查项 | 结果 |
|--------|------|
| `cargo test` | 31 passed |
| `npm run type-check` | 通过 |
| `cargo audit` | 未安装 `cargo-audit` |
| 编译警告 | 3 项 dead code |

---

## 发现统计（A + B + C + D）

| 级别 | 数量 | 说明 |
|------|------|------|
| **Critical** | 7 | 发布阻塞、数据丢失、扩展架构缺陷 |
| **Important** | 18 | 安全加固、数据完整性、行为缺陷 |
| **Minor** | 11 | 质量、一致性、可维护性 |

---

## Critical（必须优先修复）

### C-1：SQLCipher 未启用，数据库文件明文落盘

- **批次：** A  
- **位置：** `src-tauri/Cargo.toml:30-32`，`src-tauri/src/db/mod.rs`  
- **现象：** 使用 `bundled` 而非 `bundled-sqlcipher`；`init_db()` 无 `PRAGMA key`  
- **影响：** 违反迁移文档红线 #10；`keyvault.db` 可被 `sqlite3` 直接读取（含 `password_hash`、`kdf_salt`、字段密文）  
- **修复建议：**
  1. `libsqlite3-sys` 切换 `bundled-sqlcipher`
  2. unlock 后派生 DB 子密钥（HKDF）并 `PRAGMA key`
  3. lock 时关闭/重连连接池
  4. 同步更新 `SettingsView.vue` 文案与迁移文档验收项
- **验证：** 锁定后用 `sqlite3 keyvault.db ".tables"` 应无法读取

---

### C-2：剪贴板 IPC 无 session 校验

- **批次：** B  
- **位置：** `src-tauri/src/commands/clipboard_cmd.rs`  
- **现象：** `copy_to_clipboard` / `clear_clipboard` 无 `session_token`  
- **影响：** 未解锁即可通过 IPC 写剪贴板；违反红线 #12  
- **前端关联：** `src/bridge/tauri.ts`（`clipboard.copy/clear`），`src/composables/useClipboard.ts`  
- **修复建议：** 两命令增加 `session_token` + `validate`；`tauri.ts` 的 `clipboard` 调用 `getToken()`  
- **验证：** 未解锁时 invoke 剪贴板命令应返回「会话已过期」

---

### C-3：`get_setting` 无 session，可读取敏感 config

- **批次：** B  
- **位置：** `src-tauri/src/commands/settings.rs:7-11`  
- **可读敏感键：** `password_hash`、`kdf_salt`、`sync_webdav_password`（旧版可能明文）  
- **前端关联：** `src/bridge/tauri.ts:180`，`src/stores/settings.ts:15-21`（解锁前 intentional 调用）  
- **修复建议：**
  - 方案 A：拆分 `get_public_setting`（白名单：`auto_lock_minutes`、`clipboard_clear_seconds`、`theme`）与需 session 的 `get_setting`
  - 方案 B：`get_setting` 统一 require session + 敏感键黑名单  
- **验证：** 未解锁时 `invoke('get_setting', { key: 'password_hash' })` 应失败

---

### C-4：编辑条目不加载已有字段，保存会清空数据

- **批次：** C  
- **位置：** `src/components/vault/ItemForm.vue:37-49`  
- **现象：** 打开编辑时 `fields.value = []`，未调用 `getEntrySecrets`  
- **影响：** 用户直接点保存 → `update_entry` 先 DELETE 全部字段再 INSERT → **数据永久丢失**（叠加 B-I-9 非事务风险）  
- **修复建议：**
  1. `editEntry` 打开时 `await vaultBridge.getEntrySecrets(id)` 填充 `fields`
  2. 加载完成前禁用保存按钮
  3. 关闭表单时 `resetForm()` 清空字段值
- **验证：** 编辑含密码的 SSH 条目 → 不修改直接保存 → 密码仍在

---

### C-5：Native Messaging Host 与 GUI 使用不同数据库路径

- **批次：** D  
- **位置：**  
  - GUI：`src-tauri/src/lib.rs` → `app_data_dir()/keyvault.db`（Tauri 标识 `com.keyvault.app`）  
  - Host：`src-tauri/src/native_messaging/host.rs:181-189` → `%APPDATA%/keyvault/keyvault.db`  
- **影响：** 扩展连接的是**另一份数据库**；GUI 解锁、建库与扩展查询互不相通  
- **修复建议：** 抽取统一的 `get_db_path()`，两处共用 Tauri `app_data_dir` 逻辑（或共享配置路径）  
- **验证：** 桌面端创建条目后，Native Host 同路径 DB 能查到该条目

---

### C-6：Native Messaging 进程无法获得解锁态（`encryption_key` 永为空）

- **批次：** D  
- **位置：** `src-tauri/src/native_messaging/host.rs:51-61`，`main.rs:4-5`  
- **现象：** `--native-messaging` 启动**独立进程**，自建 `AppState`，`encryption_key` 始终 `None`；与 GUI 进程内存不共享  
- **影响：**  
  - `get_status` 永远返回 `locked`（除非在 Host 进程内单独 unlock，当前无此逻辑）  
  - `get_entry_for_fill` 必然失败「密码管理器已锁定」  
  - 自动填充功能**不可用**  
- **修复建议（架构级，扩展恢复开发时必做）：**  
  1. 方案 A：扩展 IPC 转发到已解锁的 GUI 进程（单进程持钥）  
  2. 方案 B：安全共享解锁态（命名管道/本地 socket + 短期令牌，需威胁建模）  
  3. 禁止在独立 Host 进程重复 `init_db` 而不共享密钥  
- **验证：** 桌面端解锁后，扩展 popup 显示 `connected` 且能填充密码

---

### C-7：扩展调用的 Native 动作未实现

- **批次：** D  
- **位置：** `host.rs:107-178`（handler），`extension/background/service-worker.js:90-96`  
- **未实现 action：**  
  - `save_credential` — 保存新凭据（`SAVE_CREDENTIAL` 消息链）  
  - `open_app` — popup 打开主应用（`popup.js:24-27` 直接 `sendNativeMessage`）  
- **影响：** 保存提示、打开应用按钮无效；返回「未知操作」  
- **修复建议：** 在 Host 或 GUI 侧实现对应 action，或扩展改为仅调用已实现 action  
- **验证：** 提交登录表单 → 点保存 → 桌面端出现新条目

---

## Important

### I-1：KDF salt 生成使用 `thread_rng` 而非 `OsRng`

- **批次：** A  
- **位置：** `src-tauri/src/commands/auth.rs:71`，`change_password:206`  
- **修复：** 改为 `OsRng.fill_bytes`

---

### I-2：`lock()` 存在 key 与 session 短暂不一致窗口

- **批次：** A  
- **位置：** `src-tauri/src/state.rs:25-36`  
- **修复：** 评估先销毁 session 再清 key，或单锁保护

---

### I-3：Session `validate()` 使用写锁做读操作

- **批次：** A  
- **位置：** `src-tauri/src/crypto/session.rs:35-54`  
- **修复：** 读路径用读锁，刷新 TTL 再升级写锁

---

### I-4：缺少 Session TTL 过期单元测试

- **批次：** A  
- **位置：** `src-tauri/src/crypto/session.rs`  
- **修复：** 增加 1800s 过期测试

---

### I-5：`get_setting` 无 session（与 C-3 同源，记为后端 Important）

- **批次：** B（已在 C-3 覆盖修复）

---

### I-6：`lock` IPC 无 session 校验

- **批次：** B  
- **位置：** `src-tauri/src/commands/auth.rs:164-169`  
- **影响：** 任意 IPC 调用者可触发锁定（DoS）  
- **修复：** 要求有效 `session_token`（或接受为设计决策并文档化）

---

### I-7：元数据命令不检查 `encryption_key`

- **批次：** B  
- **位置：** `list_entries`、`search_entries` 等（`vault.rs`）  
- **修复：** 与解密命令一致，增加 `key_guard.as_ref().ok_or("密码管理器已锁定")?`

---

### I-8：`create_entry` / `import_vault` 无数据库事务

- **批次：** B  
- **位置：** `src-tauri/src/commands/vault.rs:201-247`，`export_cmd.rs:160-201`  
- **修复：** `sqlx::transaction` 包裹元数据 + 字段写入

---

### I-9：`update_entry` 非原子，中途失败可丢字段

- **批次：** B  
- **位置：** `src-tauri/src/commands/vault.rs:289-349`  
- **修复：** 事务包裹；或 upsert 策略避免先 DELETE  
- **关联：** 与 C-4 叠加时风险极高

---

### I-10：`export_vault` 静默截断 100 条 + IPC 契约不一致

- **批次：** B  
- **位置：** `src-tauri/src/commands/export_cmd.rs:46`  
- **前端：** `src/bridge/tauri.ts:117-118` 传 `format`，Rust 不接收  
- **修复：** 导出用 `list_all` 或分页；Rust 增加 `format` 或前端移除参数

---

### I-11：`check_password_breach` 密码未 `Zeroizing`

- **批次：** B  
- **位置：** `src-tauri/src/commands/breach.rs:18`  
- **前端关联：** `src/views/SettingsView.vue:43`（用后已 `checkPassword.value = ''`）  
- **修复：** Rust 侧 `Zeroizing<Vec<u8>>`

---

### I-12：部分命令 `e.to_string()` 泄露内部错误

- **批次：** B  
- **位置：** `auth.rs`、`export_cmd.rs`、`sync.rs`、`groups.rs` 等  
- **对比：** `vault.rs` 已用通用消息 + `tracing::error!`  
- **修复：** 命令层统一使用 `AppError`（`error.rs` 已对 sqlx 脱敏）

---

### I-13：锁定后 Pinia vault store 仍保留元数据

- **批次：** C  
- **位置：** `src/stores/auth.ts:136-144`（`lock` 未清 vault），`src/stores/vault.ts`  
- **影响：** 锁定后内存中仍有条目标题/副标题（不含解密字段，但违反「锁定即清空」的严格解释）  
- **修复：** `auth.lock()` 或 `useAutoLock.lock()` 中调用 `vault.$reset()` / 清空 `entries` 与 `selectedId`

---

### I-14：CommandPalette 锁定不清剪贴板

- **批次：** C  
- **位置：** `src/components/search/CommandPalette.vue:50-54`  
- **对比：** `useAutoLock.lock()` 会先 `clearClipboard()`  
- **修复：** CommandPalette 的 lock 动作与 `useAutoLock` 对齐

---

### I-15：同步文件仍兼容 v2 明文 JSON 降级解析

- **批次：** D  
- **位置：** `src-tauri/src/sync/engine.rs:107-122`  
- **现象：** v3 解密失败时回退 `serde_json::from_str::<SyncPayload>(content)` 明文解析  
- **影响：** 若远程 `data.kv` 被替换为伪造明文 JSON（含有效 checksum），可能注入恶意条目  
- **修复：** 生产环境拒绝非 v3 加密格式，或要求签名；迁移期单独开关  
- **验证：** 上传非加密 JSON 到 WebDAV → `sync_pull` 应拒绝

---

### I-16：前端 `fs` 插件直接读写导出文件，绕过 Rust

- **批次：** D  
- **位置：** `capabilities/default.json:17-18`，`src/views/SettingsView.vue:56-93`  
- **现象：** 导入/导出使用 `@tauri-apps/plugin-dialog` + `@tauri-apps/plugin-fs`，非 `export_cmd` 写文件  
- **影响：** 违反迁移文档红线 #11「不开启 `core:fs:*`，文件操作在 Rust 中」；扩大 WebView 文件 IO 攻击面  
- **修复：** 导出改为 Rust 命令写文件，或配置严格 `fs` scope + 仅允许 dialog 返回路径  
- **验证：** 审查 `capabilities` 中 fs scope 是否限制用户选择路径

---

### I-17：`capabilities/default.json` 权限偏宽

- **批次：** D  
- **位置：** `src-tauri/capabilities/default.json`  
- **权限：** `fs:allow-read/write-text-file` 无 scope 限制；`shell:allow-open` 可打开外部 URL  
- **修复：** 按 Tauri 2 能力系统添加最小 scope；评估 `shell:open` 是否必需

---

### I-18：Native Messaging 清单未配置（不可部署）

- **批次：** D  
- **位置：** `extension/com.keyvault.app.json`  
- **现象：** `"path": ""`，`allowed_origins` 为 `PLACEHOLDER_EXTENSION_ID`  
- **修复：** 构建时写入实际扩展 ID 与 Host 可执行文件路径（安装脚本/registry）

---

### I-19：扩展 popup 复制密码无自动清空

- **批次：** D  
- **位置：** `extension/popup/popup.js:129-136`  
- **对比：** 桌面端 `useClipboard` 30s 倒计时清空  
- **修复：** 扩展侧复制后定时清空，或禁止复制仅允许填充

---

### I-20：`CHECK_SAVE` 恒返回 `shouldSave: true`

- **批次：** D  
- **位置：** `extension/background/service-worker.js:86-88`  
- **影响：** 每次带密码表单提交都弹保存条，即使用户已保存过  
- **修复：** 查询 Host/GUI 是否已有同 URL 凭据后再提示

---

### I-21：WebDAV 凭据旧版明文兼容路径仍存在

- **批次：** D（与 B 重叠，D 侧重复记录）  
- **位置：** `src-tauri/src/commands/sync.rs:72-80`  
- **修复：** 读取时迁移为 `enc:` 格式；移除明文回退

---

### I-22：同步上传载荷含明文密码（传输前加密）

- **批次：** D  
- **位置：** `sync.rs` `export_sync_entries` → `engine::serialize_encrypted`  
- **评估：** 内存中短暂明文，WebDAV 上为 AES-GCM 密文 ✅；需确保 HTTPS WebDAV  
- **修复：** 文档要求 WebDAV 必须 HTTPS；可选证书固定

---

## Minor

### M-1：`verify_master_password` 使用 `Argon2::default()` — 可接受（PHC 内嵌参数）

- **批次：** A · `src-tauri/src/crypto/kdf.rs:44-46`

### M-2：`destroy_all` token 清零受 Rust 分配器限制

- **批次：** A · `src-tauri/src/crypto/session.rs:59-64`

### M-3：dead code 警告（`entry_exists`、`AppError`、`is_unlocked`）

- **批次：** A/B · 清理或标注预留

### M-4：`cargo audit` 未纳入 CI

- **批次：** A

### M-5：`create_entry`/`update_entry` 无审计日志

- **批次：** B · `vault.rs`

### M-6：`delete_group` 不检查关联条目（FK ON DELETE SET NULL，可接受）

- **批次：** B

### M-7：解锁页 `password` ref 卸载时未显式清空

- **批次：** C · `src/views/UnlockView.vue` — `onUnmounted` 可加 `password.value = ''`

### M-8：`/login` 在已解锁时不重定向到 `/vault`

- **批次：** C · `src/router/index.ts` — UX 小问题

### M-9：`PasswordGenerator` 关闭后 `password` ref 仍驻留内存

- **批次：** C · `src/components/generator/PasswordGenerator.vue` — 关闭时清空

### M-10：条目元数据（title/subtitle/tags）数据库明文存储

- **批次：** B — 设计取舍，需在文档标明

### M-11：`search_entries` 无 LIMIT

- **批次：** B · `queries.rs:36-49`

### M-12：`generate_password` Rust 侧使用 `thread_rng`

- **批次：** B · `generator.rs:81` — 与 I-1 同类

---

### M-13：扩展 `storage` 权限宽于实际使用

- **批次：** D · `extension/manifest.json:6` — 仅用 `chrome.storage.session` 存连接状态

### M-14：扩展 content_scripts 注入 `<all_urls>`

- **批次：** D · `manifest.json:22` — 攻击面大；可改为按需 `activeTab` + scripting

### M-15：`com.keyvault.app.json` / Host 注册文档缺失

- **批次：** D — 需补充 Windows 注册表安装说明

### M-16：`save-prompt.js` 中 `pendingCredentials` 10s 后未显式清零

- **批次：** D · 密码短期驻留 content script 内存

### M-17：`tauri.conf.json` CSP 含 `style-src 'unsafe-inline'`

- **批次：** D — Vue scoped style 常见取舍；可接受

### M-18：同步 `build_payload` version=2 与 `SYNC_ENCRYPTED_VERSION=3` 不一致

- **批次：** D · `sync/engine.rs:63` — 外层 blob version 3，内层 payload version 2，易混淆

---

## IPC Session 对照表（Batch B）

| 命令 | Session | encryption_key | 备注 |
|------|---------|----------------|------|
| `is_initialized` / `get_unlock_status` | — | — | 公开 |
| `setup` / `unlock` | — | 写入 | ✅ |
| `lock` | ❌ | 清零 | I-6 |
| `change_password` / `emergency_wipe` | ✅ | ✅ | ✅ |
| `vault::*`（11） | ✅ | 解密类 ✅ | 元数据类缺 key 检查 I-7 |
| `export_vault` / `import_vault` | ✅ | ✅ | I-10 |
| `generate_password` / `check_password_breach` | ✅ | — | I-11 |
| `copy/clear_clipboard` | ❌ | — | **C-2** |
| `get_setting` | ❌ | — | **C-3** |
| `set_setting` | ✅ | — | ✅ |
| `sync::*`（6） | ✅ | 写操作 ✅ | ✅ |
| `groups::*`（4） | ✅ | — | ✅ |
| `window::*`（3） | — | — | 窗口控制 |

---

## 前端安全红线对照（Batch C）

| 红线 | 状态 | 说明 |
|------|------|------|
| vault store 只存元数据 | ✅ | `stores/vault.ts` 无解密字段 |
| ItemDetail 卸载清空 secrets | ✅ | `onUnmounted` + watch 重置 |
| session token 不落盘 | ✅ | `tauri.ts` 模块变量，无 localStorage |
| 路由守卫 | ✅ | `requiresAuth` → `/login` |
| 锁定清 session token | ✅ | `clearSessionToken()` |
| 锁定清 vault 元数据 | ⚠️ | I-13 |
| 编辑表单不缓存 secrets 在 store | ✅ | 但未加载即 **C-4** |
| 剪贴板定时清空 | ✅ | `useClipboard` + `useAutoLock` blur |
| Pinia 持久化 | ✅ | 未使用 persist 插件 |

---

## 修复优先级路线图

### P0 — 发布阻塞（桌面端）

| ID | 任务 | 主要文件 |
|----|------|----------|
| C-1 | 启用 SQLCipher + PRAGMA key | `Cargo.toml`, `db/mod.rs`, `auth.rs` |
| C-2 | 剪贴板 session 校验 | `clipboard_cmd.rs`, `tauri.ts`, `useClipboard.ts` |
| C-3 | `get_setting` 敏感键门禁 | `settings.rs`, `tauri.ts`, `settings.ts` |
| C-4 | 编辑条目加载 secrets | `ItemForm.vue` |

### P0-ext — 扩展恢复开发时阻塞（当前暂停，先记录）

| ID | 任务 | 主要文件 |
|----|------|----------|
| C-5 | 统一 DB 路径 | `lib.rs`, `native_messaging/host.rs` |
| C-6 | GUI ↔ Host 解锁态共享架构 | `native_messaging/`, `main.rs` |
| C-7 | 实现 `save_credential` / `open_app` | `host.rs`, `native_ext.rs` |
| I-18 | 配置 `com.keyvault.app.json` | `extension/` + 安装脚本 |

### P1 — 数据完整性

| ID | 任务 | 主要文件 |
|----|------|----------|
| I-9 | `update_entry` 事务化 | `vault.rs` |
| I-8 | `create_entry` / `import_vault` 事务化 | `vault.rs`, `export_cmd.rs` |
| I-10 | 导出取消 100 条上限 | `export_cmd.rs`, `tauri.ts` |

### P2 — 安全加固

| ID | 任务 | 主要文件 |
|----|------|----------|
| I-6 | `lock` session 校验 | `auth.rs` |
| I-11 | breach 密码 Zeroizing | `breach.rs` |
| I-12 | 统一错误脱敏 | 各 `commands/*.rs`, `error.rs` |
| I-1 / M-12 | OsRng 统一 | `auth.rs`, `generator.rs` |
| I-13 | 锁定清空 vault store | `auth.ts` 或 `useAutoLock.ts` |
| I-14 | CommandPalette 锁定清剪贴板 | `CommandPalette.vue` |
| I-15 | 同步拒绝 v2 明文远程文件 | `sync/engine.rs` |
| I-16 | 导入/导出改 Rust 或限制 fs scope | `SettingsView.vue`, `capabilities/` |
| I-17 | 收紧 capabilities 权限 | `default.json` |
| I-21 | 迁移 WebDAV 明文凭据 | `sync.rs` |

### P3 — 质量与测试

| ID | 任务 |
|----|------|
| I-4 | Session TTL 测试 |
| M-4 | 安装 `cargo-audit` 加入 CI |
| M-11 | `search_entries` LIMIT |
| M-5 | vault CRUD 审计日志 |

---

## 修复任务清单（可勾选）

```
P0 桌面端
[x] C-1  SQLCipher                  ← Sprint 2 完成
[x] C-2  剪贴板 session          ← Sprint 1 完成
[x] C-3  get_setting 门禁        ← Sprint 1 完成
[x] C-4  ItemForm 编辑加载 secrets ← Sprint 1 完成

P0 扩展（暂停，恢复时做）
[ ] C-5  Native Host DB 路径统一
[ ] C-6  解锁态跨进程共享
[ ] C-7  save_credential / open_app
[ ] I-18 com.keyvault.app.json 部署配置

P1
[x] I-8  create/import 事务      ← Sprint 1 完成
[x] I-9  update_entry 事务       ← Sprint 1 完成
[x] I-10 export 全量 + format 契约 ← Sprint 1 完成

P2
[x] I-6  lock session               ← Sprint 2 完成
[x] I-11 breach Zeroizing           ← Sprint 2 完成
[x] I-12 错误脱敏                   ← Sprint 2 完成（auth/breach 等关键路径）
[x] I-1  OsRng salt                 ← Sprint 2 完成
[ ] I-13 锁定清 vault store
[ ] I-14 CommandPalette 剪贴板
[ ] I-15 同步 v2 明文拒绝
[x] I-16/I-17 fs 与 capabilities    ← Sprint 2 完成
[ ] I-21 WebDAV 凭据加密迁移

P3
[ ] I-2  lock 原子性
[ ] I-3  session 读锁
[ ] I-4  TTL 测试
[ ] I-7  元数据命令 key 检查
[ ] M-*  Minor 项按需处理
```

---

## Batch C 审查摘要

**已通过：**
- `vault` store 架构正确（仅元数据）
- `ItemDetail` 按需拉取 + 卸载清空
- 路由 `requiresAuth` 守卫
- Session token 纯内存、锁定清除
- `useAutoLock` 空闲锁定 + 失焦清剪贴板
- 无 Pinia/localStorage 持久化敏感数据
- HIBP 检测后前端清空 `checkPassword`

**新发现：**
- **C-4** 编辑表单不加载字段（严重数据丢失）
- **I-13** 锁定后 vault 元数据残留内存
- **I-14** CommandPalette 锁定路径不一致

---

## Batch D 审查摘要

### Sync 引擎（`sync/engine.rs` + `sync.rs` + `webdav.rs`）

| 检查项 | 状态 |
|--------|------|
| Push 前先 pull-merge 防覆盖远程 | ✅ |
| 远程文件 AES-GCM v3 加密 | ✅ |
| checksum 防篡改 | ✅ |
| 冲突合并 `updated_at` 较新胜出 | ✅ |
| WebDAV 凭据 `enc:` 存储 | ✅（新配置） |
| v2 明文远程文件兼容 | ⚠️ I-15 |
| 旧版 WebDAV 密码明文 | ⚠️ I-21 |
| 单元测试（merge + 加解密） | ✅ 3 项 |

### Native Messaging（`native_messaging/host.rs`）

| 检查项 | 状态 |
|--------|------|
| 消息长度上限 1MB | ✅ |
| 独立 session token | ✅ |
| 与 GUI 共享 DB 路径 | ❌ C-5 |
| 与 GUI 共享解锁态 | ❌ C-6 |
| `find_credentials` / `get_entry_for_fill` | ⚠️ 架构阻塞 |
| `save_credential` / `open_app` | ❌ C-7 |

### 浏览器扩展（`extension/`）

| 检查项 | 状态 |
|--------|------|
| 红线 #8 扩展零持久化解密数据 | ✅ 仅用 `session` 存连接状态 |
| Native Messaging 通信 | ⚠️ Host 未就绪 |
| manifest CSP | ✅ |
| content_scripts 范围 | ⚠️ M-14 `<all_urls>` |
| popup 复制密码清空 | ❌ I-19 |
| 部署清单 | ❌ I-18 placeholder |

### Tauri 权限与 CSP

| 检查项 | 状态 |
|--------|------|
| CSP `script-src 'self'` | ✅ |
| `fs` 插件启用 | ⚠️ I-16 / I-17 |
| 红线 #11 最小权限 | ❌ 与文档不符 |
| `shell:allow-open` | ⚠️ 用于 `ItemDetail` 打开 URL |

### 扩展红线对照（Batch D）

| 红线 | 状态 |
|------|------|
| #8 扩展零持久化 | ✅ |
| #11 Tauri 最小权限 | ❌ fs 插件 |
| #12 IPC session | ⚠️ Native Host 自建 token（进程内 OK，跨进程不共享解锁） |

---

## Batch E：测试覆盖矩阵

### Rust 单元测试（31 项，`cargo test` 全通过）

| 模块 | 测试数 | 覆盖内容 | 缺口 |
|------|--------|----------|------|
| `crypto/kdf.rs` | 7 | 派生确定性、哈希验证 | Argon2 耗时 ≥1s 未测 |
| `crypto/cipher.rs` | 6 | 往返、错误 nonce/key | 多字段独立 nonce 集成测 |
| `crypto/session.rs` | 5 | 创建/验证/销毁/滑动窗口 | **TTL 1800s 过期未测** |
| `db/queries.rs` | 7 | config、groups、audit_log | search、软删除、LIKE 转义未测 |
| `sync/engine.rs` | 3 | merge、加解密往返 | checksum 篡改拒绝未测 |
| `commands/sync.rs` | 2 | WebDAV 凭据加解密 | push/pull 集成未测 |
| `commands/generator.rs` | 1 | Diceware 词表 ≥256 | 随机模式字符集未测 |
| `commands/native_ext.rs` | 1 | `extract_hostname` | URL 匹配逻辑未测 |
| **`commands/auth.rs`** | **0** | — | setup 防重复、lockout、change_password |
| **`commands/vault.rs`** | **0** | — | CRUD、update 事务、历史 |
| **`commands/export_cmd.rs`** | **0** | — | 导入导出往返 |
| **`commands/breach.rs`** | **0** | — | k-匿名、缓存命中 |
| **`commands/clipboard_cmd.rs`** | **0** | — | session 门禁（待实现后测） |
| **`commands/settings.rs`** | **0** | — | 公开/敏感键分离 |
| **`state.rs`** | **0** | — | lock 原子性 |
| **`native_messaging/host.rs`** | **0** | — | 消息解析、action 路由 |
| **`sync/webdav.rs`** | **0** | — | HTTP mock 测试 |

### 前端测试

| 项 | 状态 |
|----|------|
| Vitest / Playwright / Cypress | **未配置**（`package.json` 无 test script） |
| 建议首批前端测 | 路由守卫、`ItemForm` 编辑加载、session token 清除 |

### 建议补测优先级（与修复联动）

1. **P0：** `vault::update_entry` 事务回滚；`auth::setup` 重复初始化；session TTL 过期  
2. **P1：** `export_cmd` 全量导出；`breach` 仅发 5 字符前缀（mock HTTP）  
3. **P2：** `queries::search_entries` LIKE 转义；`sync` 拒绝 v2 明文  
4. **P3：** IPC session 门禁集成测（clipboard、get_setting）

---

## Batch E：`KEYVAULT_V3_FULL_MIGRATION.md` §九验收对照

### 安全

| 验收项 | 状态 | 审查说明 |
|--------|------|----------|
| `cargo audit` 无高危 | ❌ 未测 | 环境未安装 `cargo-audit` |
| `cargo test` 全部通过 | ✅ | **31** 项通过（文档写 26，已过时） |
| Argon2id 验证 ≥ 1s | ❌ 未测 | 需基准测试脚本 |
| 锁定后 strings 无明文 | ❌ 未测 | 需手动/自动化内存扫描 |
| HIBP 仅发哈希前 5 字符 | ⚠️ 代码审查通过 | `breach.rs` 实现正确；Wireshark 未抓包 |

### 性能

| 验收项 | 状态 | 说明 |
|--------|------|------|
| 冷启动 < 2s | ❌ 未测 | 需 Windows 基准 |
| 空闲内存 < 60MB | ❌ 未测 | |
| 安装包 < 20MB | ❌ 未测 | |
| 搜索 < 50ms（1000 条） | ❌ 未测 | `list_entries` 限 100 条，搜索无 LIMIT |

### 功能

| 验收项 | 状态 | 说明 |
|--------|------|------|
| 首次设置向导 | ✅ | 代码 + REQUIREMENTS-COVERAGE |
| 条目 CRUD | ⚠️ | **编辑存在 C-4 数据丢失风险** |
| 命令面板 Ctrl+K | ✅ | 已实现 |
| 密码生成器双模式 | ✅ | Diceware 走后端 |
| 浏览器扩展（3 项） | ❌ 暂停 | 且存在 C-5/C-6/C-7 架构缺陷 |
| 剪贴板 30s 清空 | ✅ | 桌面端；扩展 popup 无（I-19） |
| 自动锁定 | ✅ | |
| 导入/导出 JSON | ⚠️ | 功能有，但 **I-10 截断 100 条** |
| WebDAV 同步 | ✅ | 代码审查通过；I-15 明文兼容需注意 |
| 紧急擦除 | ✅ | |

### UI/UX

| 验收项 | 状态 | 说明 |
|--------|------|------|
| 三栏布局 1280px+ | ❌ 未目视 | REQUIREMENTS 标 Done，需 SIGNOFF |
| 详情面板 60fps | ❌ 未测 | |
| Loading 状态 | ⚠️ 部分 | 多数有，未全量走查 |
| 错误信息友好 | ⚠️ | `vault.rs` 好；`auth/sync/export` 仍泄露（I-12） |
| 无边框拖动 | ❌ 未目视 | `window.rs` 已实现 |

**§九小结：** 功能面接近完成，**安全与性能验收大多未执行**；CRUD「编辑」项应降为未通过直至修复 C-4。

---

## Batch E：12 条安全红线总对照

| # | 红线 | 状态 | 关联发现 |
|---|------|------|----------|
| 1 | 主密码零存储 | ✅ | |
| 2 | Argon2id 参数不可降级 | ✅ | |
| 3 | 密钥不落盘（内存 Zeroizing） | ✅ | |
| 4 | 锁定时清零 key + sessions | ✅ | I-2 窗口期 |
| 5 | 敏感变量 Zeroizing | ⚠️ | I-11 breach 密码未包装 |
| 6 | 字段级独立 nonce | ✅ | |
| 7 | 前端零缓存解密数据 | ⚠️ | I-13 锁定后元数据残留 |
| 8 | 扩展零持久化 | ✅ | |
| 9 | HIBP k-匿名 | ✅ 代码 | 未抓包 |
| 10 | SQLCipher DB 加密 | ❌ | **C-1** |
| 11 | Tauri 最小权限 | ❌ | **I-16/I-17** fs 插件 |
| 12 | IPC 全部验证 session | ❌ | **C-2/C-3**；I-6 lock |

**红线通过：7/12（含 2 项代码通过未实测）**

---

## Executive Summary（最终）

### 总体结论

KeyVault v3 **桌面端核心功能已基本可用**（设置、解锁、列表、详情、生成器、同步、紧急擦除），加密实现（Argon2id + AES-GCM + session）质量良好，31 项 Rust 单元测试全通过。

**当前不宜作为生产版本发布。** 主要原因：

1. **数据库文件未加密（C-1）** — 与产品安全承诺不符  
2. **编辑条目可清空全部字段（C-4 + I-9）** — 用户数据丢失风险  
3. **IPC 门禁缺口（C-2/C-3）** — 未解锁可读敏感配置、可写剪贴板  
4. **导出静默截断 100 条（I-10）** — 备份不完整  
5. **浏览器扩展不可用（C-5/C-6/C-7）** — 与暂停策略一致，但代码不可交付  

### 发现汇总

| 级别 | 数量 | 桌面端相关 | 扩展相关 |
|------|------|------------|----------|
| Critical | 7 | 4 | 3 |
| Important | 18 | 14 | 4 |
| Minor | 11 | 7 | 4 |

### 发布就绪度（审查视角）

| 维度 | 评分 | 说明 |
|------|------|------|
| 加密核心 | 8/10 | 实现扎实；缺 SQLCipher |
| IPC 安全 | 5/10 | session 覆盖不全 |
| 数据完整性 | 4/10 | 编辑/更新/导入无事务 |
| 前端安全 | 7/10 | 架构正确；C-4 致命 |
| 同步 | 7/10 | 设计合理；v2 降级风险 |
| 扩展 | 2/10 | 架构未打通 |
| 测试 | 5/10 | crypto 好；commands 空白 |
| 验收完成度 | 4/10 | 大量 §九 未测 |

**桌面端 MVP 修复后可达发布候选（RC）；扩展需独立里程碑。**

---

## 修复 Sprint 建议（按文档逐项勾选）

### Sprint 1 — 数据与安全阻塞（预估 2–3 天）

**目标：** 消除数据丢失与最致命安全债

| 顺序 | ID | 任务 |
|------|-----|------|
| 1 | C-4 | `ItemForm` 编辑加载 `getEntrySecrets` |
| 2 | I-9 | `update_entry` 事务化 |
| 3 | I-8 | `create_entry` / `import_vault` 事务化 |
| 4 | C-2 | 剪贴板 session |
| 5 | C-3 | `get_setting` 公开/敏感拆分 |
| 6 | I-10 | 导出全量 + `format` 契约 |

**退出标准：** 编辑往返不丢字段；未解锁无法 get 敏感 config / 写剪贴板；导出条数与库内一致。

---

### Sprint 2 — 存储加密与硬化（预估 3–5 天）

**目标：** 对齐安全红线 #10/#11/#12

| 顺序 | ID | 任务 |
|------|-----|------|
| 1 | C-1 | SQLCipher + HKDF + PRAGMA key |
| 2 | I-16/I-17 | 导入导出改 Rust 或 fs scope |
| 3 | I-12 | 命令层统一 `AppError` 脱敏 |
| 4 | I-6 | `lock` session（可选） |
| 5 | I-11 | breach Zeroizing |
| 6 | I-1 | OsRng 统一 |

**退出标准：** `sqlite3` 无法打开 db；`cargo test` 仍 100% 通过；错误 UI 无 SQL 片段。

---

### Sprint 3 — 质量与验收（预估 2–3 天）

**目标：** 可度量、可签字发布

| 顺序 | 任务 |
|------|------|
| 1 | 安装 `cargo-audit` 并修 CVE |
| 2 | 补测：session TTL、setup 重复、update 回滚 |
| 3 | Argon2 耗时基准（≥1s） |
| 4 | I-13/I-14 锁定清理一致性 |
| 5 | 执行 §九性能抽样（启动/内存/搜索） |
| 6 | 人工 SIGNOFF（`REQUIREMENTS-COVERAGE` 目视项） |

---

### Sprint 4 — 浏览器扩展（独立里程碑，预估 5–8 天）

**前置：** Sprint 1–2 完成且桌面端 RC

| 顺序 | ID | 任务 |
|------|-----|------|
| 1 | C-5 | 统一 DB 路径 |
| 2 | C-6 | GUI ↔ Host 解锁态架构 |
| 3 | C-7 | `save_credential` / `open_app` |
| 4 | I-18 | 部署 `com.keyvault.app.json` |
| 5 | I-19/I-20 | popup 剪贴板 + 保存去重 |

---

## 审查完成声明

- **审查范围：** `src-tauri/`、`src/`、`extension/`、capabilities、迁移文档对照  
- **审查基线：** `9534512` @ `master`  
- **批次：** A（Crypto/State）→ B（DB/IPC）→ C（前端）→ D（Sync/Ext/权限）→ E（测试/验收/汇总）  
- **交付物：** 本文档即修复唯一入口；修复时更新文首 checklist 勾选状态  

---

*全项目代码审查已完成。*

---

## Sprint 1 修复记录（2026-06-05）

| ID | 状态 | 改动摘要 |
|----|------|----------|
| C-4 | ✅ | `ItemForm.vue` 编辑时 `getEntrySecrets` 加载字段，加载中禁用保存 |
| I-9 | ✅ | `vault.rs` `update_entry` 全步骤包在事务中 |
| I-8 | ✅ | `create_entry` 事务化；`import_vault` 每条目独立事务 |
| C-2 | ✅ | `clipboard_cmd.rs` + `tauri.ts` 剪贴板需 session |
| C-3 | ✅ | `settings.rs` 公开键白名单（theme/auto_lock/clipboard_clear） |
| I-10 | ✅ | `list_all_active_entries` 全量导出；`export/import` 接受 `format` |

**验证：** `cargo test` 31 passed · `npm run type-check` 通过

**Sprint 1 退出标准：** ✅ 编辑不丢字段 · ✅ 未解锁不可写剪贴板/读敏感 config · ✅ 导出无 100 条上限

## Sprint 2 修复记录（2026-06-05）

| ID | 状态 | 改动摘要 |
|----|------|----------|
| C-1 | ✅ | SQLCipher + HKDF `derive_db_key` + 解锁时 `PRAGMA key`；锁定关闭连接池 |
| I-16/I-17 | ✅ | `export_vault_to_file` / `import_vault_from_file`（Rust + dialog + `std::fs`）；移除 `plugin-fs` 权限 |
| I-12 | ✅ | `ipc_db_err` / `ipc_crypto_err`；auth/breach 等路径脱敏 |
| I-6 | ✅ | `lock` 需 `session_token`；前端 `tauri.ts` 传 token |
| I-11 | ✅ | `breach.rs` 密码 `Zeroizing<Vec<u8>>` |
| I-1 | ✅ | `auth.rs` salt 改用 `OsRng` |

**架构变更：**
- 启动时不打开 DB；`is_initialized` = `keyvault.db` 存在
- Salt sidecar：`data_dir/kdf_salt.hex`；遗留明文库首次 unlock 自动 `PRAGMA rekey`
- `AppState.db` → `RwLock<Option<SqlitePool>>`；各命令经 `db_pool().await?` 访问

**构建说明（Windows）：** `libsqlite3-sys` 使用 `bundled-sqlcipher-vendored-openssl`，首次编译需 Perl（如 Strawberry Perl）。

**验证：** `cargo test` 31 passed · `npm run type-check` 通过

**Sprint 2 退出标准：** ✅ 锁定后无 DB 连接 · ✅ 导入导出不经前端 fs · ✅ lock 需 session · ⏳ 手动验收：`sqlite3` 无法直接读加密库（需本地 unlock 后验证）

**下一步：** Sprint 3 — cargo-audit、session TTL 测试、Argon2 基准、I-13/I-14 锁定清理
