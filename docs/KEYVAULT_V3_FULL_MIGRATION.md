# KeyVault v3 完整迁移规划

> **状态（2026-06-05）：** 迁移已执行完毕，本文档作**架构与验收参考**保留。  
> **当前现状：** 见 [PROJECT_STATUS.md](./PROJECT_STATUS.md) · **审查记录：** [CODE_REVIEW_REPORT.md](./CODE_REVIEW_REPORT.md)

> **执行对象**：Claude Code  
> **源仓库**：https://github.com/ismoyuai/keyvault  
> **迁移策略**：全新项目 `keyvault-v3/`，原仓库保留为参考，不在原地修改  
> **核心决策**：Electron → Tauri 2.0 | Node.js → Rust | sql.js → SQLCipher | Vue 3 保留

---

## 用户画像与设计约束

在开始任何代码之前，必须理解这些约束，它们影响每一个技术和设计决策：

- **风格**：暗黑高密度型，参考 Warp Terminal / Obsidian。不是"暗色模式"，是以暗黑为核心设计语言，高信息密度，终端美学。
- **核心场景（按优先级）**：
  1. 浏览器自动填充（丝滑，零摩擦，这是最高频操作）
  2. 快速查找 + 复制（搜索驱动，不依赖鼠标浏览）
  3. 开发者场景：API Key、SSH Key 管理（支持多行、格式化显示）
  4. 结构化信息存储（护照、银行卡、服务器信息）
- **平台**：Windows 主力，架构上为 macOS/Linux/移动端预留路径（Tauri 2.0 天然支持）

---

## 一、技术栈选型（最终确定）

### 1.1 完整栈

```
桌面框架    Tauri 2.0            (系统 WebView + Rust 后端)
后端语言    Rust (stable)         (加密/存储/IPC/系统调用)
前端框架    Vue 3 + TypeScript    (保留，无缝迁移)
状态管理    Pinia                 (保留)
路由        Vue Router 4          (保留)
构建工具    Vite 6                (保留)

加密        RustCrypto crates
  - argon2 = "0.5"               (Argon2id KDF)
  - aes-gcm = "0.10"             (AES-256-GCM)
  - rand = "0.8"                 (CSPRNG)
  - zeroize = "1.7"              (自动内存清零)
  - secrecy = "0.8"              (Secret<T> 包装类型)

数据库      sqlx + SQLCipher
  - sqlx = { features = ["sqlite", "runtime-tokio-rustls"] }
  - libsqlite3-sys = { features = ["bundled-sqlcipher"] }

异步运行时  tokio = { features = ["full"] }
序列化      serde + serde_json
错误处理    thiserror + anyhow
UUID        uuid = { features = ["v4"] }
时间        chrono

前端 UI 组件  自建（无 Element Plus）
图标          lucide-vue-next
```

### 1.2 为什么不用 Tauri 官方 SQL 插件

官方 `@tauri-apps/plugin-sql` **不支持 SQLCipher**，这个 issue 已开放 3 年未解决。我们直接在 Rust 后端用 `sqlx` 管理数据库连接，前端通过 Tauri `invoke()` 调用 Rust 命令，获得完整控制权。

### 1.3 为什么不用 tauri-plugin-stronghold

Stronghold 是 IOTA 的安全存储方案，依赖复杂，参数不可控。我们的 Argon2id 参数有历史约定（`memoryCost=65536, timeCost=3`），直接用 `argon2` crate 实现可控性更强。

---

## 二、项目结构

```
keyvault-v3/
├── src-tauri/                    # Rust 后端（Tauri 主进程）
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Tauri 配置（能力系统、窗口、托盘）
│   ├── capabilities/
│   │   └── default.json          # 最小权限声明
│   ├── build.rs
│   └── src/
│       ├── main.rs               # 入口，注册所有命令
│       ├── lib.rs                # 库根
│       ├── error.rs              # 统一错误类型 (thiserror)
│       ├── state.rs              # AppState（加密密钥、DB连接池）
│       ├── crypto/
│       │   ├── mod.rs
│       │   ├── kdf.rs            # Argon2id 密钥派生
│       │   ├── cipher.rs         # AES-256-GCM 加解密
│       │   └── session.rs        # 会话令牌（内存中，TTL 30min）
│       ├── db/
│       │   ├── mod.rs
│       │   ├── migrations/       # SQL 迁移文件
│       │   │   ├── 001_init.sql
│       │   │   └── 002_history.sql
│       │   ├── schema.rs         # 数据库结构体定义
│       │   └── queries.rs        # 所有 SQL 查询
│       ├── commands/             # Tauri 命令（IPC 入口）
│       │   ├── mod.rs
│       │   ├── auth.rs           # 认证相关
│       │   ├── vault.rs          # 条目 CRUD
│       │   ├── generator.rs      # 密码生成
│       │   ├── breach.rs         # HIBP 泄露检测
│       │   ├── sync.rs           # WebDAV 同步
│       │   └── export.rs         # 导入/导出
│       ├── native_messaging/     # 浏览器扩展通信
│       │   └── host.rs
│       └── tray.rs               # 系统托盘
│
├── src/                          # Vue 3 前端（TypeScript）
│   ├── design/
│   │   ├── tokens.css            # 设计令牌（暗黑高密度主题）
│   │   ├── typography.css        # 字体系统（JetBrains Mono + Geist Sans）
│   │   └── animations.css        # 动画曲线
│   ├── components/
│   │   ├── vault/
│   │   │   ├── VaultItem.vue     # 条目行（60px 高密度）
│   │   │   ├── VaultList.vue     # 虚拟滚动列表
│   │   │   ├── ItemDetail.vue    # 侧滑详情面板
│   │   │   └── ItemForm.vue      # 新建/编辑表单
│   │   ├── search/
│   │   │   └── CommandPalette.vue # 全局命令面板（⌘K）
│   │   ├── security/
│   │   │   ├── PasswordStrength.vue
│   │   │   ├── BreachBadge.vue
│   │   │   └── ClipboardTimer.vue
│   │   ├── generator/
│   │   │   └── PasswordGenerator.vue
│   │   └── ui/                   # 基础组件（Button、Input、Modal等）
│   ├── views/
│   │   ├── UnlockView.vue
│   │   ├── SetupView.vue
│   │   ├── VaultView.vue         # 主界面（三栏）
│   │   └── SettingsView.vue
│   ├── stores/
│   │   ├── auth.ts
│   │   ├── vault.ts
│   │   ├── ui.ts
│   │   └── settings.ts
│   ├── composables/
│   │   ├── useVault.ts
│   │   ├── useClipboard.ts
│   │   ├── useAutoLock.ts
│   │   ├── useShortcuts.ts
│   │   └── usePasswordGen.ts
│   ├── bridge/
│   │   └── tauri.ts              # 统一封装所有 invoke() 调用
│   └── types/
│       └── vault.ts              # 共享 TypeScript 类型
│
├── extension/                    # 浏览器扩展（完整重构）
│   ├── manifest.json
│   ├── background/
│   │   └── service-worker.ts
│   ├── content/
│   │   ├── detector.ts
│   │   ├── filler.ts
│   │   └── save-prompt.ts
│   └── popup/
│       ├── popup.html
│       ├── popup.ts
│       └── popup.css
│
├── package.json
├── vite.config.ts
└── CLAUDE.md                     # 更新版项目说明
```

