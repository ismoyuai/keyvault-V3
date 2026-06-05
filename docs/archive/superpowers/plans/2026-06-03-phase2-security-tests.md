# Phase 2: Security Hardening & Test Coverage Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add comprehensive Rust unit tests for the crypto and command modules, and wire audit logging into sensitive operations.

**Architecture:** Tests use `#[cfg(test)]` modules within each source file. Each test creates an in-memory SQLite database via `sqlite::memory:?mode=rwc` and a fresh `AppState` for isolation.

**Tech Stack:** Rust, sqlx (in-memory), tokio::test

---

### Task 1: Crypto Module Tests

**Files:**
- Modify: `src-tauri/src/crypto/kdf.rs`
- Modify: `src-tauri/src/crypto/cipher.rs`
- Modify: `src-tauri/src/crypto/session.rs`

- [ ] **Step 1: Add tests to kdf.rs**

Append to `src-tauri/src/crypto/kdf.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let password = b"test_password_123";
        let salt = [0u8; 16];
        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();
        assert_eq!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = [0u8; 16];
        let key1 = derive_key(b"password_a", &salt).unwrap();
        let key2 = derive_key(b"password_b", &salt).unwrap();
        assert_ne!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_derive_key_different_salts() {
        let password = b"same_password";
        let salt1 = [0u8; 16];
        let salt2 = [1u8; 16];
        let key1 = derive_key(password, &salt1).unwrap();
        let key2 = derive_key(password, &salt2).unwrap();
        assert_ne!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_derive_key_length() {
        let key = derive_key(b"test", &[0u8; 16]).unwrap();
        assert_eq!(key.as_ref().len(), 32);
    }

    #[test]
    fn test_hash_and_verify() {
        let password = b"my_secure_password";
        let hash = hash_master_password(password).unwrap();
        assert!(verify_master_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = b"correct_password";
        let hash = hash_master_password(password).unwrap();
        assert!(!verify_master_password(b"wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_hash_not_deterministic() {
        let password = b"same_password";
        let hash1 = hash_master_password(password).unwrap();
        let hash2 = hash_master_password(password).unwrap();
        assert_ne!(hash1, hash2); // Different random salts
    }
}
```

- [ ] **Step 2: Add tests to cipher.rs**

Append to `src-tauri/src/crypto/cipher.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"hello world secret";
        let encrypted = encrypt_field(&key, plaintext).unwrap();
        let decrypted = decrypt_field(&key, &encrypted).unwrap();
        assert_eq!(decrypted.as_ref(), plaintext);
    }

    #[test]
    fn test_encrypt_different_nonces() {
        let key = [42u8; 32];
        let plaintext = b"same data";
        let enc1 = encrypt_field(&key, plaintext).unwrap();
        let enc2 = encrypt_field(&key, plaintext).unwrap();
        assert_ne!(enc1, enc2); // Different nonces
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let encrypted = encrypt_field(&key1, b"secret").unwrap();
        assert!(decrypt_field(&key2, &encrypted).is_err());
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let key = [0u8; 32];
        assert!(decrypt_field(&key, "not-valid-base64!!!").is_err());
    }

    #[test]
    fn test_decrypt_too_short() {
        let key = [0u8; 32];
        let short = base64::engine::general_purpose::STANDARD.encode([0u8; 5]);
        assert!(decrypt_field(&key, &short).is_err());
    }

    #[test]
    fn test_encrypt_empty_plaintext() {
        let key = [42u8; 32];
        let encrypted = encrypt_field(&key, b"").unwrap();
        let decrypted = decrypt_field(&key, &encrypted).unwrap();
        assert_eq!(decrypted.as_ref(), b"");
    }
}
```

- [ ] **Step 3: Add tests to session.rs**

