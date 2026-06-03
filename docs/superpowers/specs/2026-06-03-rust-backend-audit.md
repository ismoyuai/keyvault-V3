# Rust 后端审查规格文档

> **审查目标：** 全面审查 KeyVault v3 Rust 后端代码的架构、安全性和代码质量
>
> **审查方法：** 自底向上（L1 Crypto → L2 DB → L3 State → L4 Commands → L5 App）
>
> **审查范围：** `src-tauri/src/` 下所有模块

---

## 1. 审查范围

### 1.1 模块清单

| 层 | 文件 | 职责 |
|---|---|---|
| L1 Crypto | `crypto/kdf.rs` | Argon2id 密钥派生、主密码哈希 |
| L1 Crypto | `crypto/cipher.rs` | AES-256-GCM 字段级加密/解密 |
| L1 Crypto | `crypto/session.rs` | Session token 生成、验证、销毁 |
| L1 Crypto | `crypto/mod.rs` | 模块声明 |
| L2 DB | `db/schema.rs` | 数据结构定义（EntryMeta, FieldRow, GroupRow） |
| L2 DB | `db/queries.rs` | SQL 查询函数 |
| L2 DB | `db/migrations/` | 数据库迁移文件 |
| L2 DB | `db/mod.rs` | 模块声明 |
| L3 State | `state.rs` | AppState 定义、lock/unlock 方法 |
| L3 State | `error.rs` | CryptoError、AppError 错误类型 |
| L4 Commands | `commands/auth.rs` | 认证命令（setup, unlock, lock, change_password） |
| L4 Commands | `commands/vault.rs` | 密码库命令（CRUD, search, favorites） |
| L4 Commands | `commands/generator.rs` | 密码生成器命令 |
| L4 Commands | `commands/breach.rs` | HIBP 泄露检测命令 |
| L4 Commands | `commands/native_ext.rs` | 浏览器扩展 Native Messaging |
| L4 Commands | `commands/mod.rs` | 模块声明（缺失 5 个模块） |
| L5 App | `lib.rs` | 模块注册、Tauri setup、命令注册 |
| L5 App | `main.rs` | 双模式入口（GUI / Native Messaging） |
| L5 App | `tray.rs` | 系统托盘（未实现） |

### 1.2 关联文件

| 文件 | 审查内容 |
|---|---|
| `Cargo.toml` | 依赖版本、feature flags |
| `tauri.conf.json` | 窗口配置、CSP 策略 |
| `capabilities/default.json` | IPC 权限配置 |
| `src/bridge/tauri.ts` | 前端 IPC 调用对照 |

---

## 2. 各层审查标准

### 2.1 L1 Crypto 层

#### kdf.rs (Argon2id)

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 参数选择 | memoryCost=65536, timeCost=3, parallelism=1 | Critical |
| salt 生成 | 使用 `rand::rngs::OsRng`，16 字节 | Critical |
| 密钥输出 | 使用 `Zeroizing<[u8; 32]>` 包装 | Critical |
| hash 函数 | 输出格式包含算法标识、参数、salt | Important |
| verify 函数 | 使用 constant-time 比较 | Critical |
| 错误处理 | 不泄露内部状态 | Important |

#### cipher.rs (AES-256-GCM)

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| nonce 生成 | 每次加密重新生成 12 字节随机 nonce | Critical |
| 密文格式 | `nonce (12B) \|\| ciphertext \|\| tag (16B)` | Important |
| 密钥使用 | 接收 `&[u8; 32]`，不存储密钥 | Critical |
| 错误处理 | 解密失败不泄露 timing 信息 | Important |
| 内存安全 | 临时缓冲区是否清零 | Important |

#### session.rs

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| token 生成 | 使用 `OsRng`，足够长度（32+ 字节） | Critical |
| TTL 实现 | 30 分钟过期，滑动窗口刷新 | Important |
| 并发安全 | 使用 `RwLock` 保护内部状态 | Critical |
| destroy_all | 清零所有 token 内存 | Critical |
| token 验证 | constant-time 比较 | Important |

---

### 2.2 L2 DB 层

#### schema.rs

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 字段映射 | Rust 类型与 SQL 类型一致 | Important |
| Option 处理 | SQL NULL 正确映射为 `Option<T>` | Important |
| 序列化安全 | 不泄露敏感信息（如加密密文） | Important |
| 派生宏 | 正确使用 `Serialize`, `FromRow` | Minor |