---

## 三、Rust 后端实现规范

### 3.1 安全核心：`src-tauri/src/crypto/`

#### `kdf.rs` — Argon2id 密钥派生

```rust
use argon2::{Argon2, Algorithm, Version, Params};
use argon2::password_hash::{PasswordHasher, SaltString, PasswordHash, PasswordVerifier};
use rand_core::OsRng;
use zeroize::Zeroizing;

/// 派生加密密钥（32字节），用于 AES-256-GCM
/// 参数锁定：memoryCost=65536 KiB, timeCost=3, parallelism=1
pub fn derive_key(password: &[u8], salt: &[u8; 16]) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let params = Params::new(65536, 3, 1, Some(32))
        .map_err(|_| CryptoError::KdfParamError)?;
    
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    let mut key = Zeroizing::new([0u8; 32]);
    argon2.hash_password_into(password, salt, key.as_mut())
        .map_err(|_| CryptoError::KdfError)?;
    
    Ok(key)
}

/// 哈希主密码用于验证存储（使用 password_hash 格式，包含随机 salt）
pub fn hash_master_password(password: &[u8]) -> Result<String, CryptoError> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(65536, 3, 1, None)
        .map_err(|_| CryptoError::KdfParamError)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let hash = argon2.hash_password(password, &salt)
        .map_err(|_| CryptoError::HashError)?
        .to_string();
    Ok(hash)
}

/// 验证主密码
pub fn verify_master_password(password: &[u8], hash: &str) -> Result<bool, CryptoError> {
    let parsed = PasswordHash::new(hash).map_err(|_| CryptoError::HashParseError)?;
    Ok(Argon2::default().verify_password(password, &parsed).is_ok())
}
```

#### `cipher.rs` — AES-256-GCM 字段加密

```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, AeadCore, OsRng};
use zeroize::Zeroizing;

/// 加密单个字段，返回 base64 编码的 nonce+ciphertext
/// nonce 每次随机生成（12字节），prepend 到密文前
pub fn encrypt_field(key: &[u8; 32], plaintext: &[u8]) -> Result<String, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::CipherInitError)?;
    
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let mut ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|_| CryptoError::EncryptError)?;
    
    // prepend nonce (12字节) 到密文前
    let mut result = nonce.to_vec();
    result.append(&mut ciphertext);
    
    Ok(base64::engine::general_purpose::STANDARD.encode(result))
}

/// 解密字段，解密后的数据用 Zeroizing 包装确保使用后清零
pub fn decrypt_field(key: &[u8; 32], encoded: &str) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    let data = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| CryptoError::Base64Error)?;
    
    if data.len() < 12 {
        return Err(CryptoError::InvalidCiphertext);
    }
    
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::CipherInitError)?;
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptError)?;
    
    Ok(Zeroizing::new(plaintext))
}
```

#### `session.rs` — 会话令牌

```rust
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use rand::Rng;
use tokio::sync::RwLock;

const SESSION_TTL_SECS: u64 = 1800; // 30 分钟

pub struct SessionManager {
    sessions: RwLock<HashMap<String, SystemTime>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: RwLock::new(HashMap::new()) }
    }

    pub async fn create(&self) -> String {
        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
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
                if created_at.elapsed().unwrap_or_default() > Duration::from_secs(SESSION_TTL_SECS) {
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
        self.sessions.write().await.clear();
    }
}
```

### 3.2 应用状态：`src-tauri/src/state.rs`

```rust
use sqlx::SqlitePool;
use zeroize::Zeroizing;
use tokio::sync::RwLock;
use crate::crypto::session::SessionManager;

pub struct AppState {
    /// 加密密钥，仅在解锁后存在，锁定时清零
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
    /// SQLCipher 连接池（数据库始终保持加密）
    pub db: SqlitePool,
    /// 会话令牌管理器
    pub sessions: SessionManager,
    /// KDF 的 salt（存储在 DB 的 config 表中）
    pub kdf_salt: [u8; 16],
}

impl AppState {
    pub async fn lock(&self) {
        // 清零加密密钥
        let mut key = self.encryption_key.write().await;
        if let Some(k) = key.as_mut() {
            // Zeroizing<T> 在 drop 时自动清零，但我们显式 None 化
        }
        *key = None;
        // 销毁所有会话
        self.sessions.destroy_all().await;
    }

    pub fn is_unlocked(&self) -> bool {
        // 用 try_read 的非阻塞版本检查
        self.encryption_key.try_read()
            .map(|k| k.is_some())
            .unwrap_or(false)
    }
}
```

### 3.3 数据库：`src-tauri/src/db/`

#### `migrations/001_init.sql`

```sql
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
    entry_type  TEXT NOT NULL,  -- 'login', 'api_key', 'ssh_key', 'note', 'server', 'identity', 'card', 'license', 'crypto', 'custom'
    title       TEXT NOT NULL,  -- 明文（用于搜索）
    subtitle    TEXT,           -- 明文（url、host等，用于搜索和显示）
    tags        TEXT,           -- JSON 数组，明文
    favorited   INTEGER NOT NULL DEFAULT 0,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

-- 加密字段（每个条目可有多个加密字段）
CREATE TABLE IF NOT EXISTS fields (
    id           TEXT PRIMARY KEY,
    entry_id     TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    field_key    TEXT NOT NULL,             -- 字段名称（'username', 'password', 'api_key'等）
    field_type   TEXT NOT NULL DEFAULT 'text', -- 'text', 'password', 'otp', 'url', 'file'
    enc_value    TEXT NOT NULL,             -- AES-256-GCM 加密后的 base64
    is_sensitive INTEGER NOT NULL DEFAULT 1, -- 是否为敏感字段（影响显示）
    sort_order   INTEGER NOT NULL DEFAULT 0
);

-- 泄露检测缓存
CREATE TABLE IF NOT EXISTS breach_cache (
    hash_prefix TEXT PRIMARY KEY,           -- SHA1 前5字符（大写）
    result_json TEXT NOT NULL,              -- JSON: { "entries": [{ "suffix": "...", "count": N }] }
    cached_at   INTEGER NOT NULL
);

-- 审计日志
CREATE TABLE IF NOT EXISTS audit_log (
    id          TEXT PRIMARY KEY,
    action      TEXT NOT NULL,              -- 'unlock', 'view', 'copy', 'edit', 'delete', 'export'
    entry_id    TEXT,
    field_key   TEXT,
    metadata    TEXT,                       -- JSON
    occurred_at INTEGER NOT NULL
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_entries_type      ON entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_entries_group     ON entries(group_id);
CREATE INDEX IF NOT EXISTS idx_entries_favorited ON entries(favorited);
CREATE INDEX IF NOT EXISTS idx_fields_entry      ON fields(entry_id);
```

