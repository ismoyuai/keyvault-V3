# KeyVault 浏览器扩展实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Chrome/Edge 浏览器扩展，通过 Native Messaging 与 Tauri 主进程通信，实现登录表单自动填充。

**Architecture:** Content Script 检测登录表单并注入 KV 图标 → Service Worker 管理 Native Messaging 连接 → Rust 侧 Native Messaging Host 处理请求并查询数据库。密码仅在用户点击后获取，不缓存。

**Tech Stack:** Chrome Extension Manifest V3, Native Messaging, Tauri 2.0, Rust, SQLCipher

---

## 文件结构

```
extension/
├── manifest.json                    # 扩展配置
├── background/
│   └── service-worker.js            # 消息路由 + Native Messaging 连接
├── content/
│   ├── detector.js                  # 表单检测
│   └── filler.js                    # 图标注入 + 填充逻辑
├── popup/
│   ├── popup.html                   # Popup HTML
│   ├── popup.js                     # Popup 逻辑
│   └── popup.css                    # Popup 暗黑样式
└── icons/
    ├── 16.png                       # 扩展图标
    ├── 48.png
    └── 128.png

src-tauri/src/
├── native_messaging/
│   ├── mod.rs                       # 模块导出
│   └── host.rs                      # Native Messaging Host 主循环
├── commands/native_ext.rs           # 扩展专用 Tauri 命令
└── lib.rs                           # 修改：添加 native-messaging 模式

scripts/
└── register-host.bat                # Windows Host 注册脚本
```

---

### Task 1: 扩展 Manifest 与目录结构

**Files:**
- Create: `extension/manifest.json`
- Create: `extension/icons/16.png`, `48.png`, `128.png`

- [ ] **Step 1: 创建扩展目录**

```bash
mkdir -p extension/background extension/content extension/popup extension/icons
```

- [ ] **Step 2: 创建 manifest.json**

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

- [ ] **Step 3: 生成扩展图标**

使用主应用的 `app-icon.png` 缩放生成 16/48/128px 图标，或创建占位图标。

- [ ] **Step 4: Commit**

```bash
git add extension/manifest.json extension/icons/
git commit -m "feat(extension): add manifest and icon placeholders"
```

---

### Task 2: Service Worker — Native Messaging 连接管理

**Files:**
- Create: `extension/background/service-worker.js`

- [ ] **Step 1: 创建 service-worker.js**

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
      const error = chrome.runtime.lastError?.message || 'KeyVault 连接断开';
      for (const [id, pending] of pendingRequests) {
        clearTimeout(pending.timeout);
        pending.reject(new Error(error));
      }
      pendingRequests.clear();
      // 更新连接状态
      chrome.storage.session.set({ connectionStatus: 'disconnected' });
    });

    chrome.storage.session.set({ connectionStatus: 'connected' });
    return nativePort;
  } catch {
    chrome.storage.session.set({ connectionStatus: 'disconnected' });
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
      return await sendToNative('get_entry_for_fill', { entryId: message.entryId });

    case 'SEARCH':
      return await sendToNative('search', { query: message.query });

    default:
      throw new Error(`未知操作: ${message.action}`);
  }
}
```

- [ ] **Step 2: Commit**

```bash
git add extension/background/service-worker.js
git commit -m "feat(extension): add service worker with Native Messaging"
```

---

### Task 3: Content Script — detector.js 表单检测

**Files:**
- Create: `extension/content/detector.js`

- [ ] **Step 1: 创建 detector.js**

```javascript
/**
 * 表单检测器
 * 检测页面中的登录表单，通知 Service Worker 查询匹配凭据
 */
class FormDetector {
  constructor() {
    this.detectedGroups = new Set();
    this.observer = null;
  }

  /**
   * 获取页面中所有登录表单组
   * 每组包含一个 password input 和关联的 username input
   */
  getFormGroups() {
    const groups = [];

    // 策略1：标准 password input
    document.querySelectorAll('input[type="password"]').forEach(pwdInput => {
      if (pwdInput.dataset.kvHandled) return;

      const key = this.getElementKey(pwdInput);
      if (this.detectedGroups.has(key)) return;

      this.detectedGroups.add(key);
      const usernameField = this.findUsernameField(pwdInput);
      groups.push({ passwordField: pwdInput, usernameField });
    });

    // 策略2：autocomplete 提示
    document.querySelectorAll(
      '[autocomplete="current-password"],[autocomplete="new-password"]'
    ).forEach(input => {
      if (input.type === 'password') return; // 已在策略1处理
      const key = this.getElementKey(input);
      if (this.detectedGroups.has(key)) return;

      this.detectedGroups.add(key);
      groups.push({
        passwordField: input,
        usernameField: this.findUsernameField(input),
      });
    });

    return groups;
  }