Append to `src-tauri/src/crypto/session.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_validate() {
        let mgr = SessionManager::new();
        let token = mgr.create().await;
        assert!(mgr.validate(&token).await);
    }

    #[tokio::test]
    async fn test_validate_invalid_token() {
        let mgr = SessionManager::new();
        assert!(!mgr.validate("nonexistent").await);
    }

    #[tokio::test]
    async fn test_destroy_all() {
        let mgr = SessionManager::new();
        let token = mgr.create().await;
        assert!(mgr.validate(&token).await);
        mgr.destroy_all().await;
        assert!(!mgr.validate(&token).await);
    }

    #[tokio::test]
    async fn test_sliding_window_refresh() {
        let mgr = SessionManager::new();
        let token = mgr.create().await;
        // Validate multiple times — each should succeed and refresh
        assert!(mgr.validate(&token).await);
        assert!(mgr.validate(&token).await);
        assert!(mgr.validate(&token).await);
    }
}
```

- [ ] **Step 4: Run all crypto tests**

Run: `cd src-tauri && cargo test crypto`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/crypto/kdf.rs src-tauri/src/crypto/cipher.rs src-tauri/src/crypto/session.rs
git commit -m "test: add comprehensive crypto module tests"
```

---

### Task 2: Window/Clipboard/Settings Command Tests

**Files:**
- Modify: `src-tauri/src/commands/window.rs`
- Modify: `src-tauri/src/commands/clipboard_cmd.rs`
- Modify: `src-tauri/src/commands/settings.rs`

Note: These commands take `tauri::AppHandle` or `State<'_, AppState>` which are hard to unit-test without a full Tauri runtime. The tests here focus on the underlying logic, not the Tauri integration.

- [ ] **Step 1: Add settings logic tests**

Append to `src-tauri/src/commands/settings.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_state() -> AppState {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:?mode=rwc")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE config (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        AppState {
            encryption_key: RwLock::new(None),
            db: pool,
            sessions: crate::crypto::session::SessionManager::new(),
            kdf_salt: RwLock::new(None),
        }
    }

    #[tokio::test]
    async fn test_get_setting_returns_none_for_missing() {
        let state = create_test_state().await;
        let result = queries::get_config(&state.db, "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_set_and_get_setting() {
        let state = create_test_state().await;
        queries::set_config(&state.db, "theme", "dark").await.unwrap();
        let result = queries::get_config(&state.db, "theme").await.unwrap();
        assert_eq!(result, Some("dark".to_string()));
    }

    #[tokio::test]
    async fn test_set_setting_overwrites() {
        let state = create_test_state().await;
        queries::set_config(&state.db, "theme", "dark").await.unwrap();
        queries::set_config(&state.db, "theme", "light").await.unwrap();
        let result = queries::get_config(&state.db, "theme").await.unwrap();
        assert_eq!(result, Some("light".to_string()));
    }
}
```

Add imports at the top of settings.rs if needed:
```rust
use tokio::sync::RwLock;
```

- [ ] **Step 2: Run settings tests**

