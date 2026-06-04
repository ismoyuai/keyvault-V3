# KeyVault 前端原型对齐重构 — 设计规格

> **日期:** 2026-06-04  
> **状态:** ✅ 已批准 · **执行中**（Phase 0–5 完成，Phase 6–7 待做）  
> **实现计划:** [`docs/superpowers/plans/2026-06-04-frontend-prototype-refactor.md`](../plans/2026-06-04-frontend-prototype-refactor.md)  
> **依据:** `docs/软件界面原型/UNIFIED-SPEC.md`、`README.md`、`REQUIREMENTS-COVERAGE.md`

---

## 重构进度摘要

| 指标 | 状态 |
|------|------|
| 设计决策（方案 A 壳层优先） | ✅ 已采纳 |
| 代码实现位置 | `feature/frontend-prototype-refactor` @ `.worktrees/frontend-prototype-refactor` |
| 是否已合并 master | ❌ 否 |
| 整体完成度 | **~85%** |

### 屏幕实现状态

| # | 原型 | Vue 目标 | 状态 |
|---|------|----------|------|
| 0 | — | tokens + shell | ✅ |
| 1 | `setup_1` + `setup_2` | `SetupView.vue` | ✅ |
| 2 | `unlock_lockout` | `UnlockView.vue` | ✅ |
| 3 | `keyvault_3` | `VaultView.vue` | ✅ |
| 4 | `empty_states` | `VaultEmptyState.vue` | ✅ |
| 5 | `modal_new_entry` | `ItemForm` + `EntryTypePicker` | ✅ |
| 6 | `entry_detail_generic` | `ItemDetail.vue` | ✅ |
| 7 | `modal_delete` | `DeleteConfirmModal.vue` | ✅ |
| 8 | `keyvault_4` | `PasswordGenerator.vue` | ✅ |
| 9 | `keyvault_7` | 笔记变体 B | ⏸ 延后（标准三栏） |
| 10 | `vault_trash` | Vault trash 视图 | ⏳ UI 占位 |
| 11 | `keyvault_2` | `SettingsView.vue` | ✅ |
| 12 | `modal_change_password` | `ChangePasswordModal.vue` | ✅ |
| 13 | `component_clipboard_timer` | `ClipboardTimer.vue` | ✅ |
| 14 | `keyvault_3` overlay | `CommandPalette.vue` | ✅ |

---

## 1. 目标

将 Vue 3 桌面端界面**逐屏对齐**已入库的 canonical HTML 原型（`code.html` + `screen.png`），在**不改动 Rust 安全模型**的前提下完成 UI/UX 重构。

**成功标准：**

- 每屏通过 UNIFIED-SPEC §6 检查清单
- `npm run type-check` 通过
- `cargo tauri dev` 主流程可走通（Setup → Unlock → Vault CRUD → Settings → Lock）
- 与对应 `screen.png` 视觉一致（布局、间距、中文文案、图标语义）

---

## 2. 非目标（v1）

| 项 | 说明 |
|----|------|
| 浏览器扩展 UI | 单独计划，见 `EXTENSION-PROTOTYPE-PROMPTS.md` |
| WebDAV 同步 | 后端未实现 |
| 恢复密钥 Setup | 已归档 `_2` |
| 亮色主题 | UNIFIED-SPEC §5 #8 |
| Emergency Wipe 后端 | 仅原型，UI 可占位 disabled |
| Card 条目类型 | v1 在类型选择器中隐藏 |
| 完整 Markdown 渲染（笔记） | v1 纯文本 + 代码块样式 |

---

## 3. 方案对比

### 方案 A：壳层优先（推荐）

先实现 `KvTopBar` / `KvSideNav` / `KvAppLayout`，再按 README 顺序逐屏替换 View 内容。

| 优点 | 缺点 |
|------|------|
| 避免 4 个 View 重复改标题栏/侧栏 | 前几屏完成前整体「半新半旧」 |
| 令牌与尺寸一次对齐 | 需先投入 2–3 天基础 |

### 方案 B：按 View 纵向切片

每个 View 自包含完成（含内联壳层），最后抽取 Shell。

| 优点 | 缺点 |
|------|------|
| 单屏可快速 demo | 大量重复样式，后期抽取成本高 |

### 方案 C：Tailwind 引入

与原型 HTML 一致使用 Tailwind 语义类。

| 优点 | 缺点 |
|------|------|
| 与原型 1:1 复制快 | 与现有 `tokens.css` + scoped CSS 双轨，包体积↑ |

**决策：采用方案 A**，样式继续以 `tokens.css` + Vue scoped CSS 为主，从原型 HTML **抄布局与类名语义**，映射到 CSS 变量（禁止裸 hex）。

---

## 4. 架构

### 4.1 新增目录（实现状态）

```
src/components/shell/
├── KvTopBar.vue              ✅
├── KvSideNav.vue             ✅
├── KvAppLayout.vue           ✅
├── KvTransactionalLayout.vue ✅
└── KvSettingsNav.vue         ✅

src/components/icons/
└── KvIcon.vue                ✅

src/components/vault/
├── EntryTypePicker.vue       ✅
├── DeleteConfirmModal.vue    ✅
└── VaultEmptyState.vue       ✅

src/components/settings/
└── ChangePasswordModal.vue   ✅

src/constants/
└── entryTypeIcons.ts         ✅（VISIBLE_ENTRY_TYPES 隐藏 card）

src/styles/
└── material-symbols.css      ✅（CDN 加载，离线待自托管）
```

### 4.2 令牌对齐（与 UNIFIED-SPEC §2.1）

