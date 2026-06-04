# KeyVault 浏览器扩展 · AI 原型提示词库

**版本:** 2026-06-04  
**用法:** 每次生成 = 复制 **§1 公共前缀** + **§2 对应分屏提示词**，粘贴到 AI 原型设计工具。  
**参考图（推荐）:** 桌面 `screens/vault/main/screen.png`、`screens/account/unlock/screen.png`（色值与密度对齐）；已有实现 `extension/popup/popup.css`。  
**入库:** 产出放入 `docs/软件界面原型/extension/<目录名>/`，含 `code.html`（若工具导出）与 `screen.png`。

## 与桌面原型的关系

| 维度 | 桌面应用 | 浏览器扩展 |
|------|----------|------------|
| 画布 | 1200×800 窗口 + 自定义 TopBar | Popup **360×540**；网页内 **浮层/注入组件** |
| 壳层 | 三栏 + 侧栏 260px | 无侧栏；Header + 列表 + Footer |
| 通信 | Tauri IPC | Native Messaging → 主应用（未启动/锁定则不可用） |
| 设计语言 | 同一套 Dark tokens | 同一套，但更紧凑 |

## 建议生成顺序

1. **P0（MVP）:** EXT-P0-1 → P0-2 → P0-3 → P0-4 → P0-5 → P0-6  
2. **P1:** EXT-P1-1 → P1-2 → P1-3 → P1-4  
3. **P2:** EXT-P2-1 → P2-2 → P2-3  
4. **Future:** 右键菜单、快捷键提示（可选）

---

## 1. 公共前缀（每条提示词前必附）

```text
请为密码管理器 KeyVault 的 Chrome/Edge 浏览器扩展设计 UI 原型（Dark Mode Only）。

## 产品
本地离线优先的密码管理器浏览器扩展（Manifest V3）。扩展本身不存储解密数据，通过 Native Messaging 与已解锁的 KeyVault 桌面应用通信。类似 Bitwarden/1Password 浏览器插件，但强调零持久化、连接断开即不可用。

## 设计令牌（必须与桌面 KeyVault 一致）
- 背景 surface #161B22，卡片/elevated #1C2128，输入框 #2D333B
- 页面底 #0D1117（仅网页 mock 背景时用）
- 边框 #30363D 或 rgba(255,255,255,0.1) 1px hairline
- 主色 #58A6FF（按钮、KV 图标、焦点环）；成功 #3FB950；警告 #D29922；错误 #F85149
- 正文 #E6EDF3，次要 #8B949E，弱化标签 #484F58
- 字体：Inter 或 system-ui（界面），JetBrains Mono（用户名/密码字段）
- 图标：Material Symbols Outlined 或简洁 emoji 占位
- 所有用户可见文案：简体中文

## 扩展 UI 类型
A. **Popup 面板** — 点击浏览器工具栏 KV 图标弹出，宽 360px，最大高 540px，无自定义窗口按钮  
B. **网页内注入** — 叠加在第三方登录页上（如 GitHub 登录），KV 品牌组件 z-index 最高  

## 输出
- 高保真 UI 原型图
- 标注主要间距与组件名称
- 不要设计浅色主题
- Popup 类请画出完整 360px 宽面板；网页内类请包含简化登录页作为背景上下文
```

---

## 2. 分屏提示词

### EXT-P0-1 · Popup · 已连接（`extension/popup_connected/`）

```text
[粘贴 §1 公共前缀]

## 本屏：扩展 Popup · 已连接 · 当前页匹配

### 类型 A — Popup 360×480px（可滚动）

### 结构
┌ Header 36px ─────────────────────────┐
│ shield_lock KeyVault          ● 已连接 │  状态点绿色 #3FB950
├ Search 36px ───────────────────────────┤
│ 🔍 搜索密码...                          │
├ Section ───────────────────────────────┤
│ 当前页面（LABEL 10px uppercase #484F58）│
│ ┌ Entry Card ×2 ─────────────────────┐ │
│ │ GitHub                               │ │
│ │ ismoyuai@gmail.com（Mono 11px 灰）   │ │
│ │              [填充] [复制密码]          │ │  按钮：填充主色，复制 ghost
│ └──────────────────────────────────────┘ │
│ ┌ Entry Card ──────────────────────────┐ │
│ │ GitHub（工作）                         │ │
│ │ work@company.com                     │ │
│ │              [填充] [复制密码]          │ │
│ └──────────────────────────────────────┘ │
├ Footer 36px ───────────────────────────┤
│ 打开应用                    生成密码     │  ghost 文字按钮
└────────────────────────────────────────┘

### 细节
- Entry 卡片：bg #1C2128，圆角 6px，padding 10px，hover #22272E
- 列表区可滚动，Footer 固定底栏
- 不要桌面三栏、不要侧栏

### 参考
extension/popup/popup.html 已连接布局
```