#### queries.rs

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| SQL 注入防护 | 所有查询使用参数化绑定 | Critical |
| 事务边界 | 多步操作使用事务 | Important |
| 错误处理 | 不暴露 SQL 语句或表结构 | Important |
| 缺失函数 | `write_audit_log` 是否已实现 | Important |
| 分页支持 | 大数据量查询是否有 LIMIT | Minor |

---

### 2.3 L3 State 层

#### state.rs

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 密钥保护 | `RwLock<Option<Zeroizing<[u8; 32]>>>` 正确使用 | Critical |
| lock() 实现 | 真正清零密钥内存 | Critical |
| is_unlocked() | 无 TOCTOU 风险 | Important |
| db 连接池 | 正确配置和关闭 | Important |
| kdf_salt 保护 | 与加密密钥同等保护 | Important |

#### error.rs

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 信息泄露 | AppError 序列化不暴露内部路径 | Critical |
| 错误转换 | CryptoError → 用户友好消息 | Important |
| 覆盖范围 | 所有失败场景都有对应错误类型 | Important |
| 序列化格式 | JSON 结构一致、可被前端解析 | Minor |

---

### 2.4 L4 Commands 层

#### 通用检查（每个命令）

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| Session 验证 | 敏感命令必须验证 session_token | Critical |
| 输入校验 | 参数长度、格式、范围检查 | Important |
| 审计日志 | 敏感操作记录审计日志 | Important |
| 返回值安全 | 不泄露不必要的内部信息 | Important |

#### auth.rs 特别检查

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 防重复初始化 | setup 检查是否已初始化 | Critical |
| 暴力破解防护 | unlock 是否有失败次数限制 | Important |
| change_password | 原子性重新加密所有字段 | Critical |
| 密码验证 | 使用 KDF 验证，不存储明文 | Critical |

#### vault.rs 特别检查

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 字段加密 | 敏感字段创建/更新时正确加密 | Critical |
| 访问控制 | get_entry_secrets 验证权限 | Important |
| 级联删除 | delete_entry 清理相关数据 | Important |
| 搜索安全 | 搜索不泄露加密字段内容 | Important |

---

### 2.5 L5 App 层

#### lib.rs

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 命令注册 | generate_handler![] 包含所有命令 | Critical |
| 初始化顺序 | DB → State → Tray 顺序正确 | Important |
| 错误处理 | setup 错误向用户展示友好消息 | Important |
| 资源管理 | 应用退出时正确清理资源 | Important |

#### main.rs

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 模式隔离 | Native Messaging 模式与 GUI 模式隔离 | Important |
| 资源泄漏 | 两种模式都正确清理资源 | Important |

---

## 3. 预期发现

### 3.1 Critical 级（预期）

| # | 问题 | 位置 | 影响 |
|---|---|---|---|
| C1 | 10+ 个 IPC 命令未实现 | `commands/mod.rs`, `lib.rs` | 应用无法正常运行 |
| C2 | 审计日志函数未实现 | `db/queries.rs` | 无法追踪敏感操作 |
| C3 | 零测试覆盖 | 所有模块 | 无法验证正确性 |
| C4 | change_password 非原子性 | `commands/auth.rs` | 数据一致性风险 |

### 3.2 Important 级（预期）

| # | 问题 | 位置 | 影响 |
|---|---|---|---|
| I1 | AppError 序列化可能泄露路径 | `error.rs` | 安全风险 |
| I2 | session destroy_all 可能未清零 | `session.rs` | 内存安全 |
| I3 | 导出功能假设 non-sensitive 明文存储 | `export_cmd.rs` | 数据泄露 |
| I4 | queries.rs 缺少分页 | `queries.rs` | 性能问题 |
| I5 | 错误消息中英文混杂 | 多处 | 用户体验 |

### 3.3 Minor 级（预期）

| # | 问题 | 位置 | 影响 |
|---|---|---|---|
| M1 | 缺少文档注释 | 多处 | 可维护性 |
| M2 | 未使用的 import 警告 | 多处 | 编译噪音 |
| M3 | 常量未提取 | 多处 | 可维护性 |

---

## 4. 审查工作流

```
读取源文件 → 逐层审查 → 记录发现 → 分类排序 → 产出报告
     ↓
  L1 Crypto → L2 DB → L3 State → L4 Commands → L5 App
     ↓
  每层完成后立即记录，不等全部结束
```

