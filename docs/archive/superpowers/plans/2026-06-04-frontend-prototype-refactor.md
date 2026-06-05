# 前端原型对齐重构 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 Vue 3 桌面端界面逐屏对齐 `docs/软件界面原型/` 中的 canonical 原型，统一壳层与设计令牌。

**Architecture:** 壳层优先（`src/components/shell/`）→ 按 README 屏幕顺序重构 Views/Components → 每屏对照 `screen.png` 验收。样式仅用 `tokens.css` CSS 变量；图标统一 Material Symbols。

**Tech Stack:** Vue 3, TypeScript, Pinia, Vite 6, Tauri 2.0, Material Symbols Outlined

**Design spec:** `docs/superpowers/specs/2026-06-04-frontend-prototype-refactor-design.md`

**Canonical refs:** `docs/软件界面原型/UNIFIED-SPEC.md`, `README.md`, `REQUIREMENTS-COVERAGE.md`

---

## 重构进度（截至 2026-06-04）

| 项 | 值 |
|----|-----|
| **执行分支** | `master`（已合并 `feature/frontend-prototype-refactor`） |
| **远程** | https://github.com/ismoyuai/keyvault-V3 |
| **执行方式** | Subagent-Driven + **executing-plans** 收尾 |
| **type-check** | ✅ |
| **cargo test** | ✅ 25 passed |
| **整体完成度** | **代码 100%** · **目视 sign-off 待人工**（见 [SIGNOFF 清单](./2026-06-04-frontend-prototype-refactor-SIGNOFF.md)） |

> **说明:** Cursor `/execute-plan` 已弃用，请改用 Superpowers `executing-plans` 技能。

### Phase 总览

| Phase | 范围 | 状态 | 备注 |
|-------|------|------|------|
| 前置 | Git worktree | ✅ | `.worktrees/frontend-prototype-refactor` |
| 0 | 令牌 + KvIcon + Shell | ✅ | `KvTopBar` / `KvSideNav` / `KvAppLayout` / `KvTransactionalLayout` |
| 1 | SetupView | ✅ | `KvTransactionalLayout` + 2 步向导 |
| 2 | UnlockView | ✅ | 锁定态 UI；`auth.ts` 解析失败/锁定文案 |
| 3.1 | VaultView 壳层 | ✅ | `KvAppLayout`；侧栏 UNIFIED 导航 |
| 3.2 | VaultItem | ✅ | 40×40 图标 + KvIcon |
| 3.3 | VaultEmptyState | ✅ | 三场景 empty variant |
| 3.4 | ItemDetail | ✅ | Secure Field + KvIcon 操作 |
| 3.5 | ItemForm + 类型选择 | ✅ | `EntryTypePicker`；card 隐藏；560px blur |
| 3.6 | 删除确认 | ✅ | `DeleteConfirmModal` |
| 3.7 | CommandPalette | ✅ | 600px blur；条目 + 常用命令分组 |
| 4 | 生成器 + 剪贴板 | ✅ | 四段强度条；ClipboardTimer 胶囊 |
| 5 | 设置 | ✅ | `KvSettingsNav` + `ChangePasswordModal` |
| 6 | 回收站 | ✅ | soft-delete + 恢复/永久删除/清空；30 天自动清理 |
| 7 | 清理验收 | ✅ | Lucide 已移除；tokens `#58a6ff`；type-check + cargo test PASS |

### 新增/主要变更文件（worktree）

```
src/components/shell/          # KvTopBar, KvSideNav, KvAppLayout, KvTransactionalLayout, KvSettingsNav
src/components/icons/KvIcon.vue
src/styles/material-symbols.css
src/constants/entryTypeIcons.ts
src/components/vault/EntryTypePicker.vue
src/components/vault/DeleteConfirmModal.vue
src/components/vault/VaultEmptyState.vue
src/components/vault/VaultTrashBanner.vue
src/components/settings/ChangePasswordModal.vue
src/views/SetupView.vue, UnlockView.vue, VaultView.vue, SettingsView.vue
src/stores/auth.ts             # 解锁失败/锁定状态
```

### 已知缺口 / 技术债

1. **解锁锁定倒计时**：已通过 `get_unlock_status` 与 Unlock 页挂载同步（应用重启后后端计数仍会重置）。
3. **笔记布局变体 B**（`screens/vault/note-layout-b` 320px 列表）：未单独实现，笔记走标准三栏。
4. **目视验收**：未逐屏对比 `screen.png`（建议本地 `npm run tauri dev` 人工签字）。

### 下一步（仅人工 / v1 外）