  /**
   * 在 password input 的容器中查找 username 输入框
   */
  findUsernameField(referenceInput) {
    const container =
      referenceInput.closest('form') ||
      referenceInput.closest('[class*="login"],[class*="signin"],[class*="auth"]') ||
      referenceInput.parentElement?.parentElement;

    if (!container) return null;

    return (
      container.querySelector('input[type="email"]') ||
      container.querySelector('input[autocomplete="username"],input[autocomplete="email"]') ||
      container.querySelector('input[name*="user" i],input[name*="email" i],input[name*="login" i]') ||
      container.querySelector('input[type="text"]')
    );
  }

  getElementKey(el) {
    return el.id || el.name || `${el.offsetTop}-${el.offsetLeft}`;
  }

  /**
   * 监听 SPA 动态加载的表单
   */
  watchDynamic() {
    this.observer = new MutationObserver(() => {
      const newGroups = this.getFormGroups();
      if (newGroups.length > 0) {
        this.notifyAndInject(newGroups);
      }
    });

    this.observer.observe(document.body, { childList: true, subtree: true });
  }

  /**
   * 通知 Service Worker 并注入图标
   */
  async notifyAndInject(groups) {
    try {
      const result = await chrome.runtime.sendMessage({
        action: 'FIND_CREDENTIALS',
        url: location.href,
      });

      if (result?.error) return;
      if (!result?.entries?.length) return;

      // 通知 filler.js 注入图标
      window.postMessage({
        type: 'KV_INJECT_ICONS',
        groups: groups.map(g => ({
          passwordFieldId: g.passwordField.id || g.passwordField.name || '',
          passwordFieldSelector: this.buildSelector(g.passwordField),
        })),
        entries: result.entries,
      }, '*');
    } catch {
      // Service Worker 未就绪或连接断开，静默忽略
    }
  }

  buildSelector(el) {
    if (el.id) return `#${el.id}`;
    if (el.name) return `input[name="${el.name}"]`;
    return null;
  }
}

// 初始化
const detector = new FormDetector();

// 初始检测
const initialGroups = detector.getFormGroups();
if (initialGroups.length > 0) {
  detector.notifyAndInject(initialGroups);
}

// 监听动态表单
detector.watchDynamic();
```

- [ ] **Step 2: Commit**

```bash
git add extension/content/detector.js
git commit -m "feat(extension): add form detector content script"
```

---

### Task 4: Content Script — filler.js 图标注入与填充

**Files:**
- Create: `extension/content/filler.js`

- [ ] **Step 1: 创建 filler.js**

```javascript
/**
 * 填充器
 * 在 password input 旁注入 KV 图标，处理点击填充
 */
class FormFiller {
  constructor() {
    this.injectedIcons = new Map(); // passwordField -> iconElement
    this.dropdown = null;
    this.init();
  }