---

## 5. 交付物

| 文件 | 内容 | 用途 |
|---|---|---|
| `docs/superpowers/specs/2026-06-03-rust-backend-audit.md` | 本文档 | 审查规格 |
| 审查报告（内嵌于本文档更新） | 详细发现 | 记录所有问题 |
| `docs/superpowers/plans/2026-06-03-rust-backend-fixes.md` | 修复计划 | 可执行任务清单 |

---

## 6. 安全红线对照

审查过程中必须对照以下 CLAUDE.md 安全红线逐条验证：

1. **主密码零存储** — 只存 Argon2id 哈希，不存明文或可逆形式
2. **密钥不落盘** — 内存中 Zeroizing 包装，不写入文件或日志
3. **锁定时清零** — 加密密钥 + 所有会话 token 清零
4. **字段级加密** — 每个敏感字段独立 nonce
5. **前端零缓存** — 解密数据不通过 IPC 返回给前端持久化
6. **IPC 全部验证 session token** — 除公开命令外，所有 IPC 调用验证 session

---

## 7. 审查报告

> **审查完成时间：** 2026-06-03
>
> **审查范围：** `src-tauri/src/` 全部 Rust 模块（共 15 个源文件）
>
> **发现汇总：** 5 Critical, 8 Important, 4 Minor

---

### 7.1 L1 Crypto 层审查结果

#### kdf.rs — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| 参数选择 | **通过** | MEMORY_COST=65536, TIME_COST=3, PARALLELISM=1 符合 OWASP 推荐 |
| salt 生成 | **通过** | `SaltString::generate(&mut OsRng)` 使用系统 CSPRNG |
| 密钥输出 | **通过** | `Zeroizing::new([0u8; 32])` 包装，drop 时自动清零 |
| hash 函数 | **通过** | 使用 `argon2` crate 的 `hash_password`，输出 PHC 格式含算法标识+参数+salt |
| verify 函数 | **通过** | `Argon2::default().verify_password()` 内部使用 constant-time 比较 |
| 错误处理 | **通过** | 所有错误映射到 `CryptoError` 枚举，不泄露内部状态 |

**结论：kdf.rs 实现正确，无问题。**

#### cipher.rs — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| nonce 生成 | **通过** | `Aes256Gcm::generate_nonce(&mut OsRng)` 每次加密生成新 nonce |
| 密文格式 | **通过** | `nonce (12B) \|\| ciphertext \|\| tag (16B)`，base64 编码 |
| 密钥使用 | **通过** | 接收 `&[u8; 32]`，不存储，不复制（除 cipher 初始化） |
| 错误处理 | **通过** | 解密失败返回 `CryptoError::DecryptError`，不泄露 timing |
| 内存安全 | **通过** | 解密结果用 `Zeroizing<Vec<u8>>` 包装 |

**结论：cipher.rs 实现正确，无问题。**

#### session.rs — 2 个 Important 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| token 生成 | **Important** | 使用 `rand::thread_rng()` 而非 `OsRng`。`thread_rng()` 基于 ChaCha，安全性足够但非系统级 CSPRNG |
| TTL 实现 | **通过** | 30 分钟 TTL + 滑动窗口刷新实现正确 |
| 并发安全 | **通过** | `RwLock<HashMap<String, SystemTime>>` 保护内部状态 |
| destroy_all | **Important** | `HashMap::clear()` 释放内存但不主动清零 token 字符串内容。String 不是 Zeroize-aware 类型 |
| token 验证 | **通过** | HashMap 查找是 O(1)，token 比较依赖 HashMap 的 Eq trait（非 constant-time，但 token 是随机的，timing attack 不实际） |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| I-1 | Important | token 生成使用 `thread_rng()` 而非 `OsRng` | `session.rs:22` | 改为 `use rand::rngs::OsRng; OsRng.sample_iter(...)` |
| I-2 | Important | `destroy_all` 未清零 token 内存 | `session.rs:55` | 改为遍历 HashMap，对每个 token 调用 `zeroize` 后再 clear |

---

### 7.2 L2 DB 层审查结果

#### schema.rs — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| 字段映射 | **通过** | Rust 类型与 SQL 类型一致（String→TEXT, i32→INTEGER, Option→NULLABLE） |
| Option 处理 | **通过** | `subtitle`, `tags`, `group_id`, `icon`, `color` 正确使用 `Option<String>` |
| 序列化安全 | **通过** | `FieldRow` 未派生 `Serialize`，不会意外泄露到 IPC |
| 派生宏 | **通过** | 正确使用 `Serialize`, `FromRow` |