### EXT-P0-2 · Popup · 未连接（`extension/popup_disconnected/`）

```text
[粘贴 §1 公共前缀]

## 本屏：扩展 Popup · 未连接

### 类型 A — Popup 360×320px

### 结构
- Header：KeyVault + 状态「● 未连接」（灰色 #8B949E）
- 主区域居中 empty-state（min-height ~200px）：
  - 图标：link_off 或 🔌，32px
  - 标题：「请先启动 KeyVault 应用」
  - 说明（可选小字）：「扩展需通过 Native Messaging 连接本地应用」
  - 主按钮「打开应用」（#58A6FF）
- Footer 保留但按钮可 disabled 或仅「打开应用」可用

### 不要
搜索框、凭据列表；不要英文文案
```

### EXT-P0-3 · Popup · 已锁定（`extension/popup_locked/`）

```text
[粘贴 §1 公共前缀]

## 本屏：扩展 Popup · 已连接但保险库锁定

### 类型 A — Popup 360×320px

### 结构
- Header：KeyVault + 状态「● 已锁定」（警告色 #D29922）
- 主区域 empty-state：
  - 图标：lock，32px
  - 标题：「KeyVault 已锁定」
  - 说明：「请在桌面应用中解锁后再填充密码」
  - 主按钮「解锁」（点击唤起主窗口 — 仅暗示，不画桌面窗）
- Footer：「打开应用」ghost

### 对比
与 P0-2 同壳层，仅图标/文案/状态色不同；无搜索与列表
```

### EXT-P0-4 · Popup · 搜索模式（`extension/popup_search/`）

```text
[粘贴 §1 公共前缀]

## 本屏：扩展 Popup · 全库搜索

### 类型 A — Popup 360×540px

### 结构
- Header：● 已连接（绿）
- 搜索框：已输入「github」，左侧 search 图标，右侧 clear ×
- Section「搜索结果」（替代「当前页面」）
- 3~4 条 Entry Card：标题 + 用户名，每条仅 [填充] 或 [填充][复制密码]
- 一条结果可带小字 hostname 副标题「github.com」
- Footer 同 P0-1

### 交互暗示
搜索时隐藏「当前页面」区块；实时过滤（画出已输入态即可）
```

### EXT-P0-5 · 网页内 · 单凭据填充图标（`extension/inline_fill_single/`）

```text
[粘贴 §1 公共前缀]

## 本屏：网页登录表单 · 单匹配 · KV 填充图标

### 类型 B — 网页 mock + 注入组件

### 背景（简化第三方登录页，浅色即可反衬）
- 模拟 GitHub 登录：居中白卡片 ~400px，标题「Sign in to GitHub」
- 字段：Username or email、Password（password input 右侧留空给图标）
- Sign in 按钮

### KV 注入（关键）
- 在 Password 输入框内右侧：20×20px 圆角 4px 方块
- 背景 rgba(88,166,255,0.9)，白字「KV」10px bold
- 输入框 padding-right 加大避免文字与图标重叠
- Hover：略提亮 + tooltip「KeyVault：找到 1 个匹配凭据」

### 不要
Popup 面板；不要画出完整 GitHub 品牌侵权细节，generic 即可
```

### EXT-P0-6 · 网页内 · 多凭据下拉（`extension/inline_dropdown/`）