#### `migrations/002_history.sql`

```sql
-- 密码历史记录（加密存储）
CREATE TABLE IF NOT EXISTS field_history (
    id          TEXT PRIMARY KEY,
    entry_id    TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    field_key   TEXT NOT NULL,
    enc_value   TEXT NOT NULL,              -- AES-256-GCM 加密
    changed_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_field_history_entry ON field_history(entry_id, field_key, changed_at);
```

### 3.4 Tauri 命令：`src-tauri/src/commands/`

#### 命令设计原则

所有命令签名必须遵循：
1. 接收 `session_token: String` 参数（`auth::setup` 和 `auth::unlock` 除外）
2. 调用 `state.sessions.validate(&session_token).await` 作为第一步
3. 返回 `Result<T, String>`（`String` 为错误信息）
4. 敏感返回值用 `Zeroizing` 包装，在 `impl Serialize` 时直接序列化然后立即 drop

#### `commands/auth.rs`

```rust
use tauri::State;
use crate::state::AppState;
use crate::crypto::{kdf, cipher};
use crate::error::AppError;

#[tauri::command]
pub async fn is_initialized(state: State<'_, AppState>) -> Result<bool, String> {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT value FROM config WHERE key = 'password_hash'"
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(result.is_some())
}

#[tauri::command]
pub async fn setup(
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use zeroize::Zeroizing;
    let password_bytes = Zeroizing::new(password.into_bytes());
    
    // 1. 生成 KDF salt
    let mut salt = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
    
    // 2. 派生加密密钥
    let key = kdf::derive_key(&password_bytes, &salt)
        .map_err(|e| e.to_string())?;
    
    // 3. 哈希主密码用于验证
    let hash = kdf::hash_master_password(&password_bytes)
        .map_err(|e| e.to_string())?;
    
    // 4. 存入数据库
    sqlx::query("INSERT INTO config (key, value) VALUES ('kdf_salt', ?), ('password_hash', ?)")
        .bind(hex::encode(salt))
        .bind(hash)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    
    // 5. 激活加密密钥
    *state.encryption_key.write().await = Some(key);
    
    // 6. 创建会话
    let token = state.sessions.create().await;
    Ok(token)
}

#[tauri::command]
pub async fn unlock(
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use zeroize::Zeroizing;
    let password_bytes = Zeroizing::new(password.into_bytes());
    
    // 1. 获取存储的哈希
    let hash = sqlx::query_scalar::<_, String>(
        "SELECT value FROM config WHERE key = 'password_hash'"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| "密码管理器未初始化".to_string())?;
    
    // 2. 验证密码
    let valid = kdf::verify_master_password(&password_bytes, &hash)
        .map_err(|e| e.to_string())?;
    
    if !valid {
        return Err("密码错误".to_string());
    }
    
    // 3. 获取 KDF salt
    let salt_hex = sqlx::query_scalar::<_, String>(
        "SELECT value FROM config WHERE key = 'kdf_salt'"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    
    let salt_bytes = hex::decode(&salt_hex).map_err(|e| e.to_string())?;
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&salt_bytes[..16]);
    
    // 4. 派生密钥
    let key = kdf::derive_key(&password_bytes, &salt)
        .map_err(|e| e.to_string())?;
    
    // 5. 激活密钥
    *state.encryption_key.write().await = Some(key);
    
    // 6. 创建会话
    let token = state.sessions.create().await;
    Ok(token)
}

#[tauri::command]
pub async fn lock(state: State<'_, AppState>) -> Result<(), String> {
    state.lock().await;
    Ok(())
}

#[tauri::command]
pub async fn change_password(
    session_token: String,
    old_password: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    // ... 重新派生密钥，重加密所有字段
    todo!()
}
```

#### `commands/vault.rs`

```rust
use tauri::State;
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::crypto::cipher;

#[derive(Serialize)]
pub struct EntryMeta {
    pub id: String,
    pub entry_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
    pub favorited: bool,
    pub group_id: Option<String>,
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct EntrySecrets {
    pub fields: Vec<DecryptedField>,
}

#[derive(Serialize)]
pub struct DecryptedField {
    pub field_key: String,
    pub field_type: String,
    pub value: String,
    pub is_sensitive: bool,
}

#[derive(Deserialize)]
pub struct CreateEntryInput {
    pub entry_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
    pub group_id: Option<String>,
    pub fields: Vec<FieldInput>,
}

#[derive(Deserialize)]
pub struct FieldInput {
    pub field_key: String,
    pub field_type: String,
    pub value: String,
    pub is_sensitive: bool,
}

/// 获取所有条目元数据（不含敏感字段，用于列表显示）
#[tauri::command]
pub async fn list_entries(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntryMeta>, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    
    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, i32, Option<String>, i64)>(
        "SELECT id, entry_type, title, subtitle, tags, favorited, group_id, updated_at
         FROM entries ORDER BY favorited DESC, updated_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    
    let entries = rows.into_iter().map(|(id, entry_type, title, subtitle, tags_json, favorited, group_id, updated_at)| {
        let tags = tags_json
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default();
        EntryMeta { id, entry_type, title, subtitle, tags, favorited: favorited != 0, group_id, updated_at }
    }).collect();
    
    Ok(entries)
}

/// 按需获取单个条目的解密字段（仅在用户明确查看时调用）
#[tauri::command]
pub async fn get_entry_secrets(
    session_token: String,
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<EntrySecrets, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    
    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;
    
    let rows = sqlx::query_as::<_, (String, String, String, i32)>(
        "SELECT field_key, field_type, enc_value, is_sensitive
         FROM fields WHERE entry_id = ? ORDER BY sort_order"
    )
    .bind(&entry_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    
    let mut fields = Vec::new();
    for (field_key, field_type, enc_value, is_sensitive) in rows {
        let decrypted = cipher::decrypt_field(key, &enc_value)
            .map_err(|e| e.to_string())?;
        let value = String::from_utf8(decrypted.to_vec())
            .map_err(|_| "解码失败".to_string())?;
        fields.push(DecryptedField {
            field_key,
            field_type,
            value,
            is_sensitive: is_sensitive != 0,
        });
    }
    
    Ok(EntrySecrets { fields })
}

#[tauri::command]
pub async fn create_entry(
    session_token: String,
    input: CreateEntryInput,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    
    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;
    
    let entry_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let tags_json = serde_json::to_string(&input.tags).unwrap_or_default();
    
    // 插入条目元数据
    sqlx::query(
        "INSERT INTO entries (id, group_id, entry_type, title, subtitle, tags, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&entry_id)
    .bind(&input.group_id)
    .bind(&input.entry_type)
    .bind(&input.title)
    .bind(&input.subtitle)
    .bind(&tags_json)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    
    // 插入加密字段
    for (i, field) in input.fields.iter().enumerate() {
        let enc_value = cipher::encrypt_field(key, field.value.as_bytes())
            .map_err(|e| e.to_string())?;
        let field_id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query(
            "INSERT INTO fields (id, entry_id, field_key, field_type, enc_value, is_sensitive, sort_order)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
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
    
    Ok(entry_id)
}

/// 其余命令：update_entry, delete_entry, toggle_favorite, search_entries
/// 同样模式：session 验证 → 操作 → 返回
```