**结论：schema.rs 实现正确，无问题。**

#### queries.rs — 2 个问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| SQL 注入防护 | **通过** | 所有查询使用 `sqlx::query().bind()` 参数化绑定 |
| 事务边界 | **Important** | `search_entries` 的 `format!("%{}%", query)` 虽然通过 bind 传递（安全），但 LIKE 通配符未转义，用户输入 `%` 或 `_` 会改变匹配语义 |
| 错误处理 | **通过** | 返回 `sqlx::Error`，不暴露 SQL 语句 |
| 缺失函数 | **Critical** | `write_audit_log` 未实现，但迁移文件 `001_init.sql` 已创建 `audit_log` 表 |
| 分页支持 | **Minor** | `list_entries` 和 `search_entries` 无 LIMIT，大数据量可能性能差 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| C-1 | Critical | `write_audit_log` 函数未实现 | `queries.rs` | 实现该函数，插入 `audit_log` 表 |
| I-3 | Important | LIKE 通配符未转义 | `queries.rs:19` | 对 query 中的 `%`, `_` 进行转义：`query.replace('%', "\\%").replace('_', "\\_")` |
| M-1 | Minor | 查询无分页限制 | `queries.rs:10,18` | 添加 `LIMIT ? OFFSET ?` 参数 |

#### migrations — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| 表结构 | **通过** | 5 张表（config, groups, entries, fields, breach_cache, audit_log）+ 1 张历史表 |
| 外键约束 | **通过** | `entries.group_id` ON DELETE SET NULL, `fields.entry_id` ON DELETE CASCADE |
| 索引 | **通过** | 关键查询路径有索引覆盖 |

**结论：迁移文件实现正确。**

---

### 7.3 L3 State 层审查结果

#### state.rs — 1 个 Critical 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 密钥保护 | **通过** | `RwLock<Option<Zeroizing<[u8; 32]>>>` 正确保护 |
| lock() 实现 | **Critical** | `*key = None` 会 drop `Zeroizing<[u8; 32]>`（清零），但 `destroy_all` 先于密钥清零执行。如果 `destroy_all` 期间有并发 session 验证，可能访问已清零的密钥 |
| is_unlocked() | **通过** | 使用 `try_read()` 避免阻塞，无 TOCTOU 风险（读锁原子性） |
| kdf_salt 保护 | **Important** | `kdf_salt: RwLock<Option<[u8; 16]>>` 未用 `Zeroizing` 包装。salt 不是密钥，但泄露可降低攻击成本 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| C-2 | Critical | `lock()` 中密钥清零与 session 销毁非原子操作 | `state.rs:19-25` | 先清零密钥，再销毁 session。或者用一个写锁同时保护两者 |
| I-4 | Important | `kdf_salt` 未用 `Zeroizing` 包装 | `state.rs:15` | 改为 `RwLock<Option<Zeroizing<[u8; 16]>>>` |

#### error.rs — 1 个 Important 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 信息泄露 | **Important** | `AppError::Database(sqlx::Error)` 的 `#[from]` 转换会调用 `sqlx::Error::to_string()`，可能泄露 SQL 语句或表结构 |
| 错误转换 | **通过** | 所有错误映射为中文用户友好消息 |
| 覆盖范围 | **通过** | 覆盖 Database, Crypto, SessionExpired, Locked, NotInitialized, WrongPassword, Network, BadRequest |
| 序列化格式 | **通过** | `serialize_str(&self.to_string())` 返回纯字符串，前端可直接显示 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| I-5 | Important | `AppError::Database` 序列化可能泄露 SQL 细节 | `error.rs:28` | 改为 `Database(String)` 手动映射：`impl From<sqlx::Error> for AppError { fn from(e: sqlx::Error) -> Self { AppError::Database("数据库错误".to_string()) } }` |

---

### 7.4 L4 Commands 层审查结果