  init() {
    // 监听 detector 的消息
    window.addEventListener('message', (e) => {
      if (e.data?.type === 'KV_INJECT_ICONS') {
        this.handleInject(e.data.groups, e.data.entries);
      }
    });

    // 点击外部关闭下拉
    document.addEventListener('click', (e) => {
      if (this.dropdown && !this.dropdown.contains(e.target)) {
        this.closeDropdown();
      }
    });

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') this.closeDropdown();
    });
  }

  handleInject(groups, entries) {
    for (const group of groups) {
      const pwdField = group.passwordFieldSelector
        ? document.querySelector(group.passwordFieldSelector)
        : document.getElementById(group.passwordFieldId);

      if (!pwdField || this.injectedIcons.has(pwdField)) continue;

      this.injectIcon(pwdField, entries);
    }
  }

  /**
   * 在 password input 旁注入 KV 图标
   */
  injectIcon(pwdField, entries) {
    const container = pwdField.parentElement;
    if (!container) return;

    // 确保容器可定位
    const containerPos = getComputedStyle(container).position;
    if (containerPos === 'static') {
      container.style.position = 'relative';
    }

    const icon = document.createElement('div');
    icon.className = 'kv-fill-icon';
    icon.textContent = 'KV';
    icon.title = `KeyVault: 找到 ${entries.length} 个匹配凭据`;

    // 样式
    Object.assign(icon.style, {
      position: 'absolute',
      right: '8px',
      top: '50%',
      transform: 'translateY(-50%)',
      width: '20px',
      height: '20px',
      background: 'rgba(56, 139, 253, 0.9)',
      borderRadius: '4px',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      fontSize: '10px',
      fontWeight: '600',
      color: '#fff',
      cursor: 'pointer',
      zIndex: '2147483647',
      fontFamily: '-apple-system, BlinkMacSystemFont, sans-serif',
      transition: 'all 0.15s ease',
    });

    icon.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();

      if (entries.length === 1) {
        this.fillCredential(pwdField, entries[0].id, entries[0].username);
      } else {
        this.showDropdown(icon, pwdField, entries);
      }
    });

    container.appendChild(icon);
    this.injectedIcons.set(pwdField, icon);

    // 调整 input 的 padding-right 避免文字被图标遮挡
    const currentPadding = parseInt(getComputedStyle(pwdField).paddingRight) || 0;
    pwdField.style.paddingRight = `${currentPadding + 28}px`;
  }

  /**
   * 显示多凭据下拉列表
   */
  showDropdown(anchor, pwdField, entries) {
    this.closeDropdown();

    const dropdown = document.createElement('div');
    dropdown.className = 'kv-dropdown';

    Object.assign(dropdown.style, {
      position: 'absolute',
      top: '100%',
      right: '0',
      width: '200px',
      maxHeight: '200px',
      overflowY: 'auto',
      background: '#1c2128',
      border: '1px solid rgba(255,255,255,0.1)',
      borderRadius: '6px',
      boxShadow: '0 8px 24px rgba(0,0,0,0.5)',
      zIndex: '2147483647',
      marginTop: '4px',
    });

    for (const entry of entries) {
      const item = document.createElement('div');
      Object.assign(item.style, {
        padding: '8px 12px',
        cursor: 'pointer',
        borderBottom: '1px solid rgba(255,255,255,0.05)',
        transition: 'background 0.1s',
      });

      item.innerHTML = `
        <div style="font-size:12px;font-weight:500;color:#e6edf3;">${this.escapeHtml(entry.title)}</div>
        <div style="font-size:11px;color:#8b949e;margin-top:2px;">${this.escapeHtml(entry.username || '')}</div>
      `;

      item.addEventListener('mouseenter', () => {
        item.style.background = '#22272e';
      });
      item.addEventListener('mouseleave', () => {
        item.style.background = 'transparent';
      });
      item.addEventListener('click', () => {
        this.fillCredential(pwdField, entry.id, entry.username);
        this.closeDropdown();
      });

      dropdown.appendChild(item);
    }

    anchor.style.position = 'relative';
    anchor.appendChild(dropdown);
    this.dropdown = dropdown;
  }

  closeDropdown() {
    if (this.dropdown) {
      this.dropdown.remove();
      this.dropdown = null;
    }
  }

  /**
   * 请求并填充凭据
   */
  async fillCredential(pwdField, entryId, username) {
    try {
      const result = await chrome.runtime.sendMessage({
        action: 'FILL_CREDENTIAL',
        entryId,
      });

      if (result?.error) {
        console.error('KeyVault fill error:', result.error);
        return;
      }

      // 填充 password
      this.setInputValue(pwdField, result.password);

      // 填充 username（在容器中查找）
      const container = pwdField.closest('form') || pwdField.parentElement?.parentElement;
      if (container && result.username) {
        const usernameField =
          container.querySelector('input[type="email"]') ||
          container.querySelector('input[autocomplete="username"]') ||
          container.querySelector('input[name*="user" i],input[name*="email" i]') ||
          container.querySelector('input[type="text"]');

        if (usernameField) {
          this.setInputValue(usernameField, result.username);
        }
      }

      // 图标变为成功状态
      this.showSuccess(pwdField);
    } catch (err) {
      console.error('KeyVault fill error:', err);
    }
  }

  /**
   * 设置 input 值并触发事件（兼容 React/Vue）
   */
  setInputValue(input, value) {
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
      window.HTMLInputElement.prototype, 'value'
    )?.set;

    if (nativeInputValueSetter) {
      nativeInputValueSetter.call(input, value);
    } else {
      input.value = value;
    }

    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.dispatchEvent(new Event('change', { bubbles: true }));
    input.dispatchEvent(new Event('blur', { bubbles: true }));
  }

  /**
   * 填充成功后图标变绿 3 秒
   */
  showSuccess(pwdField) {
    const icon = this.injectedIcons.get(pwdField);
    if (!icon) return;

    icon.textContent = '✓';
    icon.style.background = 'rgba(63, 185, 80, 0.9)';

    setTimeout(() => {
      icon.textContent = 'KV';
      icon.style.background = 'rgba(56, 139, 253, 0.9)';
    }, 3000);
  }

  escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }
}