#### `commands/breach.rs` — k-匿名性 HIBP 查询

```rust
use tauri::State;
use sha1::{Sha1, Digest};
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct BreachResult {
    pub breached: bool,
    pub count: u64,
}

/// k-匿名性：只发送 SHA1 前5字符，在本地比对完整哈希
/// 密码原文永远不离开设备
#[tauri::command]
pub async fn check_password_breach(
    session_token: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<BreachResult, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }
    
    // 1. 计算 SHA1 哈希
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    let hash = format!("{:X}", hasher.finalize());
    
    let prefix = &hash[..5];
    let suffix = &hash[5..];
    
    // 2. 检查缓存
    let cached = sqlx::query_as::<_, (String, i64)>(
        "SELECT result_json, cached_at FROM breach_cache WHERE hash_prefix = ?"
    )
    .bind(prefix)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    
    let now = chrono::Utc::now().timestamp();
    
    // 缓存有效期 7 天
    if let Some((result_json, cached_at)) = cached {
        if now - cached_at < 7 * 24 * 3600 {
            return parse_breach_result(&result_json, suffix);
        }
    }
    
    // 3. 查询 HIBP API（仅发送前5字符）
    let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);
    let response = reqwest::get(&url)
        .await
        .map_err(|_| "网络请求失败，跳过泄露检测".to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    
    // 4. 缓存结果
    let cache_json = serde_json::json!({ "lines": response });
    sqlx::query(
        "INSERT OR REPLACE INTO breach_cache (hash_prefix, result_json, cached_at) VALUES (?, ?, ?)"
    )
    .bind(prefix)
    .bind(cache_json.to_string())
    .bind(now)
    .execute(&state.db)
    .await
    .ok(); // 缓存失败不影响结果
    
    // 5. 本地比对（密码原文已在步骤1之后丢弃）
    for line in response.lines() {
        if let Some((s, count)) = line.split_once(':') {
            if s.trim().eq_ignore_ascii_case(suffix) {
                let count = count.trim().parse::<u64>().unwrap_or(1);
                return Ok(BreachResult { breached: true, count });
            }
        }
    }
    
    Ok(BreachResult { breached: false, count: 0 })
}

fn parse_breach_result(json: &str, suffix: &str) -> Result<BreachResult, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let lines = v["lines"].as_str().unwrap_or("");
    for line in lines.lines() {
        if let Some((s, count)) = line.split_once(':') {
            if s.trim().eq_ignore_ascii_case(suffix) {
                let count = count.trim().parse::<u64>().unwrap_or(1);
                return Ok(BreachResult { breached: true, count });
            }
        }
    }
    Ok(BreachResult { breached: false, count: 0 })
}
```

---

## 四、Tauri 能力配置（最小权限）

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "KeyVault 最小权限配置",
  "windows": ["main"],
  "permissions": [
    "core:path:default",
    "core:event:default",
    "core:window:default",
    "core:app:default",
    "core:clipboard:write-text",
    "core:tray:default",
    "shell:allow-open"
  ]
}
```

**绝对不开启**：`core:fs:*`（文件系统）、`core:shell:execute`（执行命令）。所有文件操作通过 Rust 命令实现，不暴露给前端。

---

## 五、前端 Vue 3 实现规范

### 5.1 设计语言：暗黑高密度终端美学

`src/design/tokens.css`：

```css
:root {
  /* ============================================
   * 核心调色板：以暗黑为唯一主题
   * 参考：Warp Terminal / GitHub Dark Dimmed
   * ============================================ */

  /* 背景层次（从深到浅，5层） */
  --bg-base:       #0d1117;   /* 最底层，窗口背景 */
  --bg-surface:    #161b22;   /* 面板、侧边栏 */
  --bg-elevated:   #1c2128;   /* 卡片、列表项悬停 */
  --bg-overlay:    #22272e;   /* 弹窗、下拉 */
  --bg-input:      #2d333b;   /* 输入框 */

  /* 边框 */
  --border-subtle:  rgba(255,255,255,0.05);
  --border-default: rgba(255,255,255,0.10);
  --border-strong:  rgba(255,255,255,0.20);
  --border-accent:  #388bfd;

  /* 文字层次 */
  --text-primary:   #e6edf3;
  --text-secondary: #8b949e;
  --text-tertiary:  #484f58;
  --text-disabled:  #30363d;
  --text-accent:    #79c0ff;
  --text-danger:    #f85149;
  --text-success:   #3fb950;
  --text-warning:   #d29922;

  /* 品牌色：克制的蓝（不用紫，避免俗气） */
  --accent-blue:    #388bfd;
  --accent-blue-dim: rgba(56, 139, 253, 0.15);
  --accent-blue-glow: rgba(56, 139, 253, 0.08);

  /* 语义色 */
  --color-success: #3fb950;
  --color-danger:  #f85149;
  --color-warning: #d29922;
  --color-info:    #79c0ff;

  /* 条目类型色（用于图标背景，低饱和度） */
  --type-login:   rgba(56,  139, 253, 0.15);  /* 蓝 */
  --type-api:     rgba(210, 153,  34, 0.15);  /* 琥珀 */
  --type-ssh:     rgba(63,  185,  80, 0.15);  /* 绿 */
  --type-server:  rgba(188,  140, 255, 0.15); /* 紫 */
  --type-note:    rgba(139, 148, 158, 0.15);  /* 灰 */
  --type-card:    rgba(248,  81,  73, 0.15);  /* 红 */
  --type-identity:rgba(255, 166,  87, 0.15);  /* 橙 */
  --type-crypto:  rgba(86,  211, 255, 0.15);  /* 青 */

  /* ============================================
   * 字体系统
   * ============================================ */
  --font-sans: 'Geist', 'Inter', -apple-system, sans-serif;
  --font-mono: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', monospace;

  /* 字体大小 */
  --text-xs:   11px;
  --text-sm:   12px;
  --text-base: 13px;   /* 主体文字用 13px，高密度关键 */
  --text-md:   14px;
  --text-lg:   16px;
  --text-xl:   20px;

  /* ============================================
   * 间距（紧凑系统）
   * ============================================ */
  --space-1:  3px;
  --space-2:  6px;
  --space-3:  8px;
  --space-4: 12px;
  --space-5: 16px;
  --space-6: 20px;
  --space-8: 28px;

  /* ============================================
   * 圆角（克制，不过度圆润）
   * ============================================ */
  --radius-sm:  4px;
  --radius-md:  6px;
  --radius-lg:  8px;
  --radius-xl: 12px;

  /* ============================================
   * 布局尺寸
   * ============================================ */
  --titlebar-height:     30px;   /* 自定义无边框标题栏 */
  --sidebar-width:      200px;   /* 左侧导航 */
  --list-item-height:    52px;   /* 条目行高（高密度） */
  --detail-panel-width: 360px;   /* 右侧详情面板 */

  /* ============================================
   * 动画
   * ============================================ */
  --ease-out-quart: cubic-bezier(0.25, 1, 0.5, 1);
  --ease-spring:    cubic-bezier(0.34, 1.56, 0.64, 1);
  --duration-fast:  120ms;
  --duration-base:  200ms;
  --duration-slow:  300ms;
}
```

`src/design/typography.css`：

```css
/* 
 * 字体加载：Geist Sans（现代感）+ JetBrains Mono（代码/密码显示）
 * Geist 由 Vercel 开源，视觉上比 Inter 更有辨识度
 */
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500&display=swap');

