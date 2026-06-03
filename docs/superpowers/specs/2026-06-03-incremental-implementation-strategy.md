# KeyVault v3 增量式实现策略设计

> **日期**: 2026-06-03
> **策略**: 最小文件逐步加（方案 B）
> **目标**: 增量实现，每步可验证，最小加密闭环优先

---

## 环境状态

- **Node.js**: v24.14.0 ✓
- **npm**: 11.11.0 ✓
- **Rust**: 未安装 ✗
- **前端骨架**: 已完成（33 个文件）

---

## 实现步骤

### Phase 0: 环境准备

**安装 Rust toolchain**:
1. 下载并运行 https://win.rustup.rs/x86_64 （或 `winget install Rustlang.Rustup`）
2. 安装时选择默认选项
3. **重启终端**（让 PATH 生效）
4. 验证：`rustc --version` 和 `cargo --version`

**验证标准**: `rustc --version` 和 `cargo --version` 输出正常。

---

### Phase 1: 最小 Tauri 项目（Step 1-2）

**创建文件**:
```
src-tauri/
├── Cargo.toml          # tauri, serde, serde_json
├── tauri.conf.json     # 无边框窗口、最小尺寸 900x600
├── capabilities/
│   └── default.json    # 最小权限声明
├── build.rs            # tauri_build::build()
└── src/
    ├── main.rs         # 仅 app.run()
    └── lib.rs          # 空库
```

**Cargo.toml 最小依赖**:
```toml
[package]
name = "keyvault"
version = "3.0.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

**tauri.conf.json 关键配置**:
```json
{
  "build": {
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{
      "title": "KeyVault",
      "width": 900,
      "height": 600,
      "decorations": false,
      "resizable": true
    }],
    "security": {
      "csp": "default-src 'self'; script-src 'self'"
    }
  }
}
```

**验证标准**: `cargo tauri dev` 窗口打开，显示 Vue 前端 UnlockView。

---

### Phase 2: 加密核心（Step 3-5）

**添加依赖**:
```toml
argon2 = "0.5"
aes-gcm = "0.10"
rand = "0.8"
zeroize = "1.7"
secrecy = "0.8"
base64 = "0.22"
hex = "0.4"
thiserror = "2"
tokio = { version = "1", features = ["full"] }
```

**创建文件**:
```
src-tauri/src/
├── error.rs            # AppError (thiserror)
├── state.rs            # AppState + SessionManager
└── crypto/
    ├── mod.rs
    ├── kdf.rs          # derive_key, hash_master_password, verify_master_password
    └── cipher.rs       # encrypt_field, decrypt_field
```

**关键实现**:
- `kdf.rs`: Argon2id 参数锁定 (memoryCost=65536, timeCost=3, parallelism=1, 32 字节输出)
- `cipher.rs`: AES-256-GCM，每字段独立随机 12 字节 nonce，prepend 到密文
- `state.rs`: AppState { encryption_key: RwLock<Option<Zeroizing<[u8;32]>>>, sessions: SessionManager }
- SessionManager: 64 字符随机 token，30 分钟 TTL，滑动窗口

**验证标准**: 添加一个测试 `#[tauri::command]`，前端 invoke 后收到 "crypto works" 响应。

---

### Phase 3: 数据库 + 认证命令（Step 6-8）— 最小加密闭环

**添加依赖**:
```toml
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }
libsqlite3-sys = { version = "0.30", features = ["bundled-sqlcipher"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

**创建文件**:
```
src-tauri/src/
├── db/
│   ├── mod.rs          # 初始化连接池，设置 SQLCipher key
│   ├── migrations/
│   │   └── 001_init.sql
│   ├── schema.rs       # 数据库结构体
│   └── queries.rs      # 基础查询
└── commands/
    ├── mod.rs
    └── auth.rs         # is_initialized, setup, unlock, lock