new FormFiller();
```

- [ ] **Step 2: Commit**

```bash
git add extension/content/filler.js
git commit -m "feat(extension): add filler with icon injection and credential fill"
```

---

### Task 5: Popup UI — HTML 与 CSS

**Files:**
- Create: `extension/popup/popup.html`
- Create: `extension/popup/popup.css`

- [ ] **Step 1: 创建 popup.html**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>KeyVault</title>
  <link rel="stylesheet" href="popup.css">
</head>
<body>
  <div id="app">
    <!-- 标题栏 -->
    <header class="header">
      <span class="header-title">🔐 KeyVault</span>
      <span id="status" class="status status--disconnected">● 未连接</span>
    </header>

    <!-- 未连接状态 -->
    <div id="view-disconnected" class="view">
      <div class="empty-state">
        <div class="empty-icon">🔌</div>
        <p class="empty-title">请先启动 KeyVault 应用</p>
        <button id="btn-open-app" class="btn btn--primary">打开应用</button>
      </div>
    </div>

    <!-- 已锁定状态 -->
    <div id="view-locked" class="view" style="display:none">
      <div class="empty-state">
        <div class="empty-icon">🔒</div>
        <p class="empty-title">KeyVault 已锁定</p>
        <button id="btn-unlock" class="btn btn--primary">解锁</button>
      </div>
    </div>

    <!-- 已连接状态 -->
    <div id="view-connected" class="view" style="display:none">
      <!-- 搜索框 -->
      <div class="search-bar">
        <input id="search-input" type="text" placeholder="搜索密码..." class="search-input">
      </div>

      <!-- 当前页匹配 -->
      <section id="section-current" class="section">
        <div class="section-title">当前页面</div>
        <div id="current-entries" class="entry-list"></div>
        <div id="no-matches" class="empty-hint" style="display:none">当前页面无匹配凭据</div>
      </section>

      <!-- 搜索结果 -->
      <section id="section-search" class="section" style="display:none">
        <div class="section-title">搜索结果</div>
        <div id="search-results" class="entry-list"></div>
        <div id="no-results" class="empty-hint" style="display:none">无匹配结果</div>
      </section>
    </div>

    <!-- 底部栏 -->
    <footer class="footer">
      <button id="btn-open-main" class="footer-btn">打开应用</button>
      <button id="btn-generate" class="footer-btn">生成密码</button>
    </footer>
  </div>

  <script src="popup.js"></script>
</body>
</html>
```

- [ ] **Step 2: 创建 popup.css**

```css
* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  width: 360px;
  max-height: 540px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  font-size: 13px;
  background: #161b22;
  color: #e6edf3;
  overflow: hidden;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255,255,255,0.05);
}

.header-title { font-weight: 600; font-size: 13px; }

.status { font-size: 10px; }
.status--connected { color: #3fb950; }
.status--disconnected { color: #8b949e; }
.status--locked { color: #d29922; }

.view { min-height: 200px; }

.search-bar {
  padding: 8px 12px;
  border-bottom: 1px solid rgba(255,255,255,0.05);
}

.search-input {
  width: 100%;
  padding: 6px 10px;
  background: #2d333b;
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 6px;
  color: #e6edf3;
  font-size: 12px;
  outline: none;
}

.search-input:focus { border-color: #388bfd; }

.section { padding: 8px 12px; }

.section-title {
  font-size: 10px;
  color: #484f58;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 6px;
}

.entry-list { display: flex; flex-direction: column; gap: 6px; }

.entry-card {
  background: #1c2128;
  border-radius: 6px;
  padding: 10px;
  transition: background 0.1s;
}

.entry-card:hover { background: #22272e; }

.entry-title { font-size: 12px; font-weight: 500; }
.entry-username { font-size: 11px; color: #8b949e; margin-top: 2px; }

.entry-actions {
  display: flex;
  gap: 6px;
  margin-top: 6px;
}

.btn {
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.1s;
}

.btn:hover { opacity: 0.9; }
.btn--primary { background: #388bfd; color: #fff; }
.btn--small { padding: 2px 8px; font-size: 10px; border-radius: 4px; }
.btn--fill { background: #388bfd; color: #fff; }
.btn--copy { background: #2d333b; color: #8b949e; }

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  gap: 12px;
}

.empty-icon { font-size: 32px; }
.empty-title { font-size: 13px; color: #8b949e; }
.empty-hint { font-size: 12px; color: #484f58; text-align: center; padding: 20px; }

.footer {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  border-top: 1px solid rgba(255,255,255,0.05);
}

.footer-btn {
  background: none;
  border: none;
  color: #484f58;
  font-size: 11px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
}

.footer-btn:hover { color: #8b949e; background: #1c2128; }
```

- [ ] **Step 3: Commit**

```bash
git add extension/popup/popup.html extension/popup/popup.css
git commit -m "feat(extension): add popup HTML and dark theme CSS"
```

---

### Task 6: Popup UI — JavaScript 逻辑

**Files:**
- Create: `extension/popup/popup.js`

- [ ] **Step 1: 创建 popup.js**