1. 填写 [目视验收清单](./2026-06-04-frontend-prototype-refactor-SIGNOFF.md)
2. **笔记布局变体 B**（`screens/vault/note-layout-b`）— 延后，需新计划
3. **worktree 清理**（可选）：`git worktree remove .worktrees/frontend-prototype-refactor`

---

## 前置：隔离工作区

> REQUIRED SUB-SKILL: `using-git-worktrees` — 在 `feature/frontend-prototype-refactor` 分支执行本计划。

- [x] **Worktree 已创建** — `d:\keyvault-V3\.worktrees\frontend-prototype-refactor`
- [x] **基线验证** — `npm run type-check`、`cargo test` 通过
- [x] **合并回 master** — 2026-06-04（`git merge feature/frontend-prototype-refactor`）

```powershell
# 若 .worktrees 未在 .gitignore，先添加并提交
git worktree add .worktrees/frontend-prototype-refactor -b feature/frontend-prototype-refactor
cd .worktrees/frontend-prototype-refactor
npm install
cd src-tauri; cargo test
cd ..; npm run type-check
```

---

## Phase 0: 设计系统基础 ✅

> **状态：** 已完成（worktree，2026-06-04）

### Task 0.1: 更新布局令牌

**Files:**
- Modify: `src/design/tokens.css`

- [ ] **Step 1: 更新布局尺寸**

将以下变量改为 UNIFIED-SPEC 值：

```css
--titlebar-height: 40px;
--sidebar-width: 260px;
--detail-panel-width: 400px;
--list-item-height: 48px;
```

- [ ] **Step 2: 对齐主色**

```css
--accent-blue: #58a6ff;
--text-accent: #79c0ff;
```

- [ ] **Step 3: 添加语义别名（可选，便于迁移）**

```css
--color-primary-container: var(--accent-blue);
--color-outline-variant: #30363d;
--color-surface-container: var(--bg-elevated);
```

- [ ] **Step 4: 验证**

Run: `npm run type-check`  
Expected: 无错误

---

### Task 0.2: Material Symbols + KvIcon

**Files:**
- Create: `src/styles/material-symbols.css`
- Create: `src/components/icons/KvIcon.vue`
- Modify: `src/main.ts`

- [ ] **Step 1: 创建 material-symbols.css**

```css
@import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0');
.material-symbols-outlined {
  font-family: 'Material Symbols Outlined';
  font-weight: normal;
  font-style: normal;
  font-size: 24px;
  line-height: 1;
  letter-spacing: normal;
  display: inline-block;
  white-space: nowrap;
  direction: ltr;
  -webkit-font-smoothing: antialiased;
}
.material-symbols-outlined.fill {
  font-variation-settings: 'FILL' 1;
}
```

- [ ] **Step 2: 创建 KvIcon.vue**

```vue
<script setup lang="ts">
withDefaults(defineProps<{
  name: string
  size?: number
  fill?: boolean
}>(), { size: 20, fill: false })
</script>
<template>
  <span
    class="material-symbols-outlined"
    :class="{ fill }"
    :style="{ fontSize: `${size}px` }"
  >{{ name }}</span>
</template>
```

- [ ] **Step 3: 在 main.ts 引入**

`import '@/styles/material-symbols.css'`

- [ ] **Step 4: 验证**

Run: `npm run type-check`  
Expected: PASS

---

### Task 0.3: 应用壳层组件

**Files:**
- Create: `src/components/shell/KvTopBar.vue`
- Create: `src/components/shell/KvSideNav.vue`
- Create: `src/components/shell/KvAppLayout.vue`
- Create: `src/components/shell/KvTransactionalLayout.vue`

**对照原型:** `docs/软件界面原型/screens/vault/main/code.html`（TopBar + SideNav 结构）

- [ ] **Step 1: KvTopBar.vue**

- 高度 `var(--titlebar-height)`，背景 `var(--bg-base)`
- 左：`KvIcon name="shield_lock" fill` + 「KeyVault」
- 右：最小化 / 最大化 / 关闭 → 调用 `bridge/tauri.ts` 的 `window.minimize` 等（已存在）
- `data-tauri-drag-region` 于中间拖拽区

- [ ] **Step 2: KvSideNav.vue**

Props: `activeNav`, `onNewEntry`, `onNavigate`, `onSettings`, `onLock`

结构（UNIFIED-SPEC §2.4）：
1. Vault 选择器（Local Vault，不可切换）
2. CTA「新建条目」
3. 导航项：全部 / 收藏 / 密码 / API 密钥 / 安全笔记 / 回收站
4. 底部：设置 + 锁定（图标+文字）

