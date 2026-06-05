# KeyVault v3 增量式实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 从零搭建 Tauri 2.0 + Rust 后端，实现最小加密闭环（setup → unlock → CRUD → lock）

**Architecture:** 增量式实现，每步可验证。Rust 后端负责加密/存储/IPC，Vue 3 前端通过 Tauri invoke() 调用后端命令。Phase 0-1 搭建脚手架，Phase 2 实现加密核心，Phase 3 接通数据库和认证，Phase 4 实现金库 CRUD。

**Tech Stack:** Tauri 2.0, Rust (argon2, aes-gcm, sqlx, SQLCipher), Vue 3, TypeScript, Vite 6, Pinia

---

## Phase 0: 环境准备

### Task 0.1: 安装 Rust Toolchain

**Files:** 无（系统级安装）

- [ ] **Step 1: 下载并安装 rustup**

访问 https://win.rustup.rs/x86_64 下载安装程序，或执行：
```powershell
winget install Rustlang.Rustup
```

安装时选择默认选项（stable toolchain, MSVC ABI）。

- [ ] **Step 2: 验证安装**

**重启终端**（让 PATH 生效），然后运行：
```powershell
rustc --version
cargo --version
```

Expected: 输出类似 `rustc 1.xx.x` 和 `cargo 1.xx.x`。

- [ ] **Step 3: 安装 Tauri CLI**

```powershell
cargo install tauri-cli --version "^2"
```

验证：
```powershell
cargo tauri --version
```

Expected: 输出 `tauri-cli 2.x.x`。

---

## Phase 1: 最小 Tauri 项目

### Task 1.1: 创建 src-tauri/ 基础结构

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建目录结构**

```powershell
mkdir src-tauri\src -Force
mkdir src-tauri\capabilities -Force
```

- [ ] **Step 2: 创建 Cargo.toml**

`src-tauri/Cargo.toml`:
```toml
[package]
name = "keyvault"
version = "3.0.0"
edition = "2021"

[lib]
name = "keyvault_lib"
crate-type = ["lib", "cdylib", "staticlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 3: 创建 build.rs**

`src-tauri/build.rs`:
```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 4: 创建 tauri.conf.json**

`src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-config-schema/schema.json",
  "productName": "KeyVault",
  "version": "3.0.0",
  "identifier": "com.keyvault.app",
  "build": {
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "KeyVault",
        "width": 900,
        "height": 600,
        "minWidth": 900,
        "minHeight": 600,
        "decorations": false,
        "resizable": true,
        "center": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 5: 创建 capabilities/default.json**

`src-tauri/capabilities/default.json`:
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

- [ ] **Step 6: 创建 main.rs**

`src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    keyvault_lib::run()
}
```

- [ ] **Step 7: 创建 lib.rs**

`src-tauri/src/lib.rs`:
```rust
#[tauri::command]
fn greet() -> &'static str {
    "KeyVault v3 is running"
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 8: 验证编译**

```powershell
cd src-tauri && cargo build
```

Expected: 编译成功，无错误。

- [ ] **Step 9: 安装前端依赖并验证 Tauri dev**

```powershell
npm install
cargo tauri dev
```

Expected: 窗口打开，显示 Vue 前端 UnlockView。如果前端报错，检查 vite 是否在 1420 端口运行。

- [ ] **Step 10: 提交**

```powershell
git init
git add .
git commit -m "feat: initialize Tauri 2.0 project with minimal structure"
```

---

## Phase 2: 加密核心

### Task 2.1: 添加加密依赖和错误类型

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/error.rs`

- [ ] **Step 1: 更新 Cargo.toml 添加加密依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中追加：
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

完整 Cargo.toml 应为：
```toml
[package]
name = "keyvault"
version = "3.0.0"
edition = "2021"

[lib]
name = "keyvault_lib"
crate-type = ["lib", "cdylib", "staticlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
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

- [ ] **Step 2: 创建 error.rs**

`src-tauri/src/error.rs`:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("KDF 参数错误")]
    KdfParamError,
    #[error("密钥派生失败")]
    KdfError,
    #[error("密码哈希失败")]
    HashError,
    #[error("密码哈希解析失败")]
    HashParseError,
    #[error("加密初始化失败")]
    CipherInitError,
    #[error("加密失败")]
    EncryptError,
    #[error("解密失败")]
    DecryptError,
    #[error("Base64 解码失败")]
    Base64Error,
    #[error("无效的密文")]
    InvalidCiphertext,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Crypto(#[from] CryptoError),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    #[error("会话已过期")]
    SessionExpired,
    #[error("密码管理器未初始化")]
    NotInitialized,
    #[error("密码管理器已锁定")]
    Locked,
    #[error("密码错误")]
    WrongPassword,
    #[error("{0}")]
    Other(String),
}

// Tauri 命令需要返回 String 错误
impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}
```

- [ ] **Step 3: 验证编译**

```powershell
cd src-tauri && cargo build
```

Expected: 编译成功。

- [ ] **Step 4: 提交**

```powershell
git add src-tauri/
git commit -m "feat: add crypto dependencies and error types"
```

---

### Task 2.2: 实现 Argon2id 密钥派生

**Files:**
- Create: `src-tauri/src/crypto/mod.rs`
- Create: `src-tauri/src/crypto/kdf.rs`

- [ ] **Step 1: 创建 crypto 目录**

```powershell
mkdir src-tauri\src\crypto -Force
```

- [ ] **Step 2: 创建 kdf.rs**

`src-tauri/src/crypto/kdf.rs`:
```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Algorithm, Params, Version,
};
use zeroize::Zeroizing;
use crate::error::CryptoError;

/// Argon2id 参数（锁定，不可降级）
const MEMORY_COST: u32 = 65536; // 64 MB
const TIME_COST: u32 = 3;
const PARALLELISM: u32 = 1;
const OUTPUT_LEN: usize = 32; // 256 bits

/// 派生加密密钥（32 字节），用于 AES-256-GCM
pub fn derive_key(password: &[u8], salt: &[u8; 16]) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let params = Params::new(MEMORY_COST, TIME_COST, PARALLELISM, Some(OUTPUT_LEN))
        .map_err(|_| CryptoError::KdfParamError)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(password, salt, key.as_mut())
        .map_err(|_| CryptoError::KdfError)?;

    Ok(key)
}