/* 或者从 npm 加载 Geist */
/* import '@fontsource/geist-sans'; */

body {
  font-family: var(--font-sans);
  font-size: var(--text-base);
  line-height: 1.5;
  color: var(--text-primary);
  -webkit-font-smoothing: antialiased;
}

/* 密码/Key 字段使用等宽字体，提高可读性 */
.font-mono {
  font-family: var(--font-mono);
  font-feature-settings: 'liga' 1, 'calt' 1;
  letter-spacing: 0.02em;
}
```

### 5.2 核心 IPC 桥接层 `src/bridge/tauri.ts`

```typescript
import { invoke } from '@tauri-apps/api/core';

// 会话令牌（解锁后存在内存中）
let _sessionToken: string | null = null;

export function setSessionToken(token: string) {
  _sessionToken = token;
}

export function clearSessionToken() {
  _sessionToken = null;
}

function getToken(): string {
  if (!_sessionToken) throw new Error('未解锁');
  return _sessionToken;
}

// ---- 认证 ----
export const auth = {
  isInitialized: () => invoke<boolean>('is_initialized'),
  setup: (password: string) => invoke<string>('setup', { password }),
  unlock: (password: string) => invoke<string>('unlock', { password }),
  lock: () => invoke<void>('lock'),
  changePassword: (oldPassword: string, newPassword: string) =>
    invoke<void>('change_password', { sessionToken: getToken(), oldPassword, newPassword }),
};

// ---- 金库 ----
export const vault = {
  listEntries: () => invoke<EntryMeta[]>('list_entries', { sessionToken: getToken() }),
  
  getEntrySecrets: (entryId: string) =>
    invoke<EntrySecrets>('get_entry_secrets', { sessionToken: getToken(), entryId }),
  
  createEntry: (input: CreateEntryInput) =>
    invoke<string>('create_entry', { sessionToken: getToken(), input }),
  
  updateEntry: (entryId: string, input: Partial<CreateEntryInput>) =>
    invoke<void>('update_entry', { sessionToken: getToken(), entryId, input }),
  
  deleteEntry: (entryId: string) =>
    invoke<void>('delete_entry', { sessionToken: getToken(), entryId }),
  
  toggleFavorite: (entryId: string) =>
    invoke<void>('toggle_favorite', { sessionToken: getToken(), entryId }),
  
  searchEntries: (query: string) =>
    invoke<EntryMeta[]>('search_entries', { sessionToken: getToken(), query }),
};

// ---- 安全 ----
export const security = {
  checkBreach: (password: string) =>
    invoke<BreachResult>('check_password_breach', { sessionToken: getToken(), password }),
  
  generatePassword: (options: PasswordOptions) =>
    invoke<string>('generate_password', { sessionToken: getToken(), options }),
};

// ---- TypeScript 类型（对应 Rust 结构体）----
export interface EntryMeta {
  id: string;
  entryType: string;
  title: string;
  subtitle?: string;
  tags: string[];
  favorited: boolean;
  groupId?: string;
  updatedAt: number;
}

export interface EntrySecrets {
  fields: DecryptedField[];
}

export interface DecryptedField {
  fieldKey: string;
  fieldType: string;
  value: string;
  isSensitive: boolean;
}

export interface CreateEntryInput {
  entryType: string;
  title: string;
  subtitle?: string;
  tags: string[];
  groupId?: string;
  fields: FieldInput[];
}

export interface FieldInput {
  fieldKey: string;
  fieldType: string;
  value: string;
  isSensitive: boolean;
}

export interface BreachResult {
  breached: boolean;
  count: number;
}