```text
[粘贴 §1 公共前缀]

## 本屏：网页登录表单 · 多匹配 · 凭据下拉

### 类型 B — 同 P0-5 背景 + 下拉

### KV 图标
- 同 P0-5，tooltip「找到 3 个匹配凭据」

### 下拉列表（图标正下方，右对齐）
- 宽 200px，max-height 200px，可滚动
- bg #1C2128，border 1px rgba(255,255,255,0.1)，圆角 6px，阴影
- 每项高 ~36px：主行 title 12px + 副行 username 11px #8B949E
- 示例：GitHub / ismoyuai@gmail.com；GitHub（工作）/ work@company.com；GitHub API / token-user
- 一项 hover 背景 #22272E

### 交互
画出 dropdown 展开态；Esc/点击外部关闭（文字标注即可）
```

---

### EXT-P1-1 · 网页内 · 保存密码顶栏（`extension/save_banner/`）

```text
[粘贴 §1 公共前缀]

## 本屏：登录成功后 · 保存密码提示条

### 类型 B — 网页顶部 fixed 横幅

### 布局
- position fixed，top 0，水平居中，z-index 最高
- 横幅：bg #1C2128，底圆角 8px，border + shadow
- 高度 ~40px，padding 10px 16px，flex 横向

### 内容
- 左：shield_lock +「KeyVault」
- 中：「是否保存此密码？」（#8B949E）
- 右：主按钮「保存」+ Ghost「不用」
- 可选小字：「github.com · user@example.com」（不显示明文密码）

### 背景
同 P0-5 登录页，表单可处于已提交/跳转前状态

### 说明
10 秒自动消失（标注）；非 modal，不阻断整页
```

### EXT-P1-2 · 网页内 · 填充成功反馈（`extension/inline_success/`）

```text
[粘贴 §1 公共前缀]

## 本屏：填充完成 · 成功态图标

### 类型 B — 同 P0-5 背景，Password 已填充 ••••••

### KV 图标变化
- 原 KV 方块变为成功态：背景 #3FB950，内容 white check ✓
- 3 秒后恢复 KV（标注时间线，或画两态对比小 inset）

### 表单态
- Username、Password 均有值（Password 显示圆点）
- 可选底部 toast：「已填充 GitHub」细条，与桌面 clipboard timer 风格一致但更简单
```

### EXT-P1-3 · Popup · 当前页无匹配（`extension/popup_no_match/`）

```text
[粘贴 §1 公共前缀]

## 本屏：扩展 Popup · 已连接 · 无当前页匹配

### 类型 A — Popup 360×400px

### 结构
- Header + 搜索框（同 P0-1）
- Section「当前页面」
- 空态居中：
  - icon search_off 或 link
  - 「当前页面无匹配凭据」
  - 次文案「试试搜索全库，或在 KeyVault 中添加此网站条目」
- Footer 同 P0-1

### 不要
假造当前页卡片；应明确 empty hint
```

### EXT-P1-4 · 网页内 · 不可用态图标（`extension/inline_disabled/`）

```text
[粘贴 §1 公共前缀]

## 本屏：网页内 KV 图标 · 未连接/锁定灰态

### 类型 B — 同 P0-5 背景

### 图标变体（可一图画 2 个 inset）
1. **未连接**：KV 方块 bg #484F58 40% 透明度，cursor not-allowed
   - Tooltip：「请先启动 KeyVault」
2. **已锁定**：锁 icon 替代 KV，bg #D29922 系
   - Tooltip：「KeyVault 已锁定，请先解锁」

### 说明
检测到表单但因 Native Messaging 不可用而不提供填充；与 P0-5 活跃蓝态形成对比
```

---

### EXT-P2-1 · Popup · 保存条目确认模态（`extension/save_modal/`）

```text
[粘贴 §1 公共前缀]

## 本屏：保存新凭据 · Popup 内模态

### 类型 A — Popup 360×540px，内容区 blur 遮罩

### 触发
用户点击 P1-1 保存横幅「保存」后，扩展 Popup 或独立小窗展示（优先 Popup 内模态）

### 模态内容 max-width ~320px
- 标题：「保存到 KeyVault」
- 字段：
  - 标题（预填「GitHub」可编辑）
  - 网址（预填 https://github.com/login）
  - 用户名（预填，Mono）
  - 密码（•••••• + visibility）
- 文件夹/集合下拉（可选）：「无」
- 按钮：Ghost「取消」+ 主色「保存条目」

### 不要
完整桌面新建条目 10 类型选择（保存流程默认网站密码）
```

