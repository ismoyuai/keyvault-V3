# KeyVault 浏览器扩展设计文档

> **范围**：MVP — 仅自动填充（不含保存提示、密码生成器集成）
> **目标浏览器**：Chrome / Edge（Manifest V3）
> **日期**：2026-06-03

---

## 1. 整体架构

### 1.1 数据流

```
网页登录表单
  → Content Script (detector.js) 检测 password input
  → 向 Service Worker 发送 FIND_CREDENTIALS { url }
  → Service Worker 通过 Native Messaging 转发到 Tauri 主进程
  → Tauri 查询 SQLCipher 数据库，按 URL 匹配返回条目列表（仅 title + username，不含密码）
  → Content Script 在 password input 旁注入 KV 图标
  → 用户点击图标：
      单凭据 → 请求密码 → 填充
      多凭据 → 弹出下拉列表 → 用户选择 → 请求密码 → 填充
```

### 1.2 URL 匹配策略

`find_credentials(url)` 的匹配逻辑：
1. 从 URL 提取 hostname（如 `github.com`）
2. 查询 entries 表中 `subtitle` 字段包含该 hostname 的条目
3. 同时匹配 `fields` 表中 `field_key = 'url'` 且 `enc_value` 解密后包含该 hostname 的条目
4. 返回去重后的条目列表（id, title, username）

### 1.3 安全约束

- 密码仅在用户明确点击后通过 IPC 获取，获取后立即填充，**不缓存**
- 扩展不使用 `chrome.storage` 存储任何解密数据
- `storage.session` 仅缓存连接状态（connected/disconnected/locked），扩展重启后丢失
- Native Messaging 连接断开时图标变灰，不降级工作
- 每次填充请求都经过 Tauri 侧 session token 验证

### 1.4 组件

| 组件 | 文件 | 职责 |
|------|------|------|
| Content Script - Detector | `content/detector.js` | 检测登录表单，通知 Service Worker |
| Content Script - Filler | `content/filler.js` | 注入 KV 图标，处理填充交互 |
| Service Worker | `background/service-worker.js` | 管理 Native Messaging 连接，消息路由 |
| Popup | `popup/popup.html` + `.js` + `.css` | 点击扩展图标弹出的面板 |
| Native Messaging Host | Rust 侧 `native_messaging/host.rs` | 接收扩展请求，调用 Tauri 命令 |

---

## 2. Content Script

### 2.1 detector.js — 表单检测

**检测策略**（按优先级）：

1. **主策略**：扫描 `input[type="password"]`，向上查找最近的 `form` 或容器，再找关联的 username/email input
2. **辅助策略**：`autocomplete="current-password"` / `autocomplete="new-password"` 属性
3. **SPA 适配**：`MutationObserver` 监听 `document.body` 的 `childList + subtree` 变化

**Username 查找规则**（在 password input 的容器内）：
```
1. input[type="email"]
2. input[autocomplete="username"] 或 input[autocomplete="email"]
3. input[name*="user" i] 或 input[name*="email" i] 或 input[name*="login" i]
4. input[type="text"]（fallback）
```

**去重**：用 `element.id || element.name || "${offsetTop}-${offsetLeft}"` 作为 key，已处理的元素跳过。

**流程**：
1. 页面加载完成后执行初始检测
2. 如发现表单 → 向 Service Worker 发送 `FIND_CREDENTIALS { url: location.href }`
3. 如返回匹配条目 → 通知 filler.js 注入图标
4. 启动 MutationObserver 监听动态表单

### 2.2 filler.js — 图标注入与填充

**图标样式**：
- 20x20px 圆角方块（`border-radius: 4px`）
- 背景：`rgba(56, 139, 253, 0.9)`（品牌蓝）
- 文字：白色 "KV"，10px，font-weight: 600
- 定位：`position: absolute` 在 password input 的父容器内，右对齐
- 父容器需设置 `position: relative`（如已是 static 则设置）

**交互**：
- 单凭据：点击图标 → 向 Service Worker 请求 `FILL_CREDENTIAL { entryId }` → 获取 username + password → 填充
- 多凭据：点击图标 → 弹出下浮列表 → 用户选择 → 请求并填充
- 填充后图标变为绿色 ✓，3 秒后恢复
- 填充方式：设置 `input.value` + 派发 `input` + `change` + `blur` 事件（兼容 React/Vue 受控组件）

**下拉列表**（多凭据时）：
- 出现在图标下方，200px 宽，最大 200px 高，`overflow-y: auto`
- 每项：title（主）+ username（副），高度 36px
- 样式：暗黑主题，`background: #1c2128`，`border: 1px solid rgba(255,255,255,0.1)`
- 点击外部或按 Esc 关闭

---

## 3. Service Worker

### 3.1 职责

- 管理与 Tauri 主进程的 Native Messaging 连接
- 接收 Content Script 和 Popup 的消息，路由到 Native Host
- 维护连接状态（connected / disconnected / locked）

### 3.2 Native Messaging 协议

- Host 名：`com.keyvault.app`
- 通信格式：stdio JSON（每条消息 = 4 字节小端长度头 + JSON payload）
- 请求超时：5 秒
- 连接断开时拒绝所有 pending 请求

### 3.3 消息类型