export interface PasswordOptions {
  length: number;
  uppercase: boolean;
  lowercase: boolean;
  numbers: boolean;
  symbols: boolean;
  excludeAmbiguous: boolean;
  mode: 'random' | 'diceware';
}
```

### 5.3 关键 UI 组件实现要点

#### 主界面三栏布局 `src/views/VaultView.vue`

```
窗口（100vw × 100vh，无边框）
┌─────────────────────────────────────────────────────┐
│ [自定义标题栏，30px，drag区域，窗口控制按钮]          │ ← data-tauri-drag-region
├─────────┬───────────────────────┬───────────────────┤
│ 侧边栏   │ 条目列表              │ 详情面板           │
│ 200px   │ flex-1               │ 360px             │
│         │                      │ (可关闭，滑入)      │
│ 分组     │ [搜索框，始终可见]     │                   │
│ 标签     │                      │ 条目标题           │
│         │ 虚拟滚动列表           │ 字段列表           │
│         │ 每项 52px             │ (点击显示/隐藏)    │
│         │                      │                   │
│ [+ 添加] │                      │ [编辑] [删除]      │
└─────────┴───────────────────────┴───────────────────┘
```

关键实现：
- 标题栏：`data-tauri-drag-region` 属性，自定义关闭/最小化按钮
- 列表：使用 `@vueuse/core` 的 `useVirtualList`（每项 52px）
- 详情面板：`translateX(100%)` → `translateX(0)` 滑入，`transition: 220ms cubic-bezier(0.25,1,0.5,1)`
- 搜索框：始终显示在列表顶部，输入时实时调用 `vault.searchEntries()`
- 状态键盘导航：↑↓ 箭头在列表中移动焦点，Enter 打开详情，Esc 关闭详情

#### 条目行 `src/components/vault/VaultItem.vue`

```
┌─────────────────────────────────────────────────────┐
│ [类型图标]  条目标题                    [复制] [⭐]  │ ← 52px 高
│  24×24      副标题（url/host）           hover显示    │
└─────────────────────────────────────────────────────┘
```

视觉规范：
- 类型图标：`24×24px` 圆角方块，背景色用 `--type-{type}`，图标用 Lucide
- 标题：`13px var(--text-primary) font-weight: 500`
- 副标题：`11px var(--text-secondary)`，超长截断
- 选中态：`background: var(--bg-elevated)` + 左侧 `2px solid var(--accent-blue)`
- 悬停态：复制按钮和收藏按钮淡入（`opacity: 0 → 1`）
- 泄露标记：右上角 `6px` 红点，带 `title="密码可能已泄露"` tooltip

#### 命令面板 `src/components/search/CommandPalette.vue`

触发：`Ctrl+K` / `Cmd+K`

```
┌─────────────────────────────────────────────────────┐
│ > _                                                  │ ← 模糊搜索框
├─────────────────────────────────────────────────────┤
│ 操作                                                 │
│  + 新建密码    ⌘N                                    │
│  ⊕ 新建 API Key                                     │
│  🔒 锁定       ⌘L                                   │
│  ↑ 同步        ⌘S                                   │
├─────────────────────────────────────────────────────┤
│ 最近使用                                             │
│  🌐 GitHub · github.com                             │
│  🔑 OpenAI API · api.openai.com                     │
└─────────────────────────────────────────────────────┘
```

输入 `>` 后切换到命令模式，否则为搜索模式。结果用键盘导航。

#### 密码/Key 字段显示

所有敏感字段（`isSensitive: true`）默认显示 `••••••••`，点击眼睛图标临时显示：
- 使用等宽字体（`var(--font-mono)`, `13px`）
- 长 Key（API Key、私钥等）支持横向滚动，不换行
- 右侧有复制按钮，点击后显示 `✓ 已复制`，3 秒后复原
- SSH 私钥等多行内容：`<pre>` 展示，带语法高亮边框

### 5.4 Pinia Store 设计原则（安全隔离）

```typescript
// src/stores/vault.ts
// 只存元数据，绝不缓存解密后的字段值

export const useVaultStore = defineStore('vault', () => {
  const entries = ref<EntryMeta[]>([]);
  const selectedId = ref<string | null>(null);
  
  // 详情面板：按需从 IPC 获取，组件卸载时清空
  // 不存入 store！由 ItemDetail.vue 组件自己管理局部 state
  
  async function loadEntries() {
    entries.value = await vault.listEntries();
  }
  
  async function search(query: string) {
    if (!query.trim()) {
      entries.value = await vault.listEntries();
    } else {
      entries.value = await vault.searchEntries(query);
    }
  }
  
  return { entries, selectedId, loadEntries, search };
});
```

```typescript
// src/components/vault/ItemDetail.vue (局部状态，不入 store)
const secrets = ref<EntrySecrets | null>(null);
const isLoading = ref(false);

async function loadSecrets() {
  if (!props.entryId) return;
  isLoading.value = true;
  try {
    secrets.value = await vault.getEntrySecrets(props.entryId);
  } finally {
    isLoading.value = false;
  }
}

// 组件卸载时清空（尽快释放解密数据）
onUnmounted(() => {
  secrets.value = null;
});
```

---

## 六、浏览器扩展完整重构

### 6.1 架构说明

扩展通过 Native Messaging 协议与 Tauri 主进程通信。Tauri 侧需要注册一个 Native Messaging Host：

```
浏览器扩展 ←→ Chrome Native Messaging ←→ Tauri Rust 进程
              (stdio JSON 协议)
```

这意味着：
1. 扩展永远不持有任何解密密钥或明文密码
2. 每次请求都是一次 IPC 调用，由主进程验证 session 后执行
3. 主进程未运行 = 扩展无法工作（设计如此，不降级）

### 6.2 `extension/manifest.json`（Manifest V3）

```json
{
  "manifest_version": 3,
  "name": "KeyVault",
  "version": "3.0.0",
  "description": "本地密码管理器 · 安全自动填充",
  "permissions": [
    "activeTab",
    "scripting",
    "storage",
    "nativeMessaging"
  ],
  "host_permissions": [],
  "background": {
    "service_worker": "background/service-worker.js",
    "type": "module"
  },
  "action": {
    "default_popup": "popup/popup.html",
    "default_icon": {
      "16": "icons/16.png",
      "48": "icons/48.png",
      "128": "icons/128.png"
    }
  },
  "content_scripts": [
    {
      "matches": ["<all_urls>"],
      "js": ["content/detector.js", "content/filler.js"],
      "css": ["content/save-prompt.css"],
      "run_at": "document_idle",
      "all_frames": false
    }
  ],
  "content_security_policy": {
    "extension_pages": "script-src 'self'; object-src 'none';"
  },
  "native_messaging_hosts": ["com.keyvault.app"]
}
```

### 6.3 `extension/background/service-worker.js`

```javascript
// Native Messaging 连接管理
let nativePort = null;
let pendingRequests = new Map(); // requestId -> { resolve, reject, timeout }
let requestCounter = 0;

function getOrCreateNativePort() {
  if (nativePort) return nativePort;
  
  try {
    nativePort = chrome.runtime.connectNative('com.keyvault.app');
    
    nativePort.onMessage.addListener((message) => {
      const { requestId, ...data } = message;
      const pending = pendingRequests.get(requestId);
      if (pending) {
        clearTimeout(pending.timeout);
        pendingRequests.delete(requestId);
        if (data.error) {
          pending.reject(new Error(data.error));
        } else {
          pending.resolve(data);
        }
      }
    });
    
    nativePort.onDisconnect.addListener(() => {
      nativePort = null;
      // 拒绝所有待处理请求
      for (const [id, pending] of pendingRequests) {
        clearTimeout(pending.timeout);
        pending.reject(new Error('KeyVault 连接断开'));
      }
      pendingRequests.clear();
    });
    
    return nativePort;
  } catch {
    return null;
  }
}