### EXT-P2-2 · Popup · 迷你密码生成器（`extension/generator_popup/`）

```text
[粘贴 §1 公共前缀]

## 本屏：Popup Footer「生成密码」展开面板

### 类型 A — Popup 360×540px

### 布局
- 上半：同 P0-1 已连接列表（可略简化）
- Footer 点击「生成密码」后，自底部滑出面板 h ~180px，覆盖列表下缘

### 生成器面板
- 生成的密码一行：Mono 14px + copy 按钮
- 长度 slider 12–64，默认 16
- 复选：大写 / 小写 / 数字 / 符号（与桌面 keyvault_4 逻辑一致，UI 更紧凑）
- 按钮：「重新生成」「复制」

### 参考
桌面 keyvault_4 密码生成器，压缩为扩展尺寸
```

### EXT-P2-3 · 扩展图标 · 工具栏状态（`extension/toolbar_icons/`）

```text
[粘贴 §1 公共前缀]

## 本屏：浏览器工具栏扩展图标 · 多状态规格板

### 类型 C — 图标规格板（非 Popup）

### 画布
- 横排展示 16×16、48×48、128×128 三尺寸（Chrome Web Store / toolbar / management）

### 状态变体（每个尺寸一行或 grid）
1. **默认**：shield 或 KV 字母，暗底 #161B22 + 主色 #58A6FF 强调
2. **已连接**：可加小绿点 badge（可选）
3. **未连接/锁定**：灰度或锁 overlay

### 风格
简洁、小尺寸可读；与桌面 App shield_lock 视觉家族一致
- 导出标注：PNG 透明底，无圆角外框（Chrome 自裁）
```

---

### Future · 右键菜单与快捷键（`extension/context_menu/`）

```text
[粘贴 §1 公共前缀]

## 本屏：Future · 右键菜单 / 快捷键提示

### 内容（一图多态）
1. 网页输入框右键菜单底部：「KeyVault → 填充凭据 / 生成密码」
2. 快捷键 hint overlay：「Ctrl+Shift+L 填充」角标（仅在 focus 输入框时）

### 标注
Future 功能，MVP 可不实现
```

---

## 3. 入库目录与验收

| 目录 | 优先级 | 对应代码 |
|------|--------|----------|
| `extension/popup_connected/` | P0 Must | `popup/popup.html` view-connected |
| `extension/popup_disconnected/` | P0 Must | view-disconnected |
| `extension/popup_locked/` | P0 Must | view-locked |
| `extension/popup_search/` | P0 Must | section-search |
| `extension/popup_no_match/` | P1 Should | no-matches |
| `extension/inline_fill_single/` | P0 Must | `content/filler.js` 单条 |
| `extension/inline_dropdown/` | P0 Must | filler 下拉 |
| `extension/inline_success/` | P1 Should | fill 成功态 |
| `extension/inline_disabled/` | P1 Should | 连接不可用 |
| `extension/save_banner/` | P1 Should | `content/save-prompt.js` |
| `extension/save_modal/` | P2 Should | 保存流程（待实现） |
| `extension/generator_popup/` | P2 Could | Footer 生成密码（待实现） |
| `extension/toolbar_icons/` | P2 Could | `icons/*.png` |

### 验收要点

- [ ] Popup 宽度 **360px**，Header/Footer 高度与 `popup.css` 一致（~36px）
- [ ] 色值与 [UNIFIED-SPEC.md](./UNIFIED-SPEC.md) / [design-system/DESIGN.md](./design-system/DESIGN.md) 一致
- [ ] 网页内组件在 **mock 登录页** 上展示，而非孤立控件
- [ ] 全中文案；状态三色：绿已连接 / 灰未连接 / 黄锁定
- [ ] 更新 [REQUIREMENTS-COVERAGE.md](./REQUIREMENTS-COVERAGE.md) 扩展章节（若已添加）

### 与桌面原型联调

生成 Popup 类时，可同时打开 `screens/vault/main/screen.png` 作为色板参考；生成网页内浮层时，注意 KV 图标 `#58A6FF` 与桌面主色一致（代码中 `#388bfd` 为同色系，原型统一用 #58A6FF）。