选中态：`border-left: 2px solid var(--accent-blue)` + `background: var(--bg-elevated)`

- [ ] **Step 3: KvAppLayout.vue**

```vue
<template>
  <div class="app-layout">
    <KvTopBar />
    <div class="app-body">
      <KvSideNav ... />
      <slot />
    </div>
  </div>
</template>
```

- [ ] **Step 4: KvTransactionalLayout.vue**

TopBar + 居中 slot（max-width 480px–560px），无侧栏

- [ ] **Step 5: 验证**

Run: `npm run type-check`

---

## Phase 1: Setup（变体 D） ✅

> **状态：** 已完成

### Task 1.1: SetupView 对齐 setup_1 + setup_2

**Files:**
- Modify: `src/views/SetupView.vue`

**对照:** `docs/软件界面原型/screens/account/setup-step1-password/screen.png`, `screens/account/setup-step2-confirm/screen.png`

- [ ] **Step 1: 改用 KvTransactionalLayout**

- [ ] **Step 2: Step 1 UI**

- 进度「第 1 步，共 2 步」
- 主密码输入 + `PasswordStrength`（若已有）
- 按钮「继续」

- [ ] **Step 3: Step 2 UI**

- 确认密码 + 匹配提示
- 按钮「创建密码库」

- [ ] **Step 4: 样式仅用 tokens，中文文案**

- [ ] **Step 5: 手动验证**

Run: `npm run tauri dev`  
Expected: 两步向导 → 进入 Vault

- [ ] **Step 6: Commit**

```bash
git add src/views/SetupView.vue src/components/shell/
git commit -m "feat(ui): align SetupView with screens/account/setup-step1-password/setup_2 prototypes"
```

---

## Phase 2: Unlock（变体 E） ✅

> **状态：** 已完成（锁定倒计时为前端估算，见进度表技术债 #4）

### Task 2.1: UnlockView 对齐 unlock_lockout

**Files:**
- Modify: `src/views/UnlockView.vue`

**对照:** `docs/软件界面原型/screens/account/unlock-lockout/screen.png`

- [ ] **Step 1: KvTransactionalLayout + vpn_key TopBar 变体（可选 prop）**

- [ ] **Step 2: 居中卡片 max-width 480px**

- [ ] **Step 3: 失败锁定 UI**

- 显示剩余尝试次数 / 锁定倒计时（对接 `auth` store 已有字段）

- [ ] **Step 4: 错误态抖动动画保留**

- [ ] **Step 5: 验证** — 正确/错误密码、锁定态

- [ ] **Step 6: Commit**

```bash
git commit -m "feat(ui): align UnlockView with unlock_lockout prototype"
```

---

## Phase 3: Vault 主界面（变体 A） ✅

> **状态：** 3.1–3.7 已完成；笔记变体 B 未做

### Task 3.1: VaultView 接入壳层

**Files:**
- Modify: `src/views/VaultView.vue`
- Create: `src/components/vault/VaultListPane.vue`（从 VaultView 拆出列表区，可选）

**对照:** `docs/软件界面原型/screens/vault/main/screen.png`

- [ ] **Step 1: 根节点改用 KvAppLayout**

- [ ] **Step 2: 删除 View 内联 titlebar/sidebar 样式**

- [ ] **Step 3: SideNav activeNav 与 `activeView` 同步**

- [ ] **Step 4: 列表 + 详情宽度使用 tokens**

- [ ] **Step 5: type-check + tauri dev 冒烟**

- [ ] **Step 6: Commit**

---

### Task 3.2: VaultItem 列表行

**Files:**
- Modify: `src/components/vault/VaultItem.vue`

**对照:** UNIFIED-SPEC §3.1

- [ ] **Step 1: 图标容器 40×40，圆角，边框 `var(--border-default)`**

- [ ] **Step 2: 选中 `#21262D` 等价 `var(--bg-elevated)` + 左边框 2px accent**

- [ ] **Step 3: 替换 emoji 为 KvIcon（按 entryType 映射）**

- [ ] **Step 4: Commit**

---

### Task 3.3: 空状态 empty_states

**Files:**
- Create: `src/components/vault/VaultEmptyState.vue`
- Modify: `src/views/VaultView.vue`

**对照:** `docs/软件界面原型/screens/vault/empty-states/screen.png`

- [ ] **Step 1: 三场景 props：`no-items` | `no-search` | `trash-empty`**

- [ ] **Step 2: CTA 新建条目绑定 `formOpen`**