| 方向 | Action | 参数 | 返回 |
|------|--------|------|------|
| CS → SW | `FIND_CREDENTIALS` | `{ url }` | `{ entries: [{ id, title, username }] }` |
| CS → SW | `FILL_CREDENTIAL` | `{ entryId }` | `{ username, password }` |
| Popup → SW | `GET_STATUS` | — | `{ status: "connected" \| "disconnected" \| "locked" }` |
| Popup → SW | `SEARCH` | `{ query }` | `{ entries: [{ id, title, username }] }` |
| SW → Native | `find_credentials` | `{ url }` | `{ entries }` |
| SW → Native | `get_entry_for_fill` | `{ entryId }` | `{ username, password }` |
| SW → Native | `get_status` | — | `{ status }` |
| SW → Native | `search` | `{ query }` | `{ entries }` |

### 3.4 连接管理

```javascript
// Service Worker 中
let nativePort = null;

function getOrCreateNativePort() {
  if (nativePort) return nativePort;
  nativePort = chrome.runtime.connectNative('com.keyvault.app');
  nativePort.onDisconnect.addListener(() => {
    nativePort = null;
    // 拒绝所有 pending 请求
  });
  return nativePort;
}
```

---

## 4. Popup UI

### 4.1 状态机

| 状态 | 条件 | 显示 |
|------|------|------|
| 未连接 | Native Messaging 连接失败 | 灰色图标 + "请先启动 KeyVault 应用" + 打开应用按钮 |
| 已锁定 | 连接成功但 Tauri 未解锁 | 锁图标 + "KeyVault 已锁定" + 解锁按钮 |
| 已连接 | 连接成功且已解锁 | 搜索框 + 当前页匹配凭据列表 |

### 4.2 布局（已连接状态）

```
┌──────────────────────────────────────┐
│ 🔐 KeyVault               ● 已连接   │  标题栏 36px
├──────────────────────────────────────┤
│ 🔍 搜索密码...                        │  搜索框 36px
├──────────────────────────────────────┤
│ 当前页面                              │  section 标题
│ ┌──────────────────────────────────┐ │
│ │ GitHub                           │ │  凭据卡片
│ │ ismoyuai@gmail.com               │ │  52px
│ │             [填充] [复制密码]     │ │
│ └──────────────────────────────────┘ │
│ ┌──────────────────────────────────┐ │
│ │ GitHub (工作)                    │ │
│ │ work@company.com                 │ │
│ │             [填充] [复制密码]     │ │
│ └──────────────────────────────────┘ │
├──────────────────────────────────────┤
│ [打开应用]          [生成密码]        │  底部栏 36px
└──────────────────────────────────────┘
```

- 宽度 360px，最大高度 540px
- 暗黑高密度设计语言（与主应用一致：`--bg-surface: #161b22` 等）
- 搜索实时过滤全库条目（通过 Native Messaging 查询）
- 无匹配时显示"当前页面无匹配凭据"

---

## 5. Rust 侧 Native Messaging Host

### 5.1 文件位置

`src-tauri/src/native_messaging/host.rs`

### 5.2 职责

- 独立的 stdin/stdout 循环，读取 4 字节长度头 + JSON payload
- 复用 AppState（加密密钥、DB 连接池、会话管理）
- 每个请求验证 session token 后执行对应命令
- JSON-RPC 风格：`{ "requestId": 1, "action": "find_credentials", "url": "..." }`

### 5.3 注册方式

在 Tauri 的 `main.rs` 中，当以 `--native-messaging` 参数启动时进入 Native Messaging 模式，否则正常启动 GUI。

```rust
fn main() {
    if std::env::args().any(|a| a == "--native-messaging") {
        keyvault_lib::run_native_messaging();
    } else {
        keyvault_lib::run();
    }
}
```

### 5.4 Native Messaging Host 注册

Windows 注册表路径：
```
HKEY_CURRENT_USER\SOFTWARE\Google\Chrome\NativeMessagingHosts\com.keyvault.app
```
值为 host manifest JSON 文件路径。

Host manifest (`com.keyvault.app.json`)：
```json
{
  "name": "com.keyvault.app",
  "description": "KeyVault Native Messaging Host",
  "path": "keyvault.exe",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://<EXTENSION_ID>/"]
}
```

---

## 6. Manifest 配置

```json
{
  "manifest_version": 3,
  "name": "KeyVault",
  "version": "3.0.0",
  "description": "本地密码管理器 — 安全自动填充",
  "permissions": ["activeTab", "scripting", "storage", "nativeMessaging"],
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
      "run_at": "document_idle",
      "all_frames": false
    }
  ],
  "content_security_policy": {
    "extension_pages": "script-src 'self'; object-src 'none';"
  }
}
```

- `host_permissions` 为空：不请求全站权限
- `storage` 仅用于 `storage.session` 缓存连接状态
- `all_frames: false`：仅主 frame，避免 iframe 中重复注入

---

## 7. 目录结构

```
extension/
├── manifest.json
├── background/
│   └── service-worker.js
├── content/
│   ├── detector.js
│   └── filler.js
├── popup/
│   ├── popup.html
│   ├── popup.js
│   └── popup.css
└── icons/
    ├── 16.png
    ├── 48.png
    └── 128.png
```

---

## 8. MVP 范围确认

**包含**：
- [x] 表单检测（password input + autocomplete 属性）
- [x] 输入框内 KV 图标注入
- [x] 单凭据直接填充
- [x] 多凭据下拉选择填充
- [x] Popup 当前页匹配凭据列表
- [x] Popup 搜索全库
- [x] 未连接/锁定状态处理
- [x] Native Messaging Host（Rust 侧）
- [x] Host 注册脚本

**不含**（后续迭代）：
- [ ] 保存新凭据提示
- [ ] Popup 内密码生成器
- [ ] 快捷键触发填充
- [ ] 多语言支持
