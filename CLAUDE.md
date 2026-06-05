# KeyVault v3 - 本地加密密码管理器

## 项目概述
本地离线优先的桌面密码和 API Key 管理器。
- **版本 / 阶段：** 3.0.0 · 桌面端 RC 候选
- **技术栈**: Tauri 2.0 (Rust 后端) + Vue 3 + TypeScript + Vite 6
- **加密**: AES-256-GCM + Argon2id + SQLCipher (HKDF 子密钥)
- **设计语言**: 暗黑高密度终端美学 (参考 Warp Terminal / Obsidian)

## 文档入口
- **项目现状：** `docs/PROJECT_STATUS.md`
- **文档索引：** `docs/README.md`
- **审查报告：** `docs/CODE_REVIEW_REPORT.md`
- **UI 验收：** `docs/软件界面原型/SIGNOFF.md`
- **迁移规划（历史参考）：** `docs/KEYVAULT_V3_FULL_MIGRATION.md`
- **开发过程归档：** `docs/archive/`

## 目录结构
```
keyvault-v3/
├── src-tauri/          # Rust 后端
│   ├── src/
│   │   ├── crypto/     # kdf.rs, cipher.rs, session.rs
│   │   ├── db/         # migrations/, schema.rs, queries.rs
│   │   ├── commands/   # auth, vault, sync, export, breach, ...
│   │   ├── sync/       # engine.rs, webdav.rs
│   │   └── native_messaging/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                # Vue 3 前端
│   ├── bridge/         # Tauri IPC 桥接层
│   ├── components/     # UI 组件
│   ├── stores/         # Pinia 状态管理
│   └── views/          # 页面视图
├── extension/          # 浏览器扩展（暂停，Sprint 4）
├── docs/               # 文档（见 docs/README.md）
├── package.json
└── vite.config.ts
```

## 开发命令
- `npm run dev` - 启动 Vite 开发服务器
- `npm run build` - 构建前端
- `npm run tauri dev` - 启动 Tauri 开发模式 (Rust + Vue)
- `npm run tauri build` - 构建生产包
- `npm run type-check` - TypeScript 类型检查
- `cd src-tauri && cargo test` - Rust 单元测试（41 项）
- `cd src-tauri && cargo audit` - 依赖安全审计

## 数据路径
- **标识符：** `com.keyvault.app`
- **数据目录：** Tauri `app_data_dir()`（非 `%APPDATA%/keyvault/`）
- **数据库：** `keyvault.db`（SQLCipher，仅解锁后连接）
- **Salt：** `kdf_salt.hex`

## 安全红线
1. 主密码零存储（只存 Argon2id 哈希）
2. 密钥不落盘（内存中 Zeroizing 包装）
3. 锁定时先销毁 session，再清零加密密钥并关闭 DB 连接池
4. 字段级加密（每个敏感字段独立 nonce）
5. 前端零缓存解密数据（锁定清空 vault store）
6. IPC 全部验证 session token（公开 UI 偏好键白名单除外）
7. SQLCipher 加密数据库文件

## 已知搁置（Sprint 4）
- 浏览器扩展 C-5/C-6/C-7：Native Host 与 GUI 路径/解锁态未打通

## 发布前待办
见 `docs/PROJECT_STATUS.md` 与 `docs/软件界面原型/SIGNOFF.md`