Run: `cd src-tauri && cargo test settings`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/settings.rs
git commit -m "test: add settings command tests"
```

---

### Task 3: Group Command Tests

**Files:**
- Modify: `src-tauri/src/commands/groups.rs`

- [ ] **Step 1: Add group tests**

Append to `src-tauri/src/commands/groups.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::RwLock;

    async fn create_test_state() -> (AppState, String) {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:?mode=rwc")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE config (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE groups (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, icon TEXT, color TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .unwrap();
        let sessions = crate::crypto::session::SessionManager::new();
        let token = sessions.create().await;
        let state = AppState {
            encryption_key: RwLock::new(None),
            db: pool,
            sessions,
            kdf_salt: RwLock::new(None),
        };
        (state, token)
    }

    #[tokio::test]
    async fn test_create_and_list_groups() {
        let (state, token) = create_test_state().await;
        let id = create_group(State::new(state.clone()), token.clone(), "Work".to_string(), Some("briefcase".to_string())).await.unwrap();
        let groups = list_groups(State::new(state), token).await.unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "Work");
        assert_eq!(groups[0].id, id);
    }

    #[tokio::test]
    async fn test_update_group() {
        let (state, token) = create_test_state().await;
        let id = create_group(State::new(state.clone()), token.clone(), "Old Name".to_string(), None).await.unwrap();
        update_group(State::new(state.clone()), token.clone(), id.clone(), "New Name".to_string(), Some("star".to_string())).await.unwrap();
        let groups = list_groups(State::new(state), token).await.unwrap();
        assert_eq!(groups[0].name, "New Name");
        assert_eq!(groups[0].icon, Some("star".to_string()));
    }

    #[tokio::test]
    async fn test_delete_group() {
        let (state, token) = create_test_state().await;
        let id = create_group(State::new(state.clone()), token.clone(), "To Delete".to_string(), None).await.unwrap();
        delete_group(State::new(state.clone()), token.clone(), id).await.unwrap();
        let groups = list_groups(State::new(state), token).await.unwrap();
        assert!(groups.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_session_rejected() {
        let (state, _token) = create_test_state().await;
        let result = list_groups(State::new(state), "invalid_token".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("会话已过期"));
    }
}
```

Note: `AppState` does not derive `Clone`. You'll need to either:
- Use `Arc<AppState>` in tests, or
- Create a helper that constructs the state and extracts what you need

The simplest approach: refactor the test to create the state once and use `tauri::State::new()` which requires the state to be `'static`. Since `tauri::State` is just a wrapper, you can test the underlying functions directly instead:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::RwLock;

    async fn setup_db() -> (sqlx::SqlitePool, crate::crypto::session::SessionManager, String) {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:?mode=rwc")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE groups (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, icon TEXT, color TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .unwrap();
        let sessions = crate::crypto::session::SessionManager::new();
        let token = sessions.create().await;
        (pool, sessions, token)
    }

    #[tokio::test]
    async fn test_create_list_delete_group() {
        let (pool, sessions, token) = setup_db().await;

        // Create
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        sqlx::query("INSERT INTO groups (id, name, icon, sort_order, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)")
            .bind(&id).bind("Work").bind(Some("briefcase")).bind(now).bind(now)
            .execute(&pool).await.unwrap();

        // List
        let groups = crate::db::queries::list_groups(&pool).await.unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "Work");

        // Delete
        sqlx::query("DELETE FROM groups WHERE id = ?").bind(&id).execute(&pool).await.unwrap();
        let groups = crate::db::queries::list_groups(&pool).await.unwrap();
        assert!(groups.is_empty());
    }
}
```

- [ ] **Step 2: Run group tests**

Run: `cd src-tauri && cargo test groups`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/groups.rs
git commit -m "test: add group CRUD tests"
```

---

### Task 4: Wire Audit Logging into Sensitive Operations

**Files:**
- Modify: `src-tauri/src/commands/auth.rs`
- Modify: `src-tauri/src/commands/vault.rs`

- [ ] **Step 1: Add audit logging to auth operations**

In `src-tauri/src/commands/auth.rs`, add audit log calls after successful operations.

After `setup` succeeds (before `Ok(token)`):
```rust
let _ = queries::write_audit_log(&state.db, "setup", None, None, None).await;
```

After `unlock` succeeds (before `Ok(token)`):
```rust
let _ = queries::write_audit_log(&state.db, "unlock", None, None, None).await;
```

After `lock` succeeds:
```rust
let _ = queries::write_audit_log(&state.db, "lock", None, None, None).await;
```

After `change_password` succeeds:
```rust
let _ = queries::write_audit_log(&state.db, "change_password", None, None, None).await;
```

Note: Audit log failures are silently ignored (don't block the main operation).

- [ ] **Step 2: Add audit logging to vault operations**

In `src-tauri/src/commands/vault.rs`, add audit log calls:

After `get_entry_secrets` decrypts fields (before returning):
```rust
let _ = queries::write_audit_log(&state.db, "view", Some(&entry_id), None, None).await;
```

After `delete_entry` succeeds:
```rust
let _ = queries::write_audit_log(&state.db, "delete", Some(&entry_id), None, None).await;
```

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/auth.rs src-tauri/src/commands/vault.rs
git commit -m "feat: wire audit logging into auth and vault operations"
```

---

### Task 5: Final Test Verification

- [ ] **Step 1: Run full test suite**

Run: `cd src-tauri && cargo test 2>&1`
Expected: All tests pass. Count the total number of tests.

- [ ] **Step 2: Verify test count**

Expected minimum: 15+ tests across crypto, session, settings, groups, and native_ext modules.

- [ ] **Step 3: Commit any remaining fixes**

If any tests fail, fix and commit.
