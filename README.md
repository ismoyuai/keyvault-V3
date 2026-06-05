<div align="center">

# 🔐 KeyVault v3

**本地离线优先的密码与 API Key 管理器**

数据自主可控 · 私有化部署 · 零依赖第三方云服务

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Version](https://img.shields.io/badge/version-3.0.0-orange)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)
![Vue](https://img.shields.io/badge/Vue-3-green)

</div>

---

## 📖 简介

KeyVault 是一款专为开发者和技术人员设计的本地密码管理器。与商业密码管理器不同，KeyVault 将数据完全掌控在您手中——所有密码和密钥都加密存储在本地，通过 WebDAV 与您的私有 NAS 同步，绝不经过任何第三方服务器。

**为什么选择 KeyVault？**

- 🔒 **数据自主可控** — 您的数据，您的服务器，您的规则
- 🚀 **轻量高效** — <20MB 安装包，<60MB 内存占用
- 🎨 **终端美学** — 暗黑高密度设计，专为开发者打造
- 🔑 **开发者友好** — 原生支持 API Key、SSH Key、多行密钥管理

---

## ✨ 功能特性

### 核心功能

- **密码管理** — 安全存储和管理所有密码、API Key、SSH Key
- **浏览器自动填充** — 浏览器扩展自动检测登录表单并填充凭据
- **密码生成器** — 内置安全密码生成器，支持自定义规则
- **泄露检测** — 集成 Have I Been Pwned，实时检测密码泄露风险
- **剪贴板管理** — 复制后自动倒计时清除，防止敏感信息残留

### 开发者场景

- **API Key 管理** — 专为开发者设计的密钥管理界面
- **SSH Key 存储** — 多行格式化显示，语法高亮
- **快速搜索** — Spotlight 风格命令面板，Ctrl+K 唤醒
- **虚拟滚动** — 大量条目时保持流畅性能

### 安全特性

- **字段级加密** — 每个敏感字段独立加密，独立 nonce
- **暴力破解防护** — 5 次失败锁定 5 分钟
- **自动锁定** — 空闲超时自动锁定
- **审计日志** — 记录所有敏感操作

---

## 🏗️ 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                      KeyVault v3                            │
├─────────────────────────────────────────────────────────────┤
│  浏览器扩展 (Manifest V3)                                    │
│  ├── content/detector.js — 表单检测                          │
│  ├── content/filler.js — 自动填充                            │
│  └── content/save-prompt.js — 保存提示                       │
├─────────────────────────────────────────────────────────────┤
│  Vue 3 前端 (TypeScript + Vite 6)                           │
│  ├── Pinia 状态管理                                          │
│  ├── @vueuse/core 响应式工具                                 │
│  └── 设计系统 (暗黑高密度终端美学)                            │
├─────────────────────────────────────────────────────────────┤
│  Tauri 2.0 IPC 桥接层                                       │
│  └── Session Token 验证                                      │
├─────────────────────────────────────────────────────────────┤
│  Rust 后端                                                  │
│  ├── crypto/ — AES-256-GCM + Argon2id                      │
│  ├── db/ — SQLCipher (sqlx)                                 │
│  ├── commands/ — Tauri 命令处理                              │
│  └── native_messaging/ — 浏览器扩展通信                      │
└─────────────────────────────────────────────────────────────┘
```

### 技术栈

| 组件 | 技术 | 说明 |
|------|------|------|
| 桌面框架 | Tauri 2.0 | 系统 WebView + Rust 后端 |
| 后端语言 | Rust (stable) | 加密/存储/IPC/系统调用 |
| 前端框架 | Vue 3 + TypeScript | Composition API |
| 构建工具 | Vite 6 | 快速开发体验 |
| 状态管理 | Pinia | Vue 3 官方推荐 |
| 加密方案 | AES-256-GCM | 字段级加密 |
| 密钥派生 | Argon2id | 抗暴力破解 |
| 数据库 | SQLCipher | 加密 SQLite |
| 运行时 | tokio | 异步运行时 |
| 序列化 | serde + serde_json | JSON 处理 |

---

## 🔒 安全模型

KeyVault 的安全设计遵循"零信任"原则，所有敏感数据在存储、传输、显示全链路加密。

### 安全红线

1. **主密码零存储** — 只存 Argon2id 哈希，原始密码立即清零
2. **密钥不落盘** — 加密密钥仅存在于内存，使用 Zeroizing 包装
3. **锁定时清零** — 锁定时同时清零加密密钥和销毁所有会话
4. **字段级加密** — 每个敏感字段独立加密，独立随机 nonce
5. **前端零缓存** — 解密数据不持久化到 localStorage/sessionStorage
6. **IPC 全验证** — 所有 Tauri 命令必须验证 session token

### 加密流程

```
用户输入主密码
    ↓
Argon2id 派生加密密钥 (memoryCost=65536, timeCost=3)
    ↓
派生密钥仅存在于内存 (Zeroizing<[u8; 32]>)
    ↓
每个敏感字段独立加密 (AES-256-GCM + 随机 12 字节 nonce)
    ↓
密文存储到 SQLCipher 数据库
```

### 暴力破解防护

- 连续 5 次密码错误 → 锁定 5 分钟
- 使用 AtomicU32/AtomicI64 计数器，线程安全
- 锁定期间拒绝所有解锁尝试

---

## 🚀 快速开始

### 环境要求

- **操作系统:** Windows 10/11 (64-bit)
- **Node.js:** 18+ (推荐 20+)
- **Rust:** 1.70+ (stable)
- **Visual Studio Build Tools:** 用于编译原生模块

### 安装

```bash
# 克隆仓库
git clone https://github.com/your-username/keyvault-v3.git
cd keyvault-v3

# 安装依赖
npm install

# 开发模式运行
npm run tauri dev

# 构建生产版本
npm run tauri build
```

### 首次使用

1. 启动 KeyVault，进入初始化向导
2. 设置主密码（建议 12 位以上，包含大小写字母、数字、符号）
3. 主密码将用于加密所有数据，请务必牢记
4. 初始化完成后即可开始添加密码条目

---

## 📱 浏览器扩展

KeyVault 提供浏览器扩展，实现自动填充和密码保存。

### 安装扩展

1. 构建项目后，扩展位于 `extension/` 目录
2. 在 Chrome 中打开 `chrome://extensions/`
3. 启用"开发者模式"
4. 点击"加载已解压的扩展程序"，选择 `extension/` 目录

### 功能说明

- **自动检测** — 自动识别登录表单
- **一键填充** — 点击扩展图标选择凭据填充
- **保存提示** — 登录时提示保存新凭据
- **安全通信** — 通过 Native Messaging 与主程序通信，扩展不持有任何密钥

---

## 🔧 配置指南

### 应用设置

在设置页面可以配置：

- **自动锁定时间** — 空闲超时自动锁定（默认 5 分钟）
- **剪贴板清除时间** — 复制后自动清除剪贴板（默认 30 秒）
- **主题** — 暗黑主题（默认）

### 数据存储

- 数据库位置: `%APPDATA%/keyvault/keyvault.db`
- 数据库加密: SQLCipher (AES-256)
- 配置文件: 应用内设置，无外部配置文件

---

## 🔄 同步说明

KeyVault 支持 WebDAV 同步，实现私有化部署。在 **设置 → 数据** 中配置 NAS 地址与凭据。

### 计划支持的同步方式

- **WebDAV** — 支持群晖、威联通等 NAS 设备
- **私有化部署** — 数据完全在您的网络内
- **冲突处理** — 智能合并策略，防止数据丢失

### 同步架构

```
设备 A (Windows)          设备 B (Windows)
    ↓                         ↓
KeyVault 本地数据库      KeyVault 本地数据库
    ↓                         ↓
    └─────────┬─────────────┘
              ↓
         NAS (WebDAV)
         加密同步文件
```

---

## 🛠️ 开发指南

### 项目结构

```
keyvault-v3/
├── src-tauri/          # Rust 后端
│   ├── src/
│   │   ├── main.rs     # 入口，注册所有命令
│   │   ├── error.rs    # 统一错误类型
│   │   ├── state.rs    # AppState (加密密钥、DB 连接池)
│   │   ├── crypto/     # kdf.rs, cipher.rs, session.rs
│   │   ├── db/         # migrations/, schema.rs, queries.rs
│   │   └── commands/   # auth.rs, vault.rs, generator.rs, breach.rs
│   └── Cargo.toml
├── src/                # Vue 3 前端
│   ├── bridge/         # Tauri IPC 桥接层
│   ├── components/     # UI 组件
│   ├── composables/    # 组合式函数
│   ├── stores/         # Pinia 状态管理
│   └── views/          # 页面视图
├── extension/          # 浏览器扩展 (Manifest V3)
└── docs/               # 文档
```

### 开发命令

```bash
# 启动 Vite 开发服务器
npm run dev

# 启动 Tauri 开发模式 (Rust + Vue)
npm run tauri dev

# 构建前端
npm run build

# 构建生产包
npm run tauri build

# TypeScript 类型检查
npm run type-check

# 运行 Rust 测试
cd src-tauri && cargo test
```

### 测试

项目包含 25 个 Rust 单元测试：

```bash
cd src-tauri && cargo test

# 测试内容包括:
# - crypto: 密钥派生、加解密、会话管理
# - db: 配置读写、分组 CRUD、审计日志
```

---

## 📋 开发路线图

### ✅ 已完成

- [x] 核心加密模块 (AES-256-GCM + Argon2id)
- [x] 数据库层 (SQLCipher + 迁移)
- [x] 认证系统 (设置、解锁、锁定、修改密码)
- [x] 条目 CRUD (创建、读取、更新、删除)
- [x] 字段级加密
- [x] 会话管理
- [x] 前端 UI 组件库
- [x] 命令面板 (Ctrl+K)
- [x] 密码生成器
- [x] 泄露检测 (HIBP)
- [x] 浏览器扩展基础架构
- [x] 暴力破解防护
- [x] 审计日志

### 🚧 进行中

- [ ] 人工目视验收（对照 UI 原型 sign-off）
- [ ] 浏览器扩展（暂停，待桌面端验收后单独开发）

### 📅 计划中

- [ ] 主题切换
- [ ] 移动端支持 (iOS/Android)
- [ ] 多语言支持

### ✅ 近期完成

- [x] WebDAV 同步（NAS 上传/下载/合并）
- [x] 数据导入/导出 UI
- [x] 紧急擦除
- [x] Diceware 密码生成模式

---

## ⚠️ 安全提醒

### 主密码安全

- **牢记主密码** — 主密码无法恢复，忘记将无法访问所有数据
- **使用强密码** — 建议 12 位以上，包含大小写字母、数字、符号
- **定期更换** — 建议每 6-12 个月更换一次主密码

### 数据备份

- **定期备份数据库文件** — 位于 `%APPDATA%/keyvault/keyvault.db`
- **备份加密** — 数据库本身已加密，但仍建议额外加密备份
- **多地存储** — 建议在不同位置保留备份

### 安全使用

- **及时锁定** — 离开电脑时手动锁定 (Ctrl+L)
- **检查泄露** — 定期使用泄露检测功能检查密码安全
- **更新软件** — 及时更新到最新版本

---

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！

### 贡献方式

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 开发规范

- 遵循项目代码风格
- 添加必要的测试
- 更新相关文档
- 提交前运行 `npm run type-check` 和 `cargo test`

---

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE) 开源。

---

## 🙏 致谢

- [Tauri](https://tauri.app/) — 优秀的桌面应用框架
- [Vue.js](https://vuejs.org/) — 渐进式 JavaScript 框架
- [RustCrypto](https://github.com/RustCrypto) — Rust 加密库
- [Have I Been Pwned](https://haveibeenpwned.com/) — 泄露检测服务

---

<div align="center">

**KeyVault — 您的数据，您的规则**

[报告问题](https://github.com/your-username/keyvault-v3/issues) · [功能请求](https://github.com/your-username/keyvault-v3/issues) · [文档](docs/)

</div>