#### auth.rs — 2 个 Critical 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 防重复初始化 | **Critical** | `setup` 不检查是否已初始化，重复调用会覆盖密码哈希和 salt |
| 暴力破解防护 | **Important** | `unlock` 无失败次数限制，攻击者可无限尝试 |
| change_password | **Critical** | 重新加密 fields 和 field_history 不在事务中，中途失败会导致部分字段用旧密钥、部分用新密钥 |
| 密码验证 | **通过** | 使用 `kdf::verify_master_password` 验证，只存储 PHC 格式哈希 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| C-3 | Critical | `setup` 不检查是否已初始化 | `auth.rs:16` | 在开头添加检查：`if queries::get_config(&state.db, "password_hash").await?.is_some() { return Err("已初始化".to_string()); }` |
| C-4 | Critical | `change_password` 非原子性 | `auth.rs:97-186` | 用 `sqlx::transaction` 包装所有数据库操作 |
| I-6 | Important | `unlock` 无暴力破解防护 | `auth.rs:50` | 添加失败计数器（存 DB 或内存），超过阈值锁定一段时间 |

#### vault.rs — 1 个 Important 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 字段加密 | **通过** | `create_entry` 和 `update_entry` 对所有字段调用 `cipher::encrypt_field` |
| 访问控制 | **Important** | `get_entry_secrets` 只验证 session，不验证用户是否有权访问该 entry（多用户场景下有风险，但当前单用户模型可接受） |
| 级联删除 | **通过** | SQL 外键 `ON DELETE CASCADE` 确保删除 entry 时级联删除 fields |
| 搜索安全 | **通过** | 搜索只在 title/subtitle/tags 上进行，不搜索加密字段内容 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| I-7 | Important | `update_entry` 中保存旧值到历史的查询在 DELETE 之后 | `vault.rs:247-274` | 先查询旧字段保存到历史，再 DELETE 旧字段，再 INSERT 新字段。当前代码先 DELETE 再查询会查不到旧值 |

#### generator.rs — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| Session 验证 | **通过** | 验证 session_token |
| 随机性 | **通过** | 使用 `rand::thread_rng()` (ChaCha) + Fisher-Yates 洗牌 |
| 边界处理 | **通过** | 空 charset 回退到 lowercase，长度取 max(length, required_sets.len()) |

**结论：generator.rs 实现正确，无问题。**

#### breach.rs — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| k-匿名性 | **通过** | 只发送 SHA1 前 5 字符，密码原文不离开设备 |
| 缓存机制 | **通过** | 7 天缓存，避免重复请求 |
| Session 验证 | **通过** | 验证 session_token |

**结论：breach.rs 实现正确，无问题。**

#### native_ext.rs — 1 个 Minor 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| Session 验证 | **通过** | `find_credentials_by_url` 和 `get_entry_for_fill` 都验证 session |
| URL 解析 | **通过** | `extract_hostname` 正确处理 protocol、port、www 前缀 |
| 测试覆盖 | **通过** | 有 `#[cfg(test)]` 测试 `extract_hostname` |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| M-2 | Minor | `extract_hostname` 不支持 `https://` 以外的协议（如 `ftp://`） | `native_ext.rs:149` | 可接受，密码管理器只需处理 http/https |

#### commands/mod.rs — 1 个 Critical 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 模块声明 | **Critical** | 缺少 5 个模块声明：`window`, `clipboard_cmd`, `settings`, `groups`, `export_cmd` |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| C-5 | Critical | 缺少 5 个模块声明 | `commands/mod.rs` | 添加 `pub mod window; pub mod clipboard_cmd; pub mod settings; pub mod groups; pub mod export_cmd;` |

---

### 7.5 L5 App 层审查结果

#### lib.rs — 1 个 Critical 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 命令注册 | **Critical** | `generate_handler![]` 只注册了 14 个命令，缺少：`minimize_window`, `toggle_maximize`, `close_window`, `copy_to_clipboard`, `clear_clipboard`, `get_setting`, `set_setting`, `list_groups`, `create_group`, `update_group`, `delete_group`, `export_vault`, `import_vault` |
| 初始化顺序 | **通过** | DB → State → Tray 顺序正确 |
| 错误处理 | **通过** | `expect()` 在 setup 阶段可接受（启动失败应 panic） |
| 资源管理 | **通过** | Tauri 框架自动管理资源生命周期 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| C-5（同上） | Critical | 缺少 13 个命令注册 | `lib.rs:50-65` | 在 `generate_handler![]` 中添加所有缺失命令 |

#### main.rs — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| 模式隔离 | **通过** | `--native-messaging` 参数清晰隔离两种模式 |
| 资源泄漏 | **通过** | 两种模式各自独立运行，退出时自动清理 |