async function sendToNative(action, payload = {}) {
  const port = getOrCreateNativePort();
  if (!port) throw new Error('无法连接 KeyVault，请确保应用已启动');
  
  return new Promise((resolve, reject) => {
    const requestId = ++requestCounter;
    const timeout = setTimeout(() => {
      pendingRequests.delete(requestId);
      reject(new Error('请求超时'));
    }, 5000);
    
    pendingRequests.set(requestId, { resolve, reject, timeout });
    port.postMessage({ requestId, action, ...payload });
  });
}

// 监听来自 content script 和 popup 的消息
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  handleMessage(message, sender)
    .then(sendResponse)
    .catch(err => sendResponse({ error: err.message }));
  return true; // 异步响应
});

async function handleMessage(message, sender) {
  switch (message.action) {
    case 'GET_STATUS':
      try {
        return await sendToNative('get_status');
      } catch {
        return { status: 'disconnected' };
      }
    
    case 'FIND_CREDENTIALS':
      return await sendToNative('find_credentials', { url: message.url });
    
    case 'FILL_CREDENTIAL':
      // content script 请求填充特定条目
      return await sendToNative('get_entry_for_fill', { entryId: message.entryId });
    
    case 'SAVE_CREDENTIAL':
      return await sendToNative('save_credential', {
        url: message.url,
        username: message.username,
        password: message.password,
        title: message.title,
      });
    
    default:
      throw new Error(`未知操作: ${message.action}`);
  }
}
```

### 6.4 `extension/content/detector.js`

```javascript
class FormDetector {
  constructor() {
    this.detectedGroups = new Set();
    this.observer = null;
  }

  getFormGroups() {
    const groups = [];
    
    // 策略1：标准 password input
    document.querySelectorAll('input[type="password"]').forEach(pwdInput => {
      if (pwdInput.closest('[data-kv-handled]')) return;
      
      const usernameField = this.findUsernameField(pwdInput);
      const key = this.getElementKey(pwdInput);
      
      if (!this.detectedGroups.has(key)) {
        this.detectedGroups.add(key);
        groups.push({ passwordField: pwdInput, usernameField });
      }
    });
    
    // 策略2：autocomplete 提示
    document.querySelectorAll('[autocomplete="current-password"],[autocomplete="new-password"]')
      .forEach(input => {
        if (input.type === 'password') return; // 已在策略1处理
        const key = this.getElementKey(input);
        if (!this.detectedGroups.has(key)) {
          this.detectedGroups.add(key);
          groups.push({ passwordField: input, usernameField: this.findUsernameField(input) });
        }
      });
    
    return groups;
  }
  
  findUsernameField(referenceInput) {
    // 向上查找最近的 form 或容器
    const container = referenceInput.closest('form') || 
                      referenceInput.closest('[class*="login"],[class*="signin"],[class*="auth"]') ||
                      referenceInput.parentElement?.parentElement;
    
    if (!container) return null;
    
    // 优先级查找
    return container.querySelector('input[type="email"]') ||
           container.querySelector('input[autocomplete="username"],input[autocomplete="email"]') ||
           container.querySelector('input[name*="user" i],input[name*="email" i],input[name*="login" i]') ||
           container.querySelector('input[type="text"]');
  }
  
  getElementKey(el) {
    return el.id || el.name || `${el.offsetTop}-${el.offsetLeft}`;
  }
  
  watchDynamic() {
    this.observer = new MutationObserver(() => {
      const newGroups = this.getFormGroups();
      if (newGroups.length > 0) {
        // 通知 background：页面出现了新的登录表单
        chrome.runtime.sendMessage({ 
          action: 'FORMS_DETECTED', 
          url: location.href,
          count: newGroups.length 
        });
      }
    });
    
    this.observer.observe(document.body, { childList: true, subtree: true });
  }
}

const detector = new FormDetector();

// 初始检测
const initialGroups = detector.getFormGroups();
if (initialGroups.length > 0) {
  // 查询匹配的凭据
  chrome.runtime.sendMessage({ action: 'FIND_CREDENTIALS', url: location.href })
    .then(result => {
      if (result?.entries?.length > 0) {
        // 注入填充按钮（非侵入式）
        injectFillHints(initialGroups, result.entries);
      }
    });
}

// 监听 SPA 动态加载
detector.watchDynamic();
```

### 6.5 Popup UI 规范 `extension/popup/`

popup 宽度 `360px`，最大高度 `540px`，使用和主应用完全一致的设计语言（暗黑高密度）：

```
┌────────────────────────────────────────┐
│ 🔐 KeyVault               ● 已连接     │  30px 标题栏
├────────────────────────────────────────┤
│ 🔍 搜索密码...                          │  36px 搜索框
├────────────────────────────────────────┤
│ 当前页面                                │  section 标题
│                                        │
│ ┌──────────────────────────────────┐   │
│ │ 🌐 GitHub                       │   │  52px 每项
│ │ ismoyuai@gmail.com               │   │
│ │             [填充]  [复制密码]    │   │
│ └──────────────────────────────────┘   │
│                                        │
│ ┌──────────────────────────────────┐   │
│ │ 🌐 GitHub (工作)                 │   │
│ │ work@company.com                 │   │
│ │             [填充]  [复制密码]    │   │
│ └──────────────────────────────────┘   │
├────────────────────────────────────────┤
│ [打开应用]    [生成密码]    [设置]      │  40px 底部栏
└────────────────────────────────────────┘
```

状态变体：
- **未连接**：显示"请先启动 KeyVault 应用" + 图标说明
- **已连接但锁定**：显示"KeyVault 已锁定" + "点击解锁" 按钮（调起主窗口）
- **无匹配**：显示"当前页面无匹配凭据" + 搜索框（可搜索全库）

---

## 七、执行步骤（严格按顺序）

```
Step 1   创建新目录 keyvault-v3/，不在原仓库内修改
         cd ~ && mkdir keyvault-v3 && cd keyvault-v3

Step 2   初始化 Tauri + Vue 3 + TypeScript 项目
         npm create tauri-app@latest . -- --template vue-ts
         选择：Vue, TypeScript, Vite

Step 3   配置 Rust 依赖
         修改 src-tauri/Cargo.toml，添加所有依赖（见 1.1 节）

Step 4   实现 Rust 后端（按顺序）
         4a. error.rs（统一错误类型）
         4b. crypto/（kdf.rs → cipher.rs → session.rs）
         4c. db/migrations/（SQL文件）
         4d. db/（schema.rs → queries.rs）
         4e. state.rs（AppState）
         4f. commands/auth.rs
         4g. commands/vault.rs
         4h. commands/generator.rs
         4i. commands/breach.rs
         4j. commands/sync.rs（WebDAV，参考原仓库逻辑）
         4k. main.rs（注册所有命令，初始化连接池）
         运行 cargo build 验证编译通过