/// 哈希主密码用于验证存储（使用 password_hash 格式，包含随机 salt）
pub fn hash_master_password(password: &[u8]) -> Result<String, CryptoError> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(MEMORY_COST, TIME_COST, PARALLELISM, None)
        .map_err(|_| CryptoError::KdfParamError)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let hash = argon2
        .hash_password(password, &salt)
        .map_err(|_| CryptoError::HashError)?
        .to_string();
    Ok(hash)
}

/// 验证主密码
pub fn verify_master_password(password: &[u8], hash: &str) -> Result<bool, CryptoError> {
    let parsed = PasswordHash::new(hash).map_err(|_| CryptoError::HashParseError)?;
    Ok(Argon2::default()
        .verify_password(password, &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let password = b"test_password_123";
        let salt = [0u8; 16];
        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();
        assert_eq!(*key1, *key2);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = [0u8; 16];
        let key1 = derive_key(b"password1", &salt).unwrap();
        let key2 = derive_key(b"password2", &salt).unwrap();
        assert_ne!(*key1, *key2);
    }

    #[test]
    fn test_derive_key_different_salts() {
        let password = b"same_password";
        let salt1 = [0u8; 16];
        let salt2 = [1u8; 16];
        let key1 = derive_key(password, &salt1).unwrap();
        let key2 = derive_key(password, &salt2).unwrap();
        assert_ne!(*key1, *key2);
    }

    #[test]
    fn test_hash_and_verify() {
        let password = b"my_master_password";
        let hash = hash_master_password(password).unwrap();
        assert!(verify_master_password(password, &hash).unwrap());
        assert!(!verify_master_password(b"wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_hash_format() {
        let hash = hash_master_password(b"test").unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }
}
```

- [ ] **Step 3: 创建 crypto/mod.rs**

`src-tauri/src/crypto/mod.rs`:
```rust
pub mod kdf;

pub use kdf::{derive_key, hash_master_password, verify_master_password};
```

- [ ] **Step 4: 更新 lib.rs 引入 crypto 模块**

`src-tauri/src/lib.rs`:
```rust
mod crypto;
mod error;

#[tauri::command]
fn greet() -> &'static str {
    "KeyVault v3 is running"
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: 运行测试**

```powershell
cd src-tauri && cargo test
```

Expected: 所有 5 个 kdf 测试通过。

- [ ] **Step 6: 提交**

```powershell
git add src-tauri/
git commit -m "feat: implement Argon2id key derivation with tests"
```

---

### Task 2.3: 实现 AES-256-GCM 字段加密

**Files:**
- Create: `src-tauri/src/crypto/cipher.rs`
- Modify: `src-tauri/src/crypto/mod.rs`

- [ ] **Step 1: 创建 cipher.rs**

`src-tauri/src/crypto/cipher.rs`:
```rust
use aes_gcm::{
    aead::{Aead, AeadCore, OsRng},
    Aes256Gcm, KeyInit, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use zeroize::Zeroizing;
use crate::error::CryptoError;

/// 加密单个字段，返回 base64 编码的 nonce+ciphertext
/// nonce 每次随机生成（12 字节），prepend 到密文前
pub fn encrypt_field(key: &[u8; 32], plaintext: &[u8]) -> Result<String, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::CipherInitError)?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| CryptoError::EncryptError)?;

    // prepend nonce (12 字节) 到密文前
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(result))
}

/// 解密字段，解密后的数据用 Zeroizing 包装确保使用后清零
pub fn decrypt_field(key: &[u8; 32], encoded: &str) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    let data = BASE64
        .decode(encoded)
        .map_err(|_| CryptoError::Base64Error)?;

    if data.len() < 12 {
        return Err(CryptoError::InvalidCiphertext);
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::CipherInitError)?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptError)?;

    Ok(Zeroizing::new(plaintext))
}

#[cfg(test)]
mod tests {
    use super::super::kdf;

    fn test_key() -> [u8; 32] {
        let salt = [0u8; 16];
        let key = kdf::derive_key(b"test_password", &salt).unwrap();
        *key
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"Hello, KeyVault!";
        let encrypted = encrypt_field(&key, plaintext).unwrap();
        let decrypted = decrypt_field(&key, &encrypted).unwrap();
        assert_eq!(&*decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_different_nonces() {
        let key = test_key();
        let plaintext = b"same data";
        let enc1 = encrypt_field(&key, plaintext).unwrap();
        let enc2 = encrypt_field(&key, plaintext).unwrap();
        // 同一明文加密两次应产生不同密文（因为 nonce 不同）
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = test_key();
        let salt2 = [1u8; 16];
        let key2 = *kdf::derive_key(b"other_password", &salt2).unwrap();

        let encrypted = encrypt_field(&key1, b"secret").unwrap();
        let result = decrypt_field(&key2, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext() {
        let key = test_key();
        let encrypted = encrypt_field(&key, b"secret").unwrap();
        // 篡改 base64 数据
        let mut chars: Vec<char> = encrypted.chars().collect();
        let last = chars.len() - 1;
        chars[last] = if chars[last] == 'A' { 'B' } else { 'A' };
        let tampered: String = chars.into_iter().collect();
        let result = decrypt_field(&key, &tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty() {
        let key = test_key();
        let encrypted = encrypt_field(&key, b"").unwrap();
        let decrypted = decrypt_field(&key, &encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_encrypt_unicode() {
        let key = test_key();
        let plaintext = "你好世界 🔐".as_bytes();
        let encrypted = encrypt_field(&key, plaintext).unwrap();
        let decrypted = decrypt_field(&key, &encrypted).unwrap();
        assert_eq!(&*decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let key = test_key();
        let result = decrypt_field(&key, "not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_too_short() {
        let key = test_key();
        let result = decrypt_field(&key, "AQID"); // 3 bytes, too short
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: 更新 crypto/mod.rs**

`src-tauri/src/crypto/mod.rs`:
```rust
pub mod cipher;
pub mod kdf;

pub use cipher::{decrypt_field, encrypt_field};
pub use kdf::{derive_key, hash_master_password, verify_master_password};
```

- [ ] **Step 3: 运行测试**

```powershell
cd src-tauri && cargo test
```

Expected: kdf 5 个测试 + cipher 8 个测试 = 13 个测试全部通过。

- [ ] **Step 4: 提交**

```powershell
git add src-tauri/
git commit -m "feat: implement AES-256-GCM field encryption with tests"
```

---

### Task 2.4: 实现会话管理和应用状态

**Files:**
- Create: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 state.rs**

`src-tauri/src/state.rs`:
```rust
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use rand::Rng;
use tokio::sync::RwLock;
use zeroize::Zeroizing;

const SESSION_TTL_SECS: u64 = 1800; // 30 分钟

/// 会话令牌管理器
pub struct SessionManager {
    sessions: RwLock<HashMap<String, SystemTime>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// 创建新会话，返回 64 字符随机 token
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

    /// 验证会话，滑动窗口刷新时间
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

    /// 销毁所有会话
    pub async fn destroy_all(&self) {
        self.sessions.write().await.clear();
    }
}

/// 应用全局状态
pub struct AppState {
    /// 加密密钥，仅在解锁后存在，锁定时清零
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
    /// 会话令牌管理器
    pub sessions: SessionManager,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            encryption_key: RwLock::new(None),
            sessions: SessionManager::new(),
        }
    }

    /// 锁定：清零加密密钥 + 销毁所有会话
    pub async fn lock(&self) {
        let mut key = self.encryption_key.write().await;
        *key = None; // Zeroizing 在 drop 时自动清零
        self.sessions.destroy_all().await;
    }

    /// 检查是否已解锁
    pub async fn is_unlocked(&self) -> bool {
        self.encryption_key.read().await.is_some()
    }
}
```

- [ ] **Step 2: 更新 lib.rs 引入 state 模块**

`src-tauri/src/lib.rs`:
```rust
mod crypto;
mod error;
mod state;

use state::AppState;

#[tauri::command]
fn greet() -> &'static str {
    "KeyVault v3 is running"
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: 验证编译**

```powershell
cd src-tauri && cargo build
```

Expected: 编译成功。

- [ ] **Step 4: 运行 `cargo tauri dev` 验证窗口正常打开**

Expected: 窗口打开，前端显示正常。

- [ ] **Step 5: 提交**

```powershell
git add src-tauri/
git commit -m "feat: implement session manager and application state"
```

---

## Phase 3: 数据库 + 认证命令

### Task 3.1: 配置 SQLCipher 依赖和数据库初始化

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/migrations/001_init.sql`

- [ ] **Step 1: 更新 Cargo.toml 添加数据库依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中追加：
```toml
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }
libsqlite3-sys = { version = "0.30", features = ["bundled-sqlcipher"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: 创建 migrations 目录和 SQL 文件**

```powershell
mkdir src-tauri\src\db\migrations -Force
```

`src-tauri/src/db/migrations/001_init.sql`:
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
    entry_type  TEXT NOT NULL,
    title       TEXT NOT NULL,
    subtitle    TEXT,
    tags        TEXT,
    favorited   INTEGER NOT NULL DEFAULT 0,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

-- 加密字段
CREATE TABLE IF NOT EXISTS fields (
    id           TEXT PRIMARY KEY,
    entry_id     TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    field_key    TEXT NOT NULL,
    field_type   TEXT NOT NULL DEFAULT 'text',
    enc_value    TEXT NOT NULL,
    is_sensitive INTEGER NOT NULL DEFAULT 1,
    sort_order   INTEGER NOT NULL DEFAULT 0
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_entries_type      ON entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_entries_group     ON entries(group_id);
CREATE INDEX IF NOT EXISTS idx_entries_favorited ON entries(favorited);
CREATE INDEX IF NOT EXISTS idx_fields_entry      ON fields(entry_id);
```

- [ ] **Step 3: 创建 db/mod.rs**

`src-tauri/src/db/mod.rs`:
```rust
pub mod migrations;

use sqlx::sqlite::{SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;
use zeroize::Zeroizing;

/// 初始化 SQLCipher 数据库连接池
/// db_path: 数据库文件路径
/// key: 加密密钥（32 字节）
pub async fn init_pool(db_path: &PathBuf, key: &[u8; 32]) -> Result<SqlitePool, sqlx::Error> {
    // 确保父目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    // 设置 SQLCipher 密钥
    let key_hex = hex::encode(key);
    sqlx::query(&format!("PRAGMA key = \"x'{}'\"", key_hex))
        .execute(&pool)
        .await?;

    // 验证密钥正确（尝试读取）
    sqlx::query("SELECT count(*) FROM sqlite_master")
        .execute(&pool)
        .await?;

    // 运行迁移
    migrations::run(&pool).await?;

    Ok(pool)
}

/// 初始化未加密的数据库（用于首次 setup，存储密码哈希和 salt）
pub async fn init_plain_pool(db_path: &PathBuf) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    migrations::run(&pool).await?;

    Ok(pool)
}
```

- [ ] **Step 4: 创建 db/migrations/mod.rs**

`src-tauri/src/db/migrations/mod.rs`:
```rust
use sqlx::SqlitePool;

/// 运行所有数据库迁移
pub async fn run(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("001_init.sql"))
        .execute(pool)
        .await?;
    Ok(())
}
```

- [ ] **Step 5: 更新 lib.rs 引入 db 模块**

`src-tauri/src/lib.rs`:
```rust
mod crypto;
mod db;
mod error;
mod state;

use state::AppState;

#[tauri::command]
fn greet() -> &'static str {
    "KeyVault v3 is running"
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: 验证编译**

```powershell
cd src-tauri && cargo build
```

Expected: 编译成功。首次编译 SQLCipher 可能需要几分钟。

- [ ] **Step 7: 提交**

```powershell
git add src-tauri/
git commit -m "feat: add SQLCipher database layer with migrations"
```

---

### Task 3.2: 更新 AppState 集成数据库

**Files:**
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: 更新 state.rs 添加数据库连接池**

`src-tauri/src/state.rs` 完整替换为：
```rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use rand::Rng;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use zeroize::Zeroizing;
use crate::db;

const SESSION_TTL_SECS: u64 = 1800; // 30 分钟

/// 会话令牌管理器
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

/// 应用全局状态
pub struct AppState {
    /// 加密密钥，仅在解锁后存在
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
    /// 数据库连接池（可能为 None，在 setup/unlock 后初始化）
    pub db: RwLock<Option<SqlitePool>>,
    /// 会话管理器
    pub sessions: SessionManager,
    /// 数据库文件路径
    pub db_path: PathBuf,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            encryption_key: RwLock::new(None),
            db: RwLock::new(None),
            sessions: SessionManager::new(),
            db_path: app_data_dir.join("keyvault.db"),
        }
    }

    /// 锁定：清零密钥、关闭数据库、销毁会话
    pub async fn lock(&self) {
        let mut key = self.encryption_key.write().await;
        *key = None;
        let mut db = self.db.write().await;
        if let Some(pool) = db.take() {
            pool.close().await;
        }
        self.sessions.destroy_all().await;
    }

    pub async fn is_unlocked(&self) -> bool {
        self.encryption_key.read().await.is_some()
    }

    /// 用加密密钥打开数据库
    pub async fn open_db(&self, key: &[u8; 32]) -> Result<(), String> {
        let pool = db::init_pool(&self.db_path, key)
            .await
            .map_err(|e| format!("数据库打开失败: {}", e))?;
        let mut db = self.db.write().await;
        *db = Some(pool);
        Ok(())
    }

    /// 用明文打开数据库（首次 setup）
    pub async fn open_plain_db(&self) -> Result<(), String> {
        let pool = db::init_plain_pool(&self.db_path)
            .await
            .map_err(|e| format!("数据库创建失败: {}", e))?;
        let mut db = self.db.write().await;
        *db = Some(pool);
        Ok(())
    }

    /// 获取数据库连接池
    pub async fn get_db(&self) -> Result<SqlitePool, String> {
        let db = self.db.read().await;
        db.as_ref().cloned().ok_or_else(|| "数据库未初始化".to_string())
    }
}
```

- [ ] **Step 2: 更新 lib.rs 使用 app_data_dir**

`src-tauri/src/lib.rs`:
```rust
mod crypto;
mod db;
mod error;
mod state;

use state::AppState;

#[tauri::command]
fn greet() -> &'static str {
    "KeyVault v3 is running"
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            app.manage(AppState::new(app_data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: 验证编译**

```powershell
cd src-tauri && cargo build
```

Expected: 编译成功。

- [ ] **Step 4: 提交**

```powershell
git add src-tauri/
git commit -m "feat: integrate database into AppState with lifecycle management"
```

---

### Task 3.3: 实现认证命令

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/auth.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 commands 目录**

```powershell
mkdir src-tauri\src\commands -Force
```

- [ ] **Step 2: 创建 commands/auth.rs**

`src-tauri/src/commands/auth.rs`:
```rust
use tauri::State;
use rand::RngCore;
use zeroize::Zeroizing;
use crate::crypto;
use crate::state::AppState;

/// 检查是否已初始化（是否设置过主密码）
#[tauri::command]
pub async fn is_initialized(state: State<'_, AppState>) -> Result<bool, String> {
    // 如果数据库文件存在且能读取 config 表，说明已初始化
    if !state.db_path.exists() {
        return Ok(false);
    }

    // 尝试用空密钥打开（未加密的初始数据库）
    let pool = crate::db::init_plain_pool(&state.db_path)
        .await
        .map_err(|e| e.to_string())?;

    let result: Option<String> = sqlx::query_scalar(
        "SELECT value FROM config WHERE key = 'password_hash'"
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    pool.close().await;
    Ok(result.is_some())
}

/// 首次设置主密码
#[tauri::command]
pub async fn setup(
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let password_bytes = Zeroizing::new(password.into_bytes());

    // 1. 生成 KDF salt
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);

    // 2. 派生加密密钥
    let key = crypto::derive_key(&password_bytes, &salt)
        .map_err(|e| e.to_string())?;

    // 3. 哈希主密码用于验证
    let hash = crypto::hash_master_password(&password_bytes)
        .map_err(|e| e.to_string())?;

    // 4. 打开明文数据库（首次创建）
    state.open_plain_db().await?;

    // 5. 存储 salt 和密码哈希
    let db = state.get_db().await?;
    sqlx::query("INSERT INTO config (key, value) VALUES ('kdf_salt', ?), ('password_hash', ?)")
        .bind(hex::encode(salt))
        .bind(hash)
        .execute(&db)
        .await
        .map_err(|e| e.to_string())?;

    // 6. 关闭明文数据库，用加密密钥重新打开
    db.close().await;
    {
        let mut db_guard = state.db.write().await;
        *db_guard = None;
    }
    state.open_db(&key).await?;

    // 7. 激活加密密钥
    *state.encryption_key.write().await = Some(key);

    // 8. 创建会话
    let token = state.sessions.create().await;
    Ok(token)
}

/// 解锁（验证密码后恢复加密密钥）
#[tauri::command]
pub async fn unlock(
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let password_bytes = Zeroizing::new(password.into_bytes());

    // 1. 先用明文打开数据库获取密码哈希
    if !state.db_path.exists() {
        return Err("密码管理器未初始化".to_string());
    }

    let plain_pool = crate::db::init_plain_pool(&state.db_path)
        .await
        .map_err(|e| e.to_string())?;

    // 2. 获取存储的哈希
    let hash: String = sqlx::query_scalar(
        "SELECT value FROM config WHERE key = 'password_hash'"
    )
    .fetch_one(&plain_pool)
    .await
    .map_err(|_| "密码管理器未初始化".to_string())?;

    // 3. 验证密码
    let valid = crypto::verify_master_password(&password_bytes, &hash)
        .map_err(|e| e.to_string())?;

    if !valid {
        plain_pool.close().await;
        return Err("密码错误".to_string());
    }

    // 4. 获取 KDF salt
    let salt_hex: String = sqlx::query_scalar(
        "SELECT value FROM config WHERE key = 'kdf_salt'"
    )
    .fetch_one(&plain_pool)
    .await
    .map_err(|e| e.to_string())?;

    plain_pool.close().await;

    let salt_bytes = hex::decode(&salt_hex).map_err(|e| e.to_string())?;
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&salt_bytes[..16]);

    // 5. 派生密钥
    let key = crypto::derive_key(&password_bytes, &salt)
        .map_err(|e| e.to_string())?;

    // 6. 用加密密钥打开数据库
    state.open_db(&key).await?;

    // 7. 激活密钥
    *state.encryption_key.write().await = Some(key);

    // 8. 创建会话
    let token = state.sessions.create().await;
    Ok(token)
}

/// 锁定
#[tauri::command]
pub async fn lock(state: State<'_, AppState>) -> Result<(), String> {
    state.lock().await;
    Ok(())
}
```

- [ ] **Step 3: 创建 commands/mod.rs**

`src-tauri/src/commands/mod.rs`:
```rust
pub mod auth;
```

- [ ] **Step 4: 更新 lib.rs 注册认证命令**

`src-tauri/src/lib.rs`:
```rust
mod commands;
mod crypto;
mod db;
mod error;
mod state;

use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            app.manage(AppState::new(app_data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::is_initialized,
            commands::auth::setup,
            commands::auth::unlock,
            commands::auth::lock,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: 验证编译**

```powershell
cd src-tauri && cargo build
```

Expected: 编译成功。

- [ ] **Step 6: 运行完整认证流程测试**

```powershell
cargo tauri dev
```

Expected:
1. 窗口打开 → App.vue 检测 `is_initialized` → 跳转 SetupView
2. 输入密码（≥8 字符）→ 确认密码 → 点击"创建密码库"
3. setup 命令成功 → 跳转 VaultView（空状态）
4. 关闭窗口重新打开 → UnlockView
5. 输入正确密码 → VaultView
6. 输入错误密码 → 抖动动画 + 错误提示

- [ ] **Step 7: 提交**

```powershell
git add src-tauri/
git commit -m "feat: implement auth commands (setup, unlock, lock) with SQLCipher"
```

---

## Phase 4: Vault CRUD

### Task 4.1: 实现 Vault 命令

**Files:**
- Create: `src-tauri/src/commands/vault.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 commands/vault.rs**

`src-tauri/src/commands/vault.rs`:
```rust
use tauri::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::crypto;
use crate::state::AppState;

#[derive(Serialize)]
pub struct EntryMeta {
    pub id: String,
    #[serde(rename = "entryType")]
    pub entry_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
    pub favorited: bool,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct EntrySecrets {
    pub fields: Vec<DecryptedField>,
}

#[derive(Serialize)]
pub struct DecryptedField {
    #[serde(rename = "fieldKey")]
    pub field_key: String,
    #[serde(rename = "fieldType")]
    pub field_type: String,
    pub value: String,
    #[serde(rename = "isSensitive")]
    pub is_sensitive: bool,
}

#[derive(Deserialize)]
pub struct CreateEntryInput {
    #[serde(rename = "entryType")]
    pub entry_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    pub fields: Vec<FieldInput>,
}

#[derive(Deserialize)]
pub struct FieldInput {
    #[serde(rename = "fieldKey")]
    pub field_key: String,
    #[serde(rename = "fieldType")]
    pub field_type: String,
    pub value: String,
    #[serde(rename = "isSensitive")]
    pub is_sensitive: bool,
}

#[derive(Deserialize)]
pub struct UpdateEntryInput {
    #[serde(rename = "entryType")]
    pub entry_type: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    pub fields: Option<Vec<FieldInput>>,
}

/// 验证会话并获取数据库和加密密钥
async fn require_session<'a>(
    state: &'a State<'_, AppState>,
    session_token: &str,
) -> Result<(sqlx::SqlitePool, zeroize::Zeroizing<[u8; 32]>), String> {
    if !state.sessions.validate(session_token).await {
        return Err("会话已过期".to_string());
    }
    let db = state.get_db().await?;
    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?.clone();
    Ok((db, key))
}

/// 获取所有条目元数据
#[tauri::command]
pub async fn list_entries(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntryMeta>, String> {
    let (db, _key) = require_session(&state, &session_token).await?;

    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, i32, Option<String>, i64)>(
        "SELECT id, entry_type, title, subtitle, tags, favorited, group_id, updated_at
         FROM entries ORDER BY favorited DESC, updated_at DESC"
    )
    .fetch_all(&db)
    .await
    .map_err(|e| e.to_string())?;

    let entries = rows
        .into_iter()
        .map(|(id, entry_type, title, subtitle, tags_json, favorited, group_id, updated_at)| {
            let tags: Vec<String> = tags_json
                .and_then(|j| serde_json::from_str(&j).ok())
                .unwrap_or_default();
            EntryMeta {
                id,
                entry_type,
                title,
                subtitle,
                tags,
                favorited: favorited != 0,
                group_id,
                updated_at,
            }
        })
        .collect();

    Ok(entries)
}

/// 按需获取单个条目的解密字段
#[tauri::command]
pub async fn get_entry_secrets(
    session_token: String,
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<EntrySecrets, String> {
    let (db, key) = require_session(&state, &session_token).await?;

    let rows = sqlx::query_as::<_, (String, String, String, i32)>(
        "SELECT field_key, field_type, enc_value, is_sensitive
         FROM fields WHERE entry_id = ? ORDER BY sort_order"
    )
    .bind(&entry_id)
    .fetch_all(&db)
    .await
    .map_err(|e| e.to_string())?;

    let mut fields = Vec::new();
    for (field_key, field_type, enc_value, is_sensitive) in rows {
        let decrypted = crypto::decrypt_field(&key, &enc_value)
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

/// 创建条目
#[tauri::command]
pub async fn create_entry(
    session_token: String,
    input: CreateEntryInput,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (db, key) = require_session(&state, &session_token).await?;

    let entry_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
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
    .execute(&db)
    .await
    .map_err(|e| e.to_string())?;

    // 插入加密字段
    for (i, field) in input.fields.iter().enumerate() {
        let enc_value = crypto::encrypt_field(&key, field.value.as_bytes())
            .map_err(|e| e.to_string())?;
        let field_id = Uuid::new_v4().to_string();

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
        .execute(&db)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(entry_id)
}

/// 更新条目
#[tauri::command]
pub async fn update_entry(
    session_token: String,
    entry_id: String,
    input: UpdateEntryInput,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (db, key) = require_session(&state, &session_token).await?;
    let now = Utc::now().timestamp();

    // 更新元数据
    if input.entry_type.is_some() || input.title.is_some() || input.subtitle.is_some() || input.tags.is_some() || input.group_id.is_some() {
        let tags_json = input.tags.map(|t| serde_json::to_string(&t).unwrap_or_default());

        sqlx::query(
            "UPDATE entries SET
                entry_type = COALESCE(?, entry_type),
                title = COALESCE(?, title),
                subtitle = COALESCE(?, subtitle),
                tags = COALESCE(?, tags),
                group_id = COALESCE(?, group_id),
                updated_at = ?
             WHERE id = ?"
        )
        .bind(&input.entry_type)
        .bind(&input.title)
        .bind(&input.subtitle)
        .bind(&tags_json)
        .bind(&input.group_id)
        .bind(now)
        .bind(&entry_id)
        .execute(&db)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 更新字段（如果提供了新字段）
    if let Some(fields) = input.fields {
        // 删除旧字段
        sqlx::query("DELETE FROM fields WHERE entry_id = ?")
            .bind(&entry_id)
            .execute(&db)
            .await
            .map_err(|e| e.to_string())?;

        // 插入新字段
        for (i, field) in fields.iter().enumerate() {
            let enc_value = crypto::encrypt_field(&key, field.value.as_bytes())
                .map_err(|e| e.to_string())?;
            let field_id = Uuid::new_v4().to_string();

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
            .execute(&db)
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// 删除条目
#[tauri::command]
pub async fn delete_entry(
    session_token: String,
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (db, _key) = require_session(&state, &session_token).await?;

    sqlx::query("DELETE FROM entries WHERE id = ?")
        .bind(&entry_id)
        .execute(&db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 搜索条目
#[tauri::command]
pub async fn search_entries(
    session_token: String,
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntryMeta>, String> {
    let (db, _key) = require_session(&state, &session_token).await?;

    let search_pattern = format!("%{}%", query);

    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, i32, Option<String>, i64)>(
        "SELECT id, entry_type, title, subtitle, tags, favorited, group_id, updated_at
         FROM entries
         WHERE title LIKE ? OR subtitle LIKE ?
         ORDER BY favorited DESC, updated_at DESC"
    )
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_all(&db)
    .await
    .map_err(|e| e.to_string())?;

    let entries = rows
        .into_iter()
        .map(|(id, entry_type, title, subtitle, tags_json, favorited, group_id, updated_at)| {
            let tags: Vec<String> = tags_json
                .and_then(|j| serde_json::from_str(&j).ok())
                .unwrap_or_default();
            EntryMeta {
                id,
                entry_type,
                title,
                subtitle,
                tags,
                favorited: favorited != 0,
                group_id,
                updated_at,
            }
        })
        .collect();

    Ok(entries)
}
```

- [ ] **Step 2: 更新 commands/mod.rs**

`src-tauri/src/commands/mod.rs`:
```rust
pub mod auth;
pub mod vault;
```

- [ ] **Step 3: 更新 lib.rs 注册所有命令**

`src-tauri/src/lib.rs`:
```rust
mod commands;
mod crypto;
mod db;
mod error;
mod state;

use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            app.manage(AppState::new(app_data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::is_initialized,
            commands::auth::setup,
            commands::auth::unlock,
            commands::auth::lock,
            commands::vault::list_entries,
            commands::vault::get_entry_secrets,
            commands::vault::create_entry,
            commands::vault::update_entry,
            commands::vault::delete_entry,
            commands::vault::search_entries,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: 验证编译**

```powershell
cd src-tauri && cargo build
```

Expected: 编译成功。

- [ ] **Step 5: 提交**

```powershell
git add src-tauri/
git commit -m "feat: implement vault CRUD commands (list, create, update, delete, search)"
```

---

### Task 4.2: 完善前端 VaultView 对接

**Files:**
- Modify: `src/views/VaultView.vue`
- Modify: `src/stores/vault.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: 更新 App.vue 初始化流程**

`src/App.vue` 完整替换为：
```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

onMounted(async () => {
  await auth.checkInitialized()
  if (!auth.isInitialized) {
    router.push('/setup')
  } else {
    router.push('/login')
  }
})
</script>

<template>
  <router-view />
</template>
```

- [ ] **Step 2: 更新 VaultView 添加新建条目功能**

`src/views/VaultView.vue` 完整替换为：
```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useVaultStore } from '@/stores/vault'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'
import { useShortcuts } from '@/composables/useShortcuts'
import { vault as vaultBridge } from '@/bridge/tauri'
import type { EntryMeta, CreateEntryInput, EntrySecrets } from '@/types/vault'

const router = useRouter()
const vault = useVaultStore()
const ui = useUiStore()
const auth = useAuthStore()

// 新建条目表单
const showCreateForm = ref(false)
const newTitle = ref('')
const newUsername = ref('')
const newPassword = ref('')
const newUrl = ref('')

// 详情面板
const selectedEntry = ref<EntryMeta | null>(null)
const entrySecrets = ref<EntrySecrets | null>(null)
const isLoadingSecrets = ref(false)

onMounted(() => {
  vault.loadEntries()
})

useShortcuts([
  { key: 'k', ctrl: true, handler: () => ui.toggleCommandPalette() },
  { key: 'l', ctrl: true, handler: handleLock },
  { key: 'n', ctrl: true, handler: () => { showCreateForm.value = true } },
])

async function handleLock() {
  await auth.lock()
  router.push('/login')
}

async function handleSelectEntry(entry: EntryMeta) {
  vault.selectEntry(entry.id)
  selectedEntry.value = entry
  ui.openDetailPanel()
  isLoadingSecrets.value = true
  entrySecrets.value = null
  try {
    entrySecrets.value = await vaultBridge.getEntrySecrets(entry.id)
  } catch (e) {
    console.error('Failed to load secrets:', e)
  } finally {
    isLoadingSecrets.value = false
  }
}

async function handleCreateEntry() {
  if (!newTitle.value) return
  const input: CreateEntryInput = {
    entryType: 'login',
    title: newTitle.value,
    subtitle: newUrl.value || undefined,
    tags: [],
    fields: [
      { fieldKey: 'username', fieldType: 'text', value: newUsername.value, isSensitive: false },
      { fieldKey: 'password', fieldType: 'password', value: newPassword.value, isSensitive: true },
      ...(newUrl.value ? [{ fieldKey: 'url', fieldType: 'url' as const, value: newUrl.value, isSensitive: false }] : []),
    ],
  }
  try {
    await vaultBridge.createEntry(input)
    showCreateForm.value = false
    newTitle.value = ''
    newUsername.value = ''
    newPassword.value = ''
    newUrl.value = ''
    await vault.loadEntries()
  } catch (e) {
    console.error('Failed to create entry:', e)
  }
}

async function handleDeleteEntry(entryId: string) {
  try {
    await vault.deleteEntry(entryId)
    selectedEntry.value = null
    entrySecrets.value = null
    ui.closeDetailPanel()
  } catch (e) {
    console.error('Failed to delete entry:', e)
  }
}

function closeDetail() {
  selectedEntry.value = null
  entrySecrets.value = null
  ui.closeDetailPanel()
}

function handleSearch() {
  vault.search(vault.searchQuery)
}
</script>

<template>
  <div class="vault-layout">
    <!-- 标题栏 -->
    <header class="titlebar" data-tauri-drag-region>
      <div class="titlebar-drag" data-tauri-drag-region />
      <div class="titlebar-controls">
        <button class="titlebar-btn" @click="/* minimize */">─</button>
        <button class="titlebar-btn" @click="/* maximize */">□</button>
        <button class="titlebar-btn close" @click="/* close */">✕</button>
      </div>
    </header>

    <div class="vault-body">
      <!-- 侧边栏 -->
      <aside class="sidebar">
        <div class="sidebar-header">
          <h2 class="sidebar-title">KeyVault</h2>
        </div>
        <nav class="sidebar-nav">
          <button class="nav-item active">
            <span class="nav-icon">🔐</span>
            <span class="nav-label">全部</span>
          </button>
          <button class="nav-item">
            <span class="nav-icon">⭐</span>
            <span class="nav-label">收藏</span>
          </button>
        </nav>
        <div class="sidebar-footer">
          <button class="nav-item" @click="router.push('/settings')">
            <span class="nav-icon">⚙️</span>
            <span class="nav-label">设置</span>
          </button>
          <button class="nav-item" @click="handleLock">
            <span class="nav-icon">🔒</span>
            <span class="nav-label">锁定</span>
          </button>
        </div>
      </aside>

      <!-- 条目列表 -->
      <main class="entry-list">
        <div class="list-header">
          <input
            v-model="vault.searchQuery"
            type="text"
            placeholder="搜索条目..."
            class="search-input"
            @input="handleSearch"
          />
          <button class="add-btn" title="新建条目 (Ctrl+N)" @click="showCreateForm = true">+</button>
        </div>
        <div class="list-body">
          <div
            v-for="entry in vault.entries"
            :key="entry.id"
            class="entry-item"
            :class="{ selected: vault.selectedId === entry.id }"
            @click="handleSelectEntry(entry)"
          >
            <div class="entry-icon">{{ entry.entryType === 'login' ? '🌐' : '🔑' }}</div>
            <div class="entry-info">
              <div class="entry-title">{{ entry.title }}</div>
              <div class="entry-subtitle">{{ entry.subtitle }}</div>
            </div>
          </div>
          <div v-if="!vault.isLoading && vault.entries.length === 0" class="empty-state">
            <p>暂无条目</p>
            <p class="hint">按 Ctrl+N 新建</p>
          </div>
        </div>
      </main>

      <!-- 详情面板 -->
      <aside v-if="ui.detailPanelOpen && selectedEntry" class="detail-panel">
        <div class="detail-header">
          <h3>{{ selectedEntry.title }}</h3>
          <button class="close-btn" @click="closeDetail">✕</button>
        </div>
        <div class="detail-body">
          <div v-if="isLoadingSecrets" class="loading">加载中...</div>
          <div v-else-if="entrySecrets" class="secrets-list">
            <div v-for="field in entrySecrets.fields" :key="field.fieldKey" class="secret-field">
              <label class="field-label">{{ field.fieldKey }}</label>
              <div class="field-value" :class="{ mono: field.isSensitive }">
                {{ field.isSensitive ? '••••••••' : field.value }}
              </div>
            </div>
            <div class="detail-actions">
              <button class="btn-danger" @click="handleDeleteEntry(selectedEntry!.id)">删除</button>
            </div>
          </div>
        </div>
      </aside>
    </div>

    <!-- 新建条目弹窗 -->
    <div v-if="showCreateForm" class="modal-overlay" @click.self="showCreateForm = false">
      <div class="modal">
        <h3>新建密码</h3>
        <div class="form-group">
          <label>标题</label>
          <input v-model="newTitle" placeholder="例如: GitHub" autofocus />
        </div>
        <div class="form-group">
          <label>用户名</label>
          <input v-model="newUsername" placeholder="user@example.com" />
        </div>
        <div class="form-group">
          <label>密码</label>
          <input v-model="newPassword" type="password" placeholder="密码" />
        </div>
        <div class="form-group">
          <label>网址</label>
          <input v-model="newUrl" placeholder="https://github.com" />
        </div>
        <div class="modal-actions">
          <button class="btn-secondary" @click="showCreateForm = false">取消</button>
          <button class="btn-primary" :disabled="!newTitle" @click="handleCreateEntry">创建</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.vault-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-base);
}

.titlebar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  height: var(--titlebar-height);
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
  -webkit-app-region: drag;
}
.titlebar-drag { flex: 1; }
.titlebar-controls { display: flex; -webkit-app-region: no-drag; }
.titlebar-btn {
  width: 46px; height: var(--titlebar-height);
  display: flex; align-items: center; justify-content: center;
  font-size: var(--text-xs); color: var(--text-secondary);
  transition: background var(--duration-fast);
}
.titlebar-btn:hover { background: var(--bg-elevated); }
.titlebar-btn.close:hover { background: var(--color-danger); color: #fff; }

.vault-body { display: flex; flex: 1; overflow: hidden; }

.sidebar {
  width: var(--sidebar-width); background: var(--bg-surface);
  border-right: 1px solid var(--border-subtle);
  display: flex; flex-direction: column; flex-shrink: 0;
}
.sidebar-header { padding: var(--space-4); border-bottom: 1px solid var(--border-subtle); }
.sidebar-title { font-size: var(--text-md); font-weight: 600; color: var(--text-primary); }
.sidebar-nav { flex: 1; padding: var(--space-2); display: flex; flex-direction: column; gap: var(--space-1); }
.nav-item {
  display: flex; align-items: center; gap: var(--space-3);
  padding: var(--space-2) var(--space-3); border-radius: var(--radius-md);
  font-size: var(--text-sm); color: var(--text-secondary); transition: all var(--duration-fast);
}
.nav-item:hover { background: var(--bg-elevated); color: var(--text-primary); }
.nav-item.active { background: var(--accent-blue-dim); color: var(--text-accent); }
.sidebar-footer { padding: var(--space-2); border-top: 1px solid var(--border-subtle); }

.entry-list { flex: 1; display: flex; flex-direction: column; min-width: 0; }
.list-header {
  display: flex; align-items: center; gap: var(--space-2);
  padding: var(--space-3); border-bottom: 1px solid var(--border-subtle);
}
.search-input {
  flex: 1; padding: var(--space-2) var(--space-3); font-size: var(--text-sm);
  background: var(--bg-input); border: 1px solid var(--border-default);
  border-radius: var(--radius-md); color: var(--text-primary);
}
.search-input:focus { border-color: var(--accent-blue); }
.add-btn {
  width: 32px; height: 32px; display: flex; align-items: center; justify-content: center;
  font-size: var(--text-lg); color: var(--text-secondary); border-radius: var(--radius-md);
  transition: all var(--duration-fast);
}
.add-btn:hover { background: var(--accent-blue-dim); color: var(--accent-blue); }
.list-body { flex: 1; overflow-y: auto; padding: var(--space-2); }
.entry-item {
  display: flex; align-items: center; gap: var(--space-3);
  height: var(--list-item-height); padding: 0 var(--space-3);
  border-radius: var(--radius-md); cursor: pointer; transition: background var(--duration-fast);
}
.entry-item:hover { background: var(--bg-elevated); }
.entry-item.selected { background: var(--bg-elevated); border-left: 2px solid var(--accent-blue); }
.entry-icon {
  width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
  font-size: 14px; background: var(--type-login); border-radius: var(--radius-sm); flex-shrink: 0;
}
.entry-info { flex: 1; min-width: 0; }
.entry-title {
  font-size: var(--text-base); font-weight: 500; color: var(--text-primary);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.entry-subtitle {
  font-size: var(--text-xs); color: var(--text-secondary);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.empty-state {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  height: 200px; color: var(--text-tertiary); font-size: var(--text-sm);
}
.hint { color: var(--text-tertiary); font-size: var(--text-xs); margin-top: var(--space-2); }

.detail-panel {
  width: var(--detail-panel-width); background: var(--bg-surface);
  border-left: 1px solid var(--border-subtle);
  display: flex; flex-direction: column; flex-shrink: 0;
  animation: slideInRight var(--duration-slow) var(--ease-out-quart);
}
.detail-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: var(--space-4); border-bottom: 1px solid var(--border-subtle);
}
.detail-header h3 { font-size: var(--text-md); font-weight: 600; }
.close-btn { color: var(--text-secondary); font-size: var(--text-sm); }
.close-btn:hover { color: var(--text-primary); }
.detail-body { flex: 1; padding: var(--space-4); overflow-y: auto; }
.loading { color: var(--text-tertiary); text-align: center; padding: var(--space-8); }
.secrets-list { display: flex; flex-direction: column; gap: var(--space-4); }
.secret-field { display: flex; flex-direction: column; gap: var(--space-1); }
.field-label { font-size: var(--text-xs); color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }
.field-value {
  padding: var(--space-2) var(--space-3); background: var(--bg-input);
  border-radius: var(--radius-md); font-size: var(--text-sm); color: var(--text-primary);
  word-break: break-all;
}
.field-value.mono { font-family: var(--font-mono); letter-spacing: 0.02em; }
.detail-actions { margin-top: var(--space-6); padding-top: var(--space-4); border-top: 1px solid var(--border-subtle); }
.btn-danger {
  padding: var(--space-2) var(--space-4); background: var(--color-danger);
  color: #fff; border-radius: var(--radius-md); font-size: var(--text-sm);
}
.btn-danger:hover { opacity: 0.9; }

.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.5);
  display: flex; align-items: center; justify-content: center; z-index: var(--z-modal);
}
.modal {
  background: var(--bg-surface); border: 1px solid var(--border-default);
  border-radius: var(--radius-lg); padding: var(--space-6); width: 400px;
  display: flex; flex-direction: column; gap: var(--space-4);
}
.modal h3 { font-size: var(--text-lg); font-weight: 600; }
.form-group { display: flex; flex-direction: column; gap: var(--space-1); }
.form-group label { font-size: var(--text-sm); color: var(--text-secondary); }
.form-group input {
  padding: var(--space-2) var(--space-3); background: var(--bg-input);
  border: 1px solid var(--border-default); border-radius: var(--radius-md);
  color: var(--text-primary); font-size: var(--text-sm);
}
.form-group input:focus { border-color: var(--accent-blue); outline: none; box-shadow: 0 0 0 2px var(--accent-blue-dim); }
.modal-actions { display: flex; justify-content: flex-end; gap: var(--space-3); margin-top: var(--space-2); }
.btn-secondary {
  padding: var(--space-2) var(--space-4); background: var(--bg-elevated);
  color: var(--text-secondary); border-radius: var(--radius-md); font-size: var(--text-sm);
}
.btn-secondary:hover { color: var(--text-primary); }
.btn-primary {
  padding: var(--space-2) var(--space-4); background: var(--accent-blue);
  color: #fff; border-radius: var(--radius-md); font-size: var(--text-sm);
}
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-primary:not(:disabled):hover { opacity: 0.9; }

@keyframes slideInRight {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}
</style>
```

- [ ] **Step 3: 运行完整流程验证**

```powershell
cargo tauri dev
```

Expected:
1. Setup → 输入密码 → VaultView（空状态）
2. 点击 "+" 或 Ctrl+N → 弹出新建表单
3. 输入标题、用户名、密码 → 点击创建
4. 条目出现在列表中
5. 点击条目 → 右侧详情面板显示解密后的字段
6. 搜索框输入关键词 → 列表过滤
7. 删除条目 → 条目消失

- [ ] **Step 4: 提交**

```powershell
git add .
git commit -m "feat: complete vault CRUD with frontend integration"
```

---

## 完成检查清单

- [ ] `cargo tauri dev` 窗口正常打开
- [ ] Setup 流程：输入密码 → 确认 → 创建成功
- [ ] Unlock 流程：输入正确密码 → 进入 VaultView
- [ ] 错误密码：抖动动画 + 错误提示
- [ ] 新建条目：标题 + 用户名 + 密码 → 加密存储
- [ ] 列表显示：标题 + 副标题
- [ ] 详情面板：点击条目 → 解密显示字段
- [ ] 搜索：输入关键词 → 过滤结果
- [ ] 删除：点击删除 → 条目消失
- [ ] 锁定：点击锁定 → 回到 UnlockView
- [ ] `cargo test` 所有 Rust 测试通过