```javascript
/**
 * Popup 逻辑
 * 状态机：disconnected → locked → connected
 */
const $ = (id) => document.getElementById(id);

let currentTab = null;

async function init() {
  // 获取当前 tab
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  currentTab = tab;

  // 检查连接状态
  const status = await sendToBackground({ action: 'GET_STATUS' });

  if (status?.status === 'connected') {
    showView('connected');
    loadCurrentPageEntries();
  } else if (status?.status === 'locked') {
    showView('locked');
  } else {
    showView('disconnected');
  }

  // 绑定事件
  $('btn-open-app')?.addEventListener('click', () => chrome.runtime.sendNativeMessage('com.keyvault.app', { action: 'open_app' }));
  $('btn-unlock')?.addEventListener('click', () => chrome.runtime.sendNativeMessage('com.keyvault.app', { action: 'open_app' }));
  $('btn-open-main')?.addEventListener('click', () => chrome.runtime.sendNativeMessage('com.keyvault.app', { action: 'open_app' }));
  $('btn-generate')?.addEventListener('click', () => chrome.runtime.sendNativeMessage('com.keyvault.app', { action: 'open_app' }));

  $('search-input')?.addEventListener('input', debounce(handleSearch, 300));
}

function showView(state) {
  $('view-disconnected').style.display = state === 'disconnected' ? 'block' : 'none';
  $('view-locked').style.display = state === 'locked' ? 'block' : 'none';
  $('view-connected').style.display = state === 'connected' ? 'block' : 'none';

  const statusEl = $('status');
  statusEl.className = 'status';
  if (state === 'connected') {
    statusEl.textContent = '● 已连接';
    statusEl.classList.add('status--connected');
  } else if (state === 'locked') {
    statusEl.textContent = '● 已锁定';
    statusEl.classList.add('status--locked');
  } else {
    statusEl.textContent = '● 未连接';
    statusEl.classList.add('status--disconnected');
  }
}

async function loadCurrentPageEntries() {
  if (!currentTab?.url) return;

  const result = await sendToBackground({
    action: 'FIND_CREDENTIALS',
    url: currentTab.url,
  });

  const container = $('current-entries');
  const noMatches = $('no-matches');

  if (result?.error || !result?.entries?.length) {
    container.innerHTML = '';
    noMatches.style.display = 'block';
    return;
  }

  noMatches.style.display = 'none';
  container.innerHTML = result.entries.map(entry => renderEntryCard(entry)).join('');
  bindEntryActions(container);
}

async function handleSearch() {
  const query = $('search-input').value.trim();
  const section = $('section-search');
  const currentSection = $('section-current');

  if (!query) {
    section.style.display = 'none';
    currentSection.style.display = 'block';
    return;
  }

  section.style.display = 'block';
  currentSection.style.display = 'none';

  const result = await sendToBackground({ action: 'SEARCH', query });
  const container = $('search-results');
  const noResults = $('no-results');

  if (result?.error || !result?.entries?.length) {
    container.innerHTML = '';
    noResults.style.display = 'block';
    return;
  }

  noResults.style.display = 'none';
  container.innerHTML = result.entries.map(entry => renderEntryCard(entry)).join('');
  bindEntryActions(container);
}

function renderEntryCard(entry) {
  return `
    <div class="entry-card" data-entry-id="${entry.id}" data-username="${escapeHtml(entry.username || '')}">
      <div class="entry-title">${escapeHtml(entry.title)}</div>
      <div class="entry-username">${escapeHtml(entry.username || '')}</div>
      <div class="entry-actions">
        <button class="btn btn--small btn--fill" data-action="fill">填充</button>
        <button class="btn btn--small btn--copy" data-action="copy">复制密码</button>
      </div>
    </div>
  `;
}

function bindEntryActions(container) {
  container.querySelectorAll('.entry-card').forEach(card => {
    const entryId = card.dataset.entryId;

    card.querySelector('[data-action="fill"]')?.addEventListener('click', async () => {
      // 向当前 tab 的 content script 发送填充请求
      if (currentTab?.id) {
        chrome.tabs.sendMessage(currentTab.id, {
          type: 'KV_FILL_DIRECT',
          entryId,
        });
        window.close();
      }
    });

    card.querySelector('[data-action="copy"]')?.addEventListener('click', async () => {
      const result = await sendToBackground({ action: 'FILL_CREDENTIAL', entryId });
      if (result?.password) {
        await navigator.clipboard.writeText(result.password);
        // 显示复制成功反馈
        const btn = card.querySelector('[data-action="copy"]');
        btn.textContent = '已复制';
        setTimeout(() => { btn.textContent = '复制密码'; }, 2000);
      }
    });
  });
}

function sendToBackground(message) {
  return chrome.runtime.sendMessage(message);
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

init();
```

- [ ] **Step 2: Commit**

```bash
git add extension/popup/popup.js
git commit -m "feat(extension): add popup JS with status, search, fill actions"
```

---

### Task 7: Rust — 扩展专用 Tauri 命令