- [ ] **Step 3: Commit**

---

### Task 3.4: ItemDetail 对齐 entry_detail_generic

**Files:**
- Modify: `src/components/vault/ItemDetail.vue`

**对照:** `docs/软件界面原型/screens/vault/entry-detail-ssh/screen.png`, `screens/vault/entry-detail-api-key/screen.png`

- [ ] **Step 1: Secure Field 容器样式（§3.2）**

- [ ] **Step 2: copy / visibility / open_in_new 按钮 hover 显示**

- [ ] **Step 3: 敏感字段 `font-mono` + tracking**

- [ ] **Step 4: onUnmounted 清空 secrets（保持）**

- [ ] **Step 5: Commit**

---

### Task 3.5: ItemForm + 类型选择器

**Files:**
- Create: `src/components/vault/EntryTypePicker.vue`
- Modify: `src/components/vault/ItemForm.vue`

**对照:** `screens/modals/new-entry/screen.png`（Step A 类型网格）

- [ ] **Step 1: EntryTypePicker — 从 `templates.ts` 渲染，隐藏 `card`**

- [ ] **Step 2: ItemForm 两步：选类型 → 动态字段表单**

- [ ] **Step 3: 模态 max-width 560px，遮罩 blur（§2.5 变体 F）**

- [ ] **Step 4: Commit**

---

### Task 3.6: 删除确认 modal_delete

**Files:**
- Modify: `src/components/ui/KvModal.vue`（如需）
- Modify: `src/views/VaultView.vue` 或 `ItemDetail.vue`

**对照:** `docs/软件界面原型/screens/modals/delete-confirm/screen.png`

- [ ] **Step 1: 删除前弹窗，输入标题确认或双按钮确认**

- [ ] **Step 2: 宽度 ~420px**

- [ ] **Step 3: Commit**

---

### Task 3.7: CommandPalette 对齐 keyvault_3 overlay

**Files:**
- Modify: `src/components/search/CommandPalette.vue`

- [ ] **Step 1: max-width 600px，backdrop blur**

- [ ] **Step 2: 分组：最近条目 | 常用命令（中文）**

- [ ] **Step 3: Commit**

---

## Phase 4: 模态与工具组件 ✅

> **状态：** 已完成

### Task 4.1: PasswordGenerator — keyvault_4

**Files:**
- Modify: `src/components/generator/PasswordGenerator.vue`

- [ ] **Step 1: 对齐 480px 模态布局**

- [ ] **Step 2: 强度条四段色（§3.3）**

- [ ] **Step 3: Commit**

---

### Task 4.2: ClipboardTimer — component_clipboard_timer

**Files:**
- Modify: `src/components/security/ClipboardTimer.vue`

- [ ] **Step 1: 对照原型调整位置与倒计时样式**

- [ ] **Step 2: Commit**

---

## Phase 5: 设置（变体 C） ✅

> **状态：** 已完成

### Task 5.1: SettingsView + KvSettingsNav

**Files:**
- Create: `src/components/shell/KvSettingsNav.vue`
- Modify: `src/views/SettingsView.vue`

**对照:** `docs/软件界面原型/screens/settings/main/screen.png`（及现有 HTML）

- [ ] **Step 1: 双导航布局：Side 260 + SettingsNav 192 + Content**

- [ ] **Step 2: 子 Tab：常规 | 安全 | 数据 | 关于**

- [ ] **Step 3: 安全 Tab — HIBP 开关、剪贴板超时（对接 settings store）**

- [ ] **Step 4: 数据 Tab — 导入/导出按钮（已有 bridge）**

- [ ] **Step 5: Commit**

---

### Task 5.2: ChangePasswordModal — modal_change_password

**Files:**
- Create: `src/components/settings/ChangePasswordModal.vue`
- Modify: `src/views/SettingsView.vue`

- [ ] **Step 1: 从 Settings 内联逻辑抽出**

- [ ] **Step 2: 对齐原型字段与按钮文案**

- [ ] **Step 3: Commit**

---

## Phase 6: 回收站（Should） ✅

> **状态：** UI stub；Rust 无 `soft-delete` / `deleted_at`（已 grep 确认）

### Task 6.1: Trash 视图 — vault_trash

**Files:**
- Modify: `src/views/VaultView.vue`
- Create: `src/components/vault/VaultTrashBanner.vue`
- Modify: `src/components/vault/VaultEmptyState.vue`

- [x] **Step 1: 确认 Rust 无 soft-delete → blocker + `VaultTrashBanner` 说明**

- [x] **Step 2: trash 导航：标题行、计数 0、禁用搜索、关详情、空状态**