| 令牌 | 值 | 实现状态 |
|------|-----|----------|
| `--bg-base` | `#0D1117` | ✅ |
| `--bg-surface` | `#161B22` | ✅ |
| `--bg-elevated` | `#1C2128` | ✅ |
| `--accent-blue` | `#58A6FF` | ✅ 已写入 `tokens.css` |
| `--color-primary-container` 等别名 | 见 plan | ✅ |
| `--titlebar-height` | `40px` | ✅ |
| `--sidebar-width` | `260px` | ✅ |
| `--detail-panel-width` | `400px` | ✅ |
| `--list-item-height` | `48px` | ✅ |
| `--accent-blue-dim` / `--shadow-glow` | 旧 rgba | ⏳ 仍引用 `#388bfd`，待 Phase 7 统一 |

### 4.3 图标策略

- **实现：** Google Material Symbols Outlined（与原型一致）— ✅ `material-symbols.css` + `KvIcon.vue`
- **封装：** `KvIcon.vue` 接收 `name`、`fill`、`size` — ✅
- **迁移进度：** 主要 View/业务组件已用 KvIcon；⏳ `VaultView.vue` 仍 import `lucide-vue-next`（`Plus`/`Search`/`Wand2`），Phase 7.1 移除

### 4.4 原型对照工作流（每屏固定步骤）

1. 打开 `docs/软件界面原型/<dir>/screen.png` + `code.html`
2. 列出与当前 Vue 组件的差异清单（布局、文案、交互）
3. 改组件 + 样式，仅用 `var(--*)`
4. `npm run type-check`
5. `cargo tauri dev` 手动验证
6. 在 PR/任务备注勾选 UNIFIED-SPEC §6 项

---

## 5. 屏幕映射与布局变体

| 顺序 | 原型目录 | Vue 目标 | 布局变体 | 状态 |
|------|----------|----------|----------|------|
| 0 | — | `tokens.css`, shell | 基础 | ✅ |
| 1 | `setup_1` + `setup_2` | `SetupView.vue` | D | ✅ |
| 2 | `unlock_lockout` / `keyvault_6` | `UnlockView.vue` | E | ✅ |
| 3 | `keyvault_3` | `VaultView.vue` | A | ✅ |
| 4 | `empty_states` | `VaultView` 列表区 | A | ✅ |
| 5 | `modal_new_entry` / `keyvault_1` | `ItemForm.vue` | F | ✅ |
| 6 | `entry_detail_generic` / `api_keyvault` | `ItemDetail.vue` | A | ✅ |
| 7 | `modal_delete` | `DeleteConfirmModal` | F | ✅ |
| 8 | `keyvault_4` | `PasswordGenerator.vue` | F | ✅ |
| 9 | `keyvault_7` | Note 布局（VaultView 分支） | B | ⏸ 延后 |
| 10 | `vault_trash` | VaultView trash 视图 | A | ⏳ 导航有、数据无 |
| 11 | `keyvault_2` + 扩展 | `SettingsView.vue` | C | ✅ |
| 12 | `modal_change_password` | `ChangePasswordModal.vue` | F | ✅ |
| 13 | `component_clipboard_timer` | `ClipboardTimer.vue` | 组件 | ✅ |
| 14 | `keyvault_3` overlay | `CommandPalette.vue` | F | ✅ |

---

## 6. 后端依赖（按需）

| 功能 | 设计时状态 | 当前实现 |
|------|------------|----------|
| 回收站 | 待确认 soft-delete API | ⏳ **阻塞**：`VaultView` trash 导航已接，过滤器恒返回空列表 |
| 10 条目类型 | `templates.ts` 已有 | ✅ `EntryTypePicker` + `VISIBLE_ENTRY_TYPES`（9 类，card 隐藏） |
| HIBP / 剪贴板设置 | 命令已有 | ✅ Settings「安全」Tab 已对接 `security.checkBreach` + `settings` store |
| 解锁失败锁定 | `auth.rs` 已有 | ✅ `auth.ts` 解析错误文案 + Unlock 双态 UI（倒计时为前端估算） |

---

## 7. 测试策略

- **自动化：** 每 Phase 末 `npm run type-check`；不新增 E2E（除非项目已有）
- **手动：** 每屏对照 `screen.png`；快捷键 Ctrl+K / Ctrl+N / Ctrl+L
- **安全：** `ItemDetail` 卸载仍清空 secrets；不在 store 缓存解密字段

---

## 8. 风险

| 风险 | 缓解 |
|------|------|
| Lucide → Material 工作量大 | `KvIcon` 封装 + 分 Phase 替换 |
| VaultView 文件过大 | 拆 `VaultSidebar.vue`、`VaultListPane.vue` |
| 回收站无 API | 先 UI 后 API 或 stub |

---

## 9. 批准与收尾

- **设计：** ✅ 已批准（方案 A）
- **计划：** ✅ [`2026-06-04-frontend-prototype-refactor.md`](../plans/2026-06-04-frontend-prototype-refactor.md)
- **执行：** Subagent-Driven，Phase 0–5 已在 worktree 落地
- **待收尾：** Phase 6–7 → `finishing-a-development-branch` → 更新 [`REQUIREMENTS-COVERAGE.md`](../../软件界面原型/REQUIREMENTS-COVERAGE.md) 代码列

### UNIFIED-SPEC §6 自检（实现侧）

| 检查项 | 状态 |
|--------|------|
| TopBar 40px + shield_lock | ✅ |
| 侧栏 260px + border-l-2 选中态 | ✅ |
| 侧栏底部「设置」「锁定」文字链 | ✅ |
| 敏感字段 JetBrains Mono | ✅ |
| 中文文案 | ✅ |
| 模态 blur + 宽度（新建 560 / 删除 420 / 生成器 480 / 命令 600） | ✅ |
| 逐屏 pixel-perfect 对比 `screen.png` | ⏳ 未正式验收签字 |