Step 5   配置 Tauri（tauri.conf.json）
         5a. 设置无边框窗口（decorations: false）
         5b. 配置最小窗口大小（900×600）
         5c. 配置系统托盘
         5d. 设置能力（capabilities/default.json，最小权限）

Step 6   实现前端设计系统
         6a. src/design/tokens.css（完整设计令牌）
         6b. src/design/typography.css
         6c. src/design/animations.css
         6d. 配置 Geist + JetBrains Mono 字体加载
         6e. 移除 Vite 默认样式，应用设计系统

Step 7   实现基础 UI 组件库 src/components/ui/
         Button.vue、Input.vue、Modal.vue、Tooltip.vue、Toast.vue
         每个组件完整实现，包含所有变体

Step 8   实现 IPC 桥接层 src/bridge/tauri.ts（完整类型定义）

Step 9   实现 Pinia Stores
         auth.ts → settings.ts → vault.ts → ui.ts

Step 10  实现页面组件（按依赖顺序）
         10a. UnlockView.vue（首优先，最关键的第一印象）
         10b. SetupView.vue（首次设置向导）
         10c. VaultView.vue（主界面三栏布局）
         10d. SettingsView.vue

Step 11  实现复杂功能组件
         11a. CommandPalette.vue（⌘K，键盘导航）
         11b. VaultItem.vue + VaultList.vue（虚拟滚动）
         11c. ItemDetail.vue（侧滑面板，按需加载 secrets）
         11d. ItemForm.vue（智能表单，根据 entryType 动态字段）
         11e. PasswordGenerator.vue（含强度可视化）
         11f. ClipboardTimer.vue（倒计时清空）

Step 12  实现浏览器扩展
         12a. manifest.json
         12b. background/service-worker.js
         12c. content/detector.js + filler.js + save-prompt.js
         12d. popup/（HTML + CSS + JS）
         12e. Native Messaging Host（Rust 侧）

Step 13  数据迁移工具（从旧仓库导入数据）
         读取旧版 sql.js 数据库（参考原仓库 import 模块）
         解密后用新的 SQLCipher 重新加密存储

Step 14  测试
         cargo test（Rust 单元测试）
         重点测试：encrypt/decrypt 往返、session 过期、breach k-匿名性

Step 15  构建验证
         cargo tauri build（Windows 目标）
         验证安装包大小 < 20MB
         验证内存占用空闲 < 60MB
```

---

## 八、安全红线（不可违反）

以下红线在迁移过程中必须严格遵守，任何步骤都不得违反：

1. **主密码零存储**：只存 Argon2id 哈希，原始密码在函数返回后立即被 `Zeroizing` 清零
2. **参数锁定**：`memoryCost=65536, timeCost=3, parallelism=1`，不可在任何路径降级
3. **密钥不落盘**：`AppState.encryption_key` 是内存中的 `Zeroizing<[u8;32]>`，任何时候不写入文件
4. **锁定时清零**：`AppState::lock()` 必须同时：清零 encryption_key + 销毁所有 sessions
5. **自动内存清零**：所有持有密钥或明文的变量必须使用 `Zeroizing<T>` 包装
6. **字段级加密**：每个敏感字段独立加密（独立 nonce），不共用加密块
7. **前端零缓存**：`vault store` 只存元数据，`ItemDetail.vue` 卸载时立即清空 secrets
8. **扩展零持久化**：浏览器扩展不使用 `chrome.storage` 存储任何解密数据
9. **泄露检测 k-匿名**：只向 HIBP 发送 SHA1 前5字符，绝不发送完整哈希或密码原文
10. **SQLCipher 密钥**：数据库文件本身 AES-256 加密，密钥派生自主密码的派生密钥（非直接使用 encryption_key，而是 HKDF 派生的子密钥）
11. **Tauri 最小权限**：不开启 `core:fs:*`，所有文件操作在 Rust 命令中实现
12. **IPC 全部验证 session**：所有涉及解密的命令必须先验证 session token

---

## 九、验收标准

完成后，以下所有条件必须满足：

**安全**
- [ ] `cargo audit` 无高危漏洞
- [x] `cargo test` 全部通过（26 项单元测试）
- [ ] 主密码错误时 Argon2id 验证耗时 ≥ 1 秒（暴力破解抗性）
- [ ] 锁定后内存中无可读的明文密码（通过 strings 工具验证）
- [ ] 泄露检测仅发送哈希前5字符（Wireshark 抓包确认）

**性能**
- [ ] 冷启动时间 < 2 秒（Windows）
- [ ] 空闲内存 < 60 MB
- [ ] 安装包大小 < 20 MB
- [ ] 搜索响应 < 50ms（1000 条目）

**功能**
- [x] 首次设置向导完整可用
- [x] 条目 CRUD 全部正常
- [x] 命令面板 `Ctrl+K` 正常工作，键盘导航流畅
- [x] 密码生成器：随机模式 + Diceware 模式
- [ ] 浏览器扩展：检测主流网站（Google、GitHub、Twitter）登录表单 — **暂停，待桌面端验收后单独开发**
- [ ] 浏览器扩展：自动填充用户名 + 密码 — **暂停**
- [ ] 浏览器扩展：保存提示（非侵入式顶部通知条） — **暂停**
- [x] 剪贴板：复制后 30 秒倒计时清空
- [x] 自动锁定：空闲超时自动锁定
- [x] 数据导入/导出（JSON）
- [x] WebDAV 同步（上传/下载/合并）
- [x] 紧急擦除

**UI/UX**
- [ ] 三栏布局在 1280px 以上正常
- [ ] 侧滑详情面板动画流畅（60fps）
- [ ] 所有交互有明确的 loading 状态
- [ ] 错误信息友好，不暴露技术细节给用户
- [ ] 无边框窗口，标题栏拖动区域正常工作

---

## 十、原仓库参考

迁移过程中，以下原仓库模块的**逻辑**可参考，但**不复制代码**（模块系统不同）：

原项目根目录路径：`D:\keyvault`

- `electron/sync/` → Rust 侧 WebDAV 同步逻辑（使用 `webdav-client` Rust crate）
- `electron/import/` → CSV 解析逻辑（参考字段映射规则）
- `src/constants/templates.js` → 条目类型和字段定义（转为 TypeScript 类型 + Rust 枚举）
- `extension/content.js` → 表单检测逻辑（重写为更强壮的版本）

---

*此文档由 Claude (claude.ai) 根据用户偏好（暗黑高密度 + 开发者场景 + Windows 主力 + 完整迁移 Tauri+Rust）生成。*  
*生成时间：2026-06-03*