```

**数据库迁移 001_init.sql**:
```sql
CREATE TABLE IF NOT EXISTS config (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS groups (id TEXT PRIMARY KEY, name TEXT NOT NULL, icon TEXT, color TEXT, sort_order INTEGER DEFAULT 0, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS entries (id TEXT PRIMARY KEY, group_id TEXT, entry_type TEXT NOT NULL, title TEXT NOT NULL, subtitle TEXT, tags TEXT, favorited INTEGER DEFAULT 0, sort_order INTEGER DEFAULT 0, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS fields (id TEXT PRIMARY KEY, entry_id TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE, field_key TEXT NOT NULL, field_type TEXT DEFAULT 'text', enc_value TEXT NOT NULL, is_sensitive INTEGER DEFAULT 1, sort_order INTEGER DEFAULT 0);
CREATE INDEX IF NOT EXISTS idx_entries_type ON entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_entries_group ON entries(group_id);
CREATE INDEX IF NOT EXISTS idx_fields_entry ON fields(entry_id);
```

**认证命令签名**:
```rust
#[tauri::command] async fn is_initialized(state) -> Result<bool, String>
#[tauri::command] async fn setup(password, state) -> Result<String, String>  // 返回 session_token
#[tauri::command] async fn unlock(password, state) -> Result<String, String> // 返回 session_token
#[tauri::command] async fn lock(state) -> Result<(), String>
```

**前端对接**: bridge/tauri.ts 已有对应接口，确保命令名和参数匹配即可。

**验证标准**:
1. `cargo tauri dev` → SetupView → 输入密码 → 数据库创建，password_hash 和 kdf_salt 写入 config 表
2. 关闭重开 → UnlockView → 输入密码 → VaultView（空状态）
3. 锁定后内存中 encryption_key 为 None

---

### Phase 4: Vault CRUD（Step 9-11）— 第一个可用版本

**创建文件**:
```
src-tauri/src/commands/
├── vault.rs            # list, get_secrets, create, update, delete, search
```

**命令签名**:
```rust
#[tauri::command] async fn list_entries(session_token, state) -> Result<Vec<EntryMeta>, String>
#[tauri::command] async fn get_entry_secrets(session_token, entry_id, state) -> Result<EntrySecrets, String>
#[tauri::command] async fn create_entry(session_token, input, state) -> Result<String, String>
#[tauri::command] async fn update_entry(session_token, entry_id, input, state) -> Result<(), String>
#[tauri::command] async fn delete_entry(session_token, entry_id, state) -> Result<(), String>
#[tauri::command] async fn search_entries(session_token, query, state) -> Result<Vec<EntryMeta>, String>
```

**安全规则**:
- 所有命令第一步验证 session_token
- `list_entries` 只返回元数据，不解密字段
- `get_entry_secrets` 按需解密，返回后不缓存
- `create_entry` 每个字段独立加密（独立 nonce）

**前端完善**:
- VaultView 三栏布局：侧边栏 + 条目列表 + 详情面板
- 新建条目表单（根据 entry_type 动态字段）
- 详情面板按需加载 secrets，onUnmounted 清空

**main.rs 注册所有命令**:
```rust
app.invoke_handler(tauri::generate_handler![
    is_initialized, setup, unlock, lock,
    list_entries, get_entry_secrets, create_entry,
    update_entry, delete_entry, search_entries,
])
```

**验证标准**:
1. 能创建条目（密码类型），加密存储到 fields 表
2. 列表显示条目标题和副标题
3. 点击条目，详情面板显示解密后的字段
4. 编辑、删除正常工作
5. 搜索能匹配标题

---

## 后续里程碑（本次不实现）

完成 Phase 4 后，按优先级继续：
- **M2**: 密码生成器 + 剪贴板定时清空
- **M3**: 收藏、分组管理
- **M4**: 导入/导出
- **M5**: WebDAV 同步
- **M6**: 浏览器扩展
- **M7**: HIBP 泄露检测
- **M8**: UI 组件库完善、虚拟滚动、动画

---

## 安全红线（贯穿所有 Phase）

1. 主密码零存储（只存 Argon2id 哈希）
2. 密钥不落盘（AppState.encryption_key 内存中 Zeroizing 包装）
3. 锁定时清零 encryption_key + 销毁所有 sessions
4. 字段级加密（每字段独立 nonce）
5. 前端零缓存解密数据（vault store 只存元数据）
6. IPC 全部验证 session token（setup/unlock 除外）

---

## 技术决策记录

| 决策 | 选择 | 理由 |
|------|------|------|
| 实现节奏 | 增量式，最小闭环优先 | 每步可验证，出错定位快 |
| Tauri 初始化 | 手动创建 src-tauri/ | 已有前端骨架，不需要 CLI 模板 |
| 数据库 | sqlx + SQLCipher | 官方插件不支持 SQLCipher |
| UI 组件库 | 暂用原生 HTML | Phase 4 先跑通功能，后续再完善组件库 |
| 虚拟滚动 | 暂用普通列表 | 条目少时无性能问题，后续加 @vueuse/core |