**Files:**
- Create: `src-tauri/src/commands/native_ext.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: 创建 native_ext.rs**

```rust
use serde::Serialize;
use tauri::State;

use crate::crypto::cipher;
use crate::db::queries;
use crate::state::AppState;

#[derive(Serialize)]
pub struct CredentialMatch {
    pub id: String,
    pub title: String,
    pub username: Option<String>,
}

#[derive(Serialize)]
pub struct FillResult {
    pub username: Option<String>,
    pub password: String,
}

/// 按 URL 查找匹配凭据（不含密码）
pub async fn find_credentials_by_url(
    session_token: &str,
    url: &str,
    state: &AppState,
) -> Result<Vec<CredentialMatch>, String> {
    if !state.sessions.validate(session_token).await {
        return Err("会话已过期".to_string());
    }

    // 提取 hostname
    let hostname = extract_hostname(url).ok_or("无效的 URL")?;

    // 先按 subtitle 匹配
    let entries = queries::list_entries(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let mut matches: Vec<CredentialMatch> = Vec::new();

    for entry in &entries {
        let subtitle_match = entry
            .subtitle
            .as_ref()
            .map(|s| s.contains(&hostname))
            .unwrap_or(false);

        if subtitle_match {
            // 获取 username 字段
            let username = get_username_for_entry(&state, &entry.id).await;
            matches.push(CredentialMatch {
                id: entry.id.clone(),
                title: entry.title.clone(),
                username,
            });
        }
    }

    // 如果 subtitle 没匹配到，尝试解密 url 字段匹配
    if matches.is_empty() {
        let key_guard = state.encryption_key.read().await;
        if let Some(key) = key_guard.as_ref() {
            for entry in &entries {
                let fields = queries::get_entry_fields(&state.db, &entry.id)
                    .await
                    .unwrap_or_default();

                for field in &fields {
                    if field.field_key == "url" {
                        if let Ok(decrypted) = cipher::decrypt_field(key, &field.enc_value) {
                            if let Ok(url_val) = String::from_utf8(decrypted.to_vec()) {
                                if url_val.contains(&hostname) {
                                    let username = get_username_for_entry(&state, &entry.id).await;
                                    matches.push(CredentialMatch {
                                        id: entry.id.clone(),
                                        title: entry.title.clone(),
                                        username,
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(matches)
}

/// 获取条目的 username 字段值
async fn get_username_for_entry(state: &AppState, entry_id: &str) -> Option<String> {
    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref()?;

    let fields = queries::get_entry_fields(&state.db, entry_id)
        .await
        .ok()?;

    for field in &fields {
        if field.field_key == "username" {
            if let Ok(decrypted) = cipher::decrypt_field(key, &field.enc_value) {
                return String::from_utf8(decrypted.to_vec()).ok();
            }
        }
    }

    None
}

/// 获取条目的完整凭据（用于填充）
pub async fn get_entry_for_fill(
    session_token: &str,
    entry_id: &str,
    state: &AppState,
) -> Result<FillResult, String> {
    if !state.sessions.validate(session_token).await {
        return Err("会话已过期".to_string());
    }

    let key_guard = state.encryption_key.read().await;
    let key = key_guard.as_ref().ok_or("密码管理器已锁定")?;

    let fields = queries::get_entry_fields(&state.db, entry_id)
        .await
        .map_err(|e| e.to_string())?;

    let mut username: Option<String> = None;
    let mut password: Option<String> = None;

    for field in &fields {
        let decrypted = cipher::decrypt_field(key, &field.enc_value)
            .map_err(|e| e.to_string())?;
        let value = String::from_utf8(decrypted.to_vec())
            .map_err(|_| "解码失败".to_string())?;

        match field.field_key.as_str() {
            "username" => username = Some(value),
            "password" => password = Some(value),
            _ => {}
        }
    }

    Ok(FillResult {
        username,
        password: password.ok_or("未找到密码字段")?,
    })
}

/// 从 URL 提取 hostname
fn extract_hostname(url: &str) -> Option<String> {
    // 简单提取：去掉协议和路径
    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    let hostname = without_protocol.split('/').next()?;
    // 去掉端口
    let hostname = hostname.split(':').next()?;
    // 去掉 www. 前缀
    let hostname = hostname.strip_prefix("www.").unwrap_or(hostname);

    Some(hostname.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hostname() {
        assert_eq!(extract_hostname("https://github.com/login"), Some("github.com".to_string()));
        assert_eq!(extract_hostname("http://www.google.com:8080/path"), Some("google.com".to_string()));
        assert_eq!(extract_hostname("https://app.example.com"), Some("app.example.com".to_string()));
    }
}
```

- [ ] **Step 2: 更新 commands/mod.rs**

```rust
pub mod auth;
pub mod breach;
pub mod generator;
pub mod native_ext;
pub mod vault;
```

- [ ] **Step 3: 运行测试验证**

```bash
cd src-tauri && cargo test native_ext
```

Expected: `test_extract_hostname` passes

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/native_ext.rs src-tauri/src/commands/mod.rs
git commit -m "feat(extension): add native_ext commands for credential matching"
```

---

### Task 8: Rust — Native Messaging Host 主循环

**Files:**
- Create: `src-tauri/src/native_messaging/mod.rs`
- Create: `src-tauri/src/native_messaging/host.rs`

- [ ] **Step 1: 创建 mod.rs**

```rust
pub mod host;
```

- [ ] **Step 2: 创建 host.rs**

```rust
use std::io::{self, Read, Write};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::commands::native_ext;
use crate::crypto;
use crate::db;
use crate::state::AppState;
use tokio::sync::RwLock;

#[derive(Deserialize)]
struct Request {
    #[serde(rename = "requestId")]
    request_id: u32,
    action: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(rename = "entryId", default)]
    entry_id: Option<String>,
    #[serde(default)]
    query: Option<String>,
}

#[derive(Serialize)]
struct Response {
    #[serde(rename = "requestId")]
    request_id: u32,
    #[serde(flatten)]
    data: ResponseData,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ResponseData {
    Entries { entries: Vec<serde_json::Value> },
    Fill { username: Option<String>, password: String },
    Status { status: String },
    Error { error: String },
}

/// Native Messaging 主入口
pub fn run() {
    let rt = Runtime::new().expect("无法创建 tokio 运行时");

    // 初始化数据库
    let db_path = get_db_path();
    let pool = rt.block_on(async {
        db::init_db(&db_path)
            .await
            .expect("无法初始化数据库")
    });

    let state = AppState {
        encryption_key: RwLock::new(None),
        db: pool,
        sessions: crypto::session::SessionManager::new(),
        kdf_salt: RwLock::new(None),
    };

    // Native Messaging Host 自身的 session token
    // 由主进程管理，连接即认证
    let host_session_token = rt.block_on(state.sessions.create());

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    loop {
        // 读取 4 字节长度头
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).is_err() {
            break; // stdin 关闭
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        if len > 1024 * 1024 {
            break; // 消息过大
        }

        // 读取 JSON payload
        let mut payload = vec![0u8; len];
        if reader.read_exact(&mut payload).is_err() {
            break;
        }

        // 解析请求
        let request: Request = match serde_json::from_slice(&payload) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // 处理请求（使用 Host 自身的 session token）
        let response_data = rt.block_on(handle_request(&request, &state, &host_session_token));

        // 构建响应
        let response = Response {
            request_id: request.request_id,
            data: response_data,
        };

        // 写入响应
        let response_json = serde_json::to_vec(&response).unwrap();
        let len = (response_json.len() as u32).to_le_bytes();
        let _ = writer.write_all(&len);
        let _ = writer.write_all(&response_json);
        let _ = writer.flush();
    }
}

async fn handle_request(request: Request, state: &AppState, host_token: &str) -> ResponseData {
    let token = host_token;

    match request.action.as_str() {
        "get_status" => {
            if state.encryption_key.read().await.is_some() {
                ResponseData::Status { status: "connected".to_string() }
            } else {
                ResponseData::Status { status: "locked".to_string() }
            }
        }

        "find_credentials" => {
            let url = match &request.url {
                Some(u) => u.clone(),
                None => return ResponseData::Error { error: "缺少 url 参数".to_string() },
            };

            match native_ext::find_credentials_by_url(token, &url, state).await {
                Ok(entries) => {
                    let entries_json: Vec<serde_json::Value> = entries
                        .iter()
                        .map(|e| serde_json::to_value(e).unwrap())
                        .collect();
                    ResponseData::Entries { entries: entries_json }
                }
                Err(e) => ResponseData::Error { error: e },
            }
        }

        "get_entry_for_fill" => {
            let entry_id = match &request.entry_id {
                Some(id) => id.clone(),
                None => return ResponseData::Error { error: "缺少 entryId 参数".to_string() },
            };

            match native_ext::get_entry_for_fill(token, &entry_id, state).await {
                Ok(result) => ResponseData::Fill {
                    username: result.username,
                    password: result.password,
                },
                Err(e) => ResponseData::Error { error: e },
            }
        }

        "search" => {
            let query = match &request.query {
                Some(q) => q.clone(),
                None => return ResponseData::Error { error: "缺少 query 参数".to_string() },
            };

            // 需要验证 session
            if !state.sessions.validate(token).await {
                return ResponseData::Error { error: "会话已过期".to_string() };
            }

            match crate::db::queries::search_entries(&state.db, &query).await {
                Ok(entries) => {
                    let entries_json: Vec<serde_json::Value> = entries
                        .iter()
                        .map(|e| {
                            serde_json::json!({
                                "id": e.id,
                                "title": e.title,
                                "username": null::<String>,
                            })
                        })
                        .collect();
                    ResponseData::Entries { entries: entries_json }
                }
                Err(e) => ResponseData::Error { error: e.to_string() },
            }
        }

        _ => ResponseData::Error { error: format!("未知操作: {}", request.action) },
    }
}

fn get_db_path() -> String {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("keyvault");

    std::fs::create_dir_all(&data_dir).ok();

    let db_path = data_dir.join("keyvault.db");
    format!("sqlite:{}?mode=rwc", db_path.display())
}
```

- [ ] **Step 3: 运行编译检查**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过（可能有 unused 警告）

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/native_messaging/
git commit -m "feat(extension): add Native Messaging Host with stdin/stdout loop"
```

---

### Task 9: Rust — 更新 main.rs 支持双模式启动

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 添加 dirs 依赖到 Cargo.toml**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：
```toml
dirs = "6"
```

- [ ] **Step 2: 更新 lib.rs 添加 native messaging 模块和入口**

在 `lib.rs` 顶部添加模块声明：

```rust
mod native_messaging;
```

在文件末尾添加函数：

```rust
pub fn run_native_messaging() {
    native_messaging::host::run();
}
```

- [ ] **Step 3: 更新 main.rs 支持双模式**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|a| a == "--native-messaging") {
        keyvault_lib::run_native_messaging();
    } else {
        keyvault_lib::run();
    }
}
```

- [ ] **Step 4: 编译验证**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/main.rs src-tauri/src/lib.rs
git commit -m "feat(extension): add dual-mode startup (GUI / Native Messaging)"
```

---

### Task 10: Host 注册脚本与 Manifest

**Files:**
- Create: `scripts/register-host.bat`
- Create: `extension/com.keyvault.app.json`

- [ ] **Step 1: 创建 Host Manifest**

```json
{
  "name": "com.keyvault.app",
  "description": "KeyVault Native Messaging Host",
  "path": "",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://PLACEHOLDER_EXTENSION_ID/"]
}
```

> 注：`path` 和 `allowed_origins` 需要在安装时动态填充。

- [ ] **Step 2: 创建注册脚本**

```batch
@echo off
REM 注册 KeyVault Native Messaging Host for Chrome
REM 以管理员权限运行

set HOST_MANIFEST=%~dp0..\extension\com.keyvault.app.json
set EXE_PATH=%~dp0..\src-tauri\target\release\keyvault.exe

REM 更新 manifest 中的 path
powershell -Command "(Get-Content '%HOST_MANIFEST%') -replace '\"path\": \"\"', '\"path\": \"%EXE_PATH:\\=\\\\%\"' | Set-Content '%HOST_MANIFEST%'"

REM 注册到 Chrome
reg add "HKCU\SOFTWARE\Google\Chrome\NativeMessagingHosts\com.keyvault.app" /ve /t REG_SZ /d "%HOST_MANIFEST%" /f

REM 注册到 Edge
reg add "HKCU\SOFTWARE\Microsoft\Edge\NativeMessagingHosts\com.keyvault.app" /ve /t REG_SZ /d "%HOST_MANIFEST%" /f

echo Native Messaging Host 注册完成
pause
```

- [ ] **Step 3: Commit**

```bash
git add scripts/register-host.bat extension/com.keyvault.app.json
git commit -m "feat(extension): add host registration script and manifest"
```

---

### Task 11: 集成测试 — 手动验证

- [ ] **Step 1: 编译 Tauri 应用**

```bash
cd src-tauri && cargo build
```

- [ ] **Step 2: 注册 Native Messaging Host**

```bash
scripts/register-host.bat
```

- [ ] **Step 3: 加载扩展到 Chrome**

1. 打开 `chrome://extensions/`
2. 开启"开发者模式"
3. 点击"加载已解压的扩展程序"
4. 选择 `extension/` 目录

- [ ] **Step 4: 测试流程**

1. 启动 KeyVault 应用，创建/解锁密码库
2. 创建一个 login 类型条目（title: "GitHub", username: "test", password: "pass123", url: "github.com"）
3. 打开 `https://github.com/login`
4. 验证：password input 旁出现 KV 图标
5. 点击图标 → 验证 username 和 password 被填充

- [ ] **Step 5: 测试 Popup**

1. 点击扩展图标
2. 验证显示"已连接"状态
3. 验证当前页匹配凭据显示
4. 测试搜索功能

- [ ] **Step 6: 测试断连状态**

1. 关闭 KeyVault 应用
2. 验证扩展图标变灰/显示"未连接"
3. 点击图标 → 显示"请先启动 KeyVault 应用"