- [x] **Step 3: 布局对齐 `vault_trash`（64px 标题行 + 搜索占位）**

- [ ] **Step 4: Commit**（待用户要求）

---

## Phase 7: 清理与验收 ✅

> **状态：** Lucide 已移除；`REQUIREMENTS-COVERAGE.md` 已更新；目视 sign-off 未做

### Task 7.1: 移除 Lucide

**Files:**
- Modify: `package.json`, `src/views/VaultView.vue`

- [x] **Step 1: `rg lucide-vue-next` 清零**

- [x] **Step 2: `npm uninstall lucide-vue-next`**

- [ ] **Step 3: Commit**（待用户要求）

---

### Task 7.2: 全量验收

- [x] **Step 1:** `npm run type-check` — PASS

- [x] **Step 2:** `cd src-tauri && cargo test` — PASS（25）

- [x] **Step 3:** 代码侧 §6 已验收；逐屏目视见 [SIGNOFF 清单](./2026-06-04-frontend-prototype-refactor-SIGNOFF.md)（待人工勾选）

- [x] **Step 4:** 更新 `docs/软件界面原型/REQUIREMENTS-COVERAGE.md` 代码列

- [x] **Step 5:** `finishing-a-development-branch`（已合并 `master`）

### Task 7.3: UNIFIED-SPEC §6 代码侧验收（2026-06-04）

> 非 pixel-perfect；合并后 `master` 代码审查勾选。

- [x] TopBar 40px + `shield_lock`（`KvTopBar.vue`）
- [x] 侧栏 260px + border-l-2（`KvSideNav.vue`）
- [x] 侧栏底部设置/锁定文字链
- [x] tokens.css 变量，组件无裸 hex
- [x] 敏感字段 mono（`ItemDetail` secure fields）
- [x] 中文文案（Setup/Unlock/Vault/Settings/Modals）
- [x] 布局变体 A/C/D/E/F（笔记变体 B 延后）
- [x] 模态 blur + 宽度（560/420/480/600）
- [ ] 逐屏对照 `screen.png`（需人工 — 见 [SIGNOFF](./2026-06-04-frontend-prototype-refactor-SIGNOFF.md)）

---

## 计划执行结论（executing-plans）

| 类别 | 状态 |
|------|------|
| Phase 0–8 实现 | ✅ 完成 |
| 合并 `master` + 推送 `origin` | ✅ |
| 自动化验证 | ✅ `type-check` + `cargo test` |
| 目视 pixel-perfect | ⏳ 需产品/设计在 SIGNOFF 清单签字 |

---

## Phase 8: 回收站数据流 ✅

> **状态：** 2026-06-04 完成（`master`）

### Task 8.1: Rust soft-delete

**Files:**
- Modify: `src-tauri/src/db/mod.rs`, `schema.rs`, `queries.rs`
- Modify: `src-tauri/src/commands/vault.rs`, `lib.rs`

- [x] `entries.deleted_at` 迁移（幂等）
- [x] `delete_entry` → 软删除；`list_trash_entries` / `restore_entry` / `purge_entry` / `empty_trash`
- [x] 列表/搜索排除已删除；30 天 `purge_expired_trash`

### Task 8.2: 前端回收站

**Files:**
- Modify: `src/stores/vault.ts`, `src/views/VaultView.vue`, `ItemDetail.vue`, `DeleteConfirmModal.vue`

- [x] 回收站导航加载 trash 列表、搜索、计数、清空
- [x] 详情恢复 / 永久删除；删除确认改为「移至回收站」

### Task 8.3: 解锁锁定同步

- [x] `get_unlock_status` IPC + `auth.syncUnlockStatus`（Unlock 页刷新倒计时）

---

## 完成检查清单

- [x] TopBar 40px + shield_lock
- [x] SideNav 260px + border-l-2 选中态
- [x] 侧栏底部设置/锁定文字链
- [x] 无裸 hex（除 tokens.css）— `--accent-blue-dim` / `--shadow-glow` 已统一 `#58a6ff`
- [x] 中文界面
- [x] Setup 2 步 / Unlock 锁定 / Vault 三栏 / 设置双导航
- [x] 新建条目类型选择器（Card 隐藏）
- [x] 删除确认模态
- [x] 空状态三场景
- [x] 回收站完整流程（soft-delete + 恢复/清空）
- [x] 移除 Lucide 依赖
- [ ] 逐屏目视对比 `screen.png`（清单已生成，待签字）
- [x] 合并 `feature/frontend-prototype-refactor` → `master`
