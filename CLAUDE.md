# KeyVault v3 - 本地加密密码管理器

## 项目概述
本地离线优先的桌面密码和 API Key 管理器。
- **技术栈**: Tauri 2.0 (Rust 后端) + Vue 3 + TypeScript + Vite 6
- **加密**: AES-256-GCM + Argon2id (RustCrypto crates)
- **数据库**: SQLCipher (sqlx)
- **设计语言**: 暗黑高密度终端美学 (参考 Warp Terminal / Obsidian)

## 目录结构
```
keyvault-v3/
├── src-tauri/          # Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── error.rs
│   │   ├── state.rs
│   │   ├── crypto/     # kdf.rs, cipher.rs, session.rs
│   │   ├── db/         # migrations/, schema.rs, queries.rs
│   │   ├── commands/   # auth.rs, vault.rs, generator.rs, breach.rs, sync.rs
│   │   └── tray.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                # Vue 3 前端
│   ├── bridge/         # Tauri IPC 桥接层
│   ├── components/     # UI 组件
│   ├── composables/    # 组合式函数
│   ├── constants/      # 常量定义
│   ├── design/         # 设计令牌
│   ├── router/         # 路由
│   ├── stores/         # Pinia 状态管理
│   ├── styles/         # 全局样式
│   ├── types/          # TypeScript 类型
│   └── views/          # 页面视图
├── extension/          # 浏览器扩展 (Manifest V3)
├── docs/               # 文档
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## 开发命令
- `npm run dev` - 启动 Vite 开发服务器
- `npm run build` - 构建前端
- `npm run tauri dev` - 启动 Tauri 开发模式 (Rust + Vue)
- `npm run tauri build` - 构建生产包
- `npm run type-check` - TypeScript 类型检查

## 安全红线
1. 主密码零存储（只存 Argon2id 哈希）
2. 密钥不落盘（内存中 Zeroizing 包装）
3. 锁定时清零加密密钥 + 销毁所有会话
4. 字段级加密（每个敏感字段独立 nonce）
5. 前端零缓存解密数据
6. IPC 全部验证 session token

## 执行步骤
详见 `docs/KEYVAULT_V3_FULL_MIGRATION.md` 的第七节。