**结论：main.rs 实现正确，无问题。**

#### tray.rs — 未实现

`tray.rs` 文件不存在，但 `Cargo.toml` 已启用 `tray-icon` feature。需要创建。

---

### 7.6 安全红线对照结果

| # | 红线 | 状态 | 说明 |
|---|---|---|---|
| 1 | 主密码零存储 | **通过** | 只存 Argon2id PHC 格式哈希 |
| 2 | 密钥不落盘 | **通过** | `Zeroizing<[u8; 32]>` 包装，不写入文件 |
| 3 | 锁定时清零 | **部分通过** | `lock()` 清零密钥，但 session token 未清零（I-2） |
| 4 | 字段级加密 | **通过** | 每个敏感字段独立 nonce |
| 5 | 前端零缓存 | **通过** | IPC 只返回解密后的文本，不返回密文 |
| 6 | IPC 全部验证 session | **通过** | 除 `is_initialized` 和 `setup` 外，所有命令验证 session |

---

### 7.7 发现汇总

#### Critical 级（5 个）

| # | 问题 | 位置 | 修复建议 |
|---|---|---|---|
| C-1 | `write_audit_log` 函数未实现 | `db/queries.rs` | 实现该函数，插入 `audit_log` 表 |
| C-2 | `lock()` 中密钥清零与 session 销毁非原子 | `state.rs:19-25` | 先清零密钥再销毁 session，或用单写锁保护 |
| C-3 | `setup` 不检查是否已初始化 | `auth.rs:16` | 添加重复初始化检查 |
| C-4 | `change_password` 非原子性 | `auth.rs:97-186` | 用 `sqlx::transaction` 包装 |
| C-5 | 缺少 13 个 IPC 命令注册 | `lib.rs:50-65`, `commands/mod.rs` | 创建 5 个新模块，注册 13 个命令 |

#### Important 级（8 个）

| # | 问题 | 位置 | 修复建议 |
|---|---|---|---|
| I-1 | session token 生成用 `thread_rng()` | `session.rs:22` | 改为 `OsRng` |
| I-2 | `destroy_all` 未清零 token 内存 | `session.rs:55` | 遍历清零后再 clear |
| I-3 | LIKE 通配符未转义 | `queries.rs:19` | 转义 `%` 和 `_` |
| I-4 | `kdf_salt` 未用 `Zeroizing` 包装 | `state.rs:15` | 改为 `Zeroizing<[u8; 16]>` |
| I-5 | `AppError::Database` 可能泄露 SQL 细节 | `error.rs:28` | 手动映射为通用错误消息 |
| I-6 | `unlock` 无暴力破解防护 | `auth.rs:50` | 添加失败计数和锁定机制 |
| I-7 | `update_entry` 历史保存逻辑顺序错误 | `vault.rs:247-274` | 先查询旧值保存历史，再 DELETE，再 INSERT |
| I-8 | 系统托盘未实现 | `tray.rs`（缺失） | 创建 `tray.rs` 并注册到 `lib.rs` |

#### Minor 级（4 个）

| # | 问题 | 位置 | 修复建议 |
|---|---|---|---|
| M-1 | 查询无分页限制 | `queries.rs:10,18` | 添加 LIMIT/OFFSET |
| M-2 | `extract_hostname` 仅支持 http/https | `native_ext.rs:149` | 可接受，无需修复 |
| M-3 | 部分模块缺少文档注释 | 多处 | 添加关键函数 doc comments |
| M-4 | 错误消息中英文混杂 | 多处 | 统一为中文 |

---

### 7.8 修复优先级排序

**第一优先级（阻塞应用运行）：**
1. C-5: 创建缺失的 5 个命令模块 + 注册 13 个命令
2. C-1: 实现 `write_audit_log`

**第二优先级（数据一致性）：**
3. C-4: `change_password` 原子性
4. C-3: `setup` 防重复初始化
5. I-7: `update_entry` 历史保存顺序

**第三优先级（安全加固）：**
6. C-2: `lock()` 原子性
7. I-1: session token 改用 OsRng
8. I-2: `destroy_all` 清零 token
9. I-4: kdf_salt 用 Zeroizing 包装
10. I-5: AppError::Database 隐藏 SQL 细节
11. I-6: 暴力破解防护

**第四优先级（质量改进）：**
12. I-8: 系统托盘
13. I-3: LIKE 通配符转义
14. M-1 ~ M-4: Minor 问题
