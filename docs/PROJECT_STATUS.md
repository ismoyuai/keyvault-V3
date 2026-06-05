# KeyVault v3 项目现状

> **版本：** 3.0.0  
> **阶段：** 桌面端 **RC 候选**（Release Candidate）  
> **最后更新：** 2026-06-05

---

## 一句话总结

KeyVault v3 是一款 **Tauri 2 + Vue 3 + Rust** 的本地加密密码管理器。桌面端核心功能与安全加固已完成，**41 项 Rust 单测全通过**；浏览器扩展架构未打通，暂不作为 v3.0 交付范围。

---

## 功能完成度

### 桌面端（可交付）

| 模块 | 状态 | 说明 |
|------|------|------|
| 首次设置 / 解锁 / 锁定 | ✅ | Argon2id + session，5 次失败锁定 5 分钟 |
| 条目 CRUD | ✅ | 10 种类型，字段级 AES-GCM，编辑加载防丢数据 |
| 回收站 | ✅ | 软删除 30 天保留 |
| 命令面板 | ✅ | Ctrl+K 搜索与快捷操作 |
| 密码生成器 | ✅ | 随机 + Diceware |
| HIBP 泄露检测 | ✅ | k-匿名前缀查询 |
| 导入 / 导出 JSON | ✅ | Rust 端文件 IO，全量无条数上限 |
| WebDAV 同步 | ✅ | v3 加密远程文件，拒绝明文 v2 |
| 紧急擦除 | ✅ | 主密码确认后清空库 |
| 系统托盘 | ✅ | 最小化到托盘 |
| 自动锁定 / 剪贴板清空 | ✅ | 可配置超时 |

### 浏览器扩展（暂停，Sprint 4）

| 模块 | 状态 | 阻塞项 |
|------|------|--------|
| 表单检测 / 填充 UI | ⚠️ 代码存在 | C-5 DB 路径不一致 |
| Native Messaging | ⚠️ 代码存在 | C-6 与 GUI 解锁态不共享 |
| 保存凭据 / 打开应用 | ❌ | C-7 未实现 |

---

## 安全状态

| 红线 | 状态 |
|------|------|
| 主密码零存储 | ✅ |
| 密钥内存 Zeroizing，锁定清零 | ✅ |
| SQLCipher 数据库加密 | ✅（含单测；建议 CLI 再签字） |
| 字段级独立 nonce | ✅ |
| IPC session 校验 | ✅（公开设置键白名单） |
| 前端零缓存解密数据 | ✅（锁定清空 vault store） |
| Tauri 最小权限 | ✅（已移除 fs 插件） |

详见 [CODE_REVIEW_REPORT.md](./CODE_REVIEW_REPORT.md) 红线表与 Sprint 修复记录。

---

## 质量指标

| 检查项 | 结果 |
|--------|------|
| `cargo test` | **41** passed |
| `npm run type-check` | 通过 |
| `cargo audit` | 0 vulnerability |
| Argon2 耗时 | ≥ 900ms（单测） |
| 搜索性能 | 1000 条 < 50ms（单测） |

```bash
cd src-tauri && cargo test
npm run type-check
cd src-tauri && cargo audit
```

---

## 数据与路径

| 项 | 位置 |
|----|------|
| 应用标识 | `com.keyvault.app` |
| 数据目录 | Tauri `app_data_dir()`（Windows 通常为 `%APPDATA%\com.keyvault.app\`） |
| 数据库 | `keyvault.db`（SQLCipher） |
| KDF salt | `kdf_salt.hex`（sidecar） |
| UI 偏好 | `preferences.json`（未解锁可读：主题、自动锁定、剪贴板超时） |

> **注意：** `extension/` Native Host 仍使用旧路径 `%APPDATA%/keyvault/`，与 GUI 不一致（C-5，Sprint 4 修复）。

---

## 待完成（发布前）

### 人工 SIGNOFF

- [ ] UI 目视验收 — [软件界面原型/SIGNOFF.md](./软件界面原型/SIGNOFF.md)
- [ ] `sqlite3` CLI：锁定后无法读取 `keyvault.db`
- [ ] 冷启动 < 2s、空闲内存 < 60MB、安装包 < 20MB

### 下一里程碑

- [ ] **Sprint 4：** 浏览器扩展（C-5/C-6/C-7）
- [ ] 可选：主题切换、多语言、移动端

---

## 文档结构（整理后）

```
docs/
├── README.md                 # 本索引
├── PROJECT_STATUS.md         # 本文件
├── CODE_REVIEW_REPORT.md     # 审查与修复记录
├── KEYVAULT_V3_FULL_MIGRATION.md  # 迁移规划（历史参考）
├── 软件界面原型/              # UI 原型 + 验收
│   ├── SIGNOFF.md
│   ├── REQUIREMENTS-COVERAGE.md
│   └── screens/
└── archive/                  # 开发过程归档
    └── superpowers/          # AI 会话计划与审计稿
```

---

## 近期提交脉络

| 提交 | 内容 |
|------|------|
| Sprint 1–2 | SQLCipher、IPC 硬化、CRUD 事务、导入导出 |
| Sprint 3 | 锁定清理、单测补齐、cargo-audit |
| 硬化收尾 | I-2/3/7/15/21（session、同步、元数据门禁） |
| 跟进 | sync 脱敏、列表全量、报告同步 |
