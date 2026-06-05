# Vue 前端审查修复计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复前端审查中发现的全部 5 Critical、9 Important、10 Minor 问题

**Architecture:** 按优先级分 4 阶段：阻塞运行 → 功能完整性 → 安全加固 → 质量改进。每阶段内任务相互独立，可安全按序执行。

**Tech Stack:** Vue 3 Composition API, Pinia, Tauri 2.0 IPC, @vueuse/core

---

## 文件影响清单

| 文件 | 操作 | 涉及任务 |
|---|---|---|
| `src/bridge/tauri.ts` | 修改 | T1, T3, T9 |
| `src/router/index.ts` | 修改 | T2 |
| `src/components/vault/ItemForm.vue` | 修改 | T4, T5, T13 |
| `src/views/SettingsView.vue` | 修改 | T6, T12, T13 |
| `src/composables/usePasswordGen.ts` | 修改 | T7 |
| `src/views/VaultView.vue` | 修改 | T8, T17 |
| `src/components/search/CommandPalette.vue` | 修改 | T10 |
| `src/components/ui/KvModal.vue` | 修改 | T11 |
| `src/composables/useClipboard.ts` | 修改 | T14, T15 |
| `src/composables/useAutoLock.ts` | 修改 | T18 |
| `src/composables/useShortcuts.ts` | 修改 | T19 |
| `src/main.ts` | 修改 | T20 |

---

## Phase 1: Critical — 阻塞应用运行

### Task 1: 移除 tray IPC 调用 [C-1]

**Files:**
- Modify: `src/bridge/tauri.ts:139-145`

**问题:** `tray.lock()` 和 `tray.show()` 调用未注册的 IPC 命令 `tray_lock`/`tray_show`，运行时会抛出错误。

- [ ] **Step 1: 删除 tray 对象**

在 `src/bridge/tauri.ts` 中，删除整个 `tray` 导出对象（第 139-145 行）：

```typescript
// 删除以下代码：
// ============================================
// 系统托盘
// ============================================
export const tray = {
  lock: () => invoke<void>('tray_lock'),
  show: () => invoke<void>('tray_show'),
}
```

- [ ] **Step 2: 确认无引用**

在项目中搜索 `tray` 的引用，确认没有其他文件导入或使用 `tray` 对象。如果有，一并移除。

Run: `grep -r "from.*bridge/tauri.*tray\|import.*tray.*bridge" src/`
Expected: 无匹配结果

- [ ] **Step 3: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 4: 提交**

```bash
git add src/bridge/tauri.ts
git commit -m "fix: remove unregistered tray IPC calls"
```

---

### Task 2: 实现路由守卫 [C-2]

**Files:**
- Modify: `src/router/index.ts:36-41`

**问题:** `router.beforeEach` 空实现，未解锁用户可直接访问 `/vault` 和 `/settings`。

- [ ] **Step 1: 添加 auth store 导入和真实守卫逻辑**

替换 `src/router/index.ts` 的全部内容：

```typescript
import { createRouter, createWebHashHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/login',
    },
    {
      path: '/setup',
      name: 'setup',
      component: () => import('@/views/SetupView.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/UnlockView.vue'),
    },
    {
      path: '/vault',
      name: 'vault',
      component: () => import('@/views/VaultView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      meta: { requiresAuth: true },
    },
  ],
})

router.beforeEach((to) => {
  const auth = useAuthStore()
  if (to.meta.requiresAuth && !auth.isUnlocked) {
    return { name: 'login' }
  }
})

export default router
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/router/index.ts
git commit -m "fix: implement real auth guard in router beforeEach"
```

---

### Task 3: 修复 updateEntry 类型不匹配 [C-3]

**Files:**
- Modify: `src/bridge/tauri.ts:65`

**问题:** `updateEntry` 使用 `Partial<CreateEntryInput>`，但 Rust 端期望完整对象。

- [ ] **Step 1: 修改 updateEntry 参数类型**

在 `src/bridge/tauri.ts` 第 65 行，将 `Partial<CreateEntryInput>` 改为 `CreateEntryInput`：

```typescript
// 修改前：
updateEntry: (entryId: string, input: Partial<CreateEntryInput>) =>

// 修改后：
updateEntry: (entryId: string, input: CreateEntryInput) =>
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误（如果调用方传了 Partial，需要同步修复调用方）

- [ ] **Step 3: 提交**

```bash
git add src/bridge/tauri.ts
git commit -m "fix: updateEntry param type to full CreateEntryInput"
```

---

### Task 4: 修复 ItemForm import 位置 [C-4]

**Files:**
- Modify: `src/components/vault/ItemForm.vue:1-104`

**问题:** `import { vault as vaultBridge }` 在 script setup 底部（第 104 行），应在顶部。

- [ ] **Step 1: 移动 import 到顶部**

在 `src/components/vault/ItemForm.vue` 中：

1. 删除第 104 行的 `import { vault as vaultBridge } from '@/bridge/tauri'`
2. 在 script setup 顶部的 import 区域（第 8 行之后）添加：

```typescript
import { vault as vaultBridge } from '@/bridge/tauri'
```

最终 import 区域应为：

```typescript
import { ref, computed, watch } from 'vue'
import KvButton from '@/components/ui/KvButton.vue'
import KvInput from '@/components/ui/KvInput.vue'
import KvModal from '@/components/ui/KvModal.vue'
import { TEMPLATES, TEMPLATE_LIST } from '@/constants/templates'
import { useVaultStore } from '@/stores/vault'
import { useToast } from '@/composables/useToast'
import { vault as vaultBridge } from '@/bridge/tauri'
import type { EntryType, FieldInput, CreateEntryInput, EntryMeta } from '@/types/vault'
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/components/vault/ItemForm.vue
git commit -m "fix: move vaultBridge import to top of script setup"
```

---

### Task 5: 添加 ItemForm props watch [C-5]

**Files:**
- Modify: `src/components/vault/ItemForm.vue:27-29`

**问题:** `step`、`selectedType`、`title` 等 ref 在 setup 时初始化一次，重新打开不同条目时显示旧数据。

- [ ] **Step 1: 添加 watch 导入**

在 `src/components/vault/ItemForm.vue` 第 2 行，将 `ref, computed` 改为 `ref, computed, watch`：

```typescript
import { ref, computed, watch } from 'vue'
```

- [ ] **Step 2: 添加 watch 重置逻辑**

在 `const template = computed(...)` 之后（约第 34 行后）添加：

```typescript
watch(() => props.open, (isOpen) => {
  if (isOpen) {
    if (props.editEntry) {
      step.value = 'form'
      selectedType.value = props.editEntry.entryType
      title.value = props.editEntry.title
      subtitle.value = props.editEntry.subtitle || ''
      tags.value = props.editEntry.tags.join(', ')
      // fields 需要从 editEntry 加载，但当前 EntryMeta 不含 fields
      // 所以编辑时 fields 由用户重新填写
      fields.value = []
    } else {
      resetForm()
    }
  }
})
```

- [ ] **Step 3: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 4: 提交**

```bash
git add src/components/vault/ItemForm.vue
git commit -m "fix: watch props.open to reset form state on reopen"
```

---

## Phase 2: Important — 功能完整性

### Task 6: 修复设置持久化 [I-1]

**Files:**
- Modify: `src/views/SettingsView.vue:31,43`

**问题:** `select` 的 `v-model` 直接绑定 store ref，绕过 setter 方法（`setAutoLock`/`setClipboardClear`），变更不会写入后端。

- [ ] **Step 1: 改用 @change 调用 store 方法**

在 `src/views/SettingsView.vue` 中，替换两个 select 元素：

将第 31 行的：
```html
<select v-model="settings.autoLockMinutes" class="setting-select">
```

改为：
```html
<select :value="settings.autoLockMinutes" class="setting-select" @change="settings.setAutoLock(Number(($event.target as HTMLSelectElement).value))">
```

将第 43 行的：
```html
<select v-model="settings.clipboardClearSeconds" class="setting-select">
```

改为：
```html
<select :value="settings.clipboardClearSeconds" class="setting-select" @change="settings.setClipboardClear(Number(($event.target as HTMLSelectElement).value))">
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/views/SettingsView.vue
git commit -m "fix: persist settings changes via store setter methods"
```

---

### Task 7: 修复密码生成器 modulo bias [I-3]

**Files:**
- Modify: `src/composables/usePasswordGen.ts:77,82`

**问题:** `randomBytes[i] % chars.length` 有 modulo bias，应使用 rejection sampling。

- [ ] **Step 1: 添加 rejection sampling 辅助函数**

在 `src/composables/usePasswordGen.ts` 的 `CHARSETS` 定义之后（约第 23 行后）添加：

```typescript
/**
 * 从 [0, limit) 范围内生成均匀分布的随机索引
 * 使用 rejection sampling 消除 modulo bias
 */
function secureRandomIndex(limit: number): number {
  if (limit <= 0 || limit > 256) throw new Error('limit must be 1-256')
  const maxValid = Math.floor(256 / limit) * limit
  const bytes = new Uint8Array(1)
  do {
    crypto.getRandomValues(bytes)
  } while (bytes[0] >= maxValid)
  return bytes[0] % limit
}
```

- [ ] **Step 2: 替换 generate() 中的 modulo 用法**

在 `generate()` 函数中，将第 77 行：
```typescript
pwd += chars[randomBytes[i] % chars.length]
```

改为：
```typescript
pwd += chars[secureRandomIndex(chars.length)]
```

将第 82 行：
```typescript
pwd += charset[randomBytes[i + required.length] % charset.length]
```

改为：
```typescript
pwd += charset[secureRandomIndex(charset.length)]
```

- [ ] **Step 3: 更新 Fisher-Yates 洗牌**

将第 88 行的洗牌部分：
```typescript
const j = randomBytes[len + i] % (i + 1)
```

改为：
```typescript
const j = secureRandomIndex(i + 1)
```

- [ ] **Step 4: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 5: 提交**

```bash
git add src/composables/usePasswordGen.ts
git commit -m "fix: eliminate modulo bias in password generator with rejection sampling"
```

---

### Task 8: filteredEntries 改为 computed [I-4]

**Files:**
- Modify: `src/views/VaultView.vue:82-86,177,186`

**问题:** `filteredEntries` 是普通函数，每次模板渲染重新计算。

- [ ] **Step 1: 将函数改为 computed**

在 `src/views/VaultView.vue` 中，将第 82-86 行：
```typescript
const filteredEntries = () => {
  if (activeView.value === 'favorites') return vault.entries.filter(e => e.favorited)
  if (activeView.value === 'recent') return vault.entries.slice(0, 20)
  return vault.entries
}
```

改为：
```typescript
const filteredEntries = computed(() => {
  if (activeView.value === 'favorites') return vault.entries.filter(e => e.favorited)
  if (activeView.value === 'recent') return vault.entries.slice(0, 20)
  return vault.entries
})
```

- [ ] **Step 2: 更新模板中的调用**

将模板中所有 `filteredEntries()` 改为 `filteredEntries`（去掉括号）：

第 177 行：`v-for="entry in filteredEntries()"` → `v-for="entry in filteredEntries"`
第 186 行：`v-if="!vault.isLoading && filteredEntries().length === 0"` → `v-if="!vault.isLoading && filteredEntries.length === 0"`

- [ ] **Step 3: 确认 computed 已导入**

检查第 2 行已有 `computed` 导入（当前是 `import { ref, onMounted } from 'vue'`），需要添加：

```typescript
import { ref, computed, onMounted } from 'vue'
```

- [ ] **Step 4: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 5: 提交**

```bash
git add src/views/VaultView.vue
git commit -m "perf: convert filteredEntries to computed property"
```

---

### Task 9: 添加 export/import bridge 调用 [I-9]

**Files:**
- Modify: `src/bridge/tauri.ts`

**问题:** 缺少 `export_vault`/`import_vault` 的 bridge 调用。

- [ ] **Step 1: 添加 export/import 到 vault 对象**

在 `src/bridge/tauri.ts` 的 `vault` 对象中（`searchEntries` 之后）添加：

```typescript
exportVault: (format: string) =>
  invoke<string>('export_vault', { sessionToken: getToken(), format }),

importVault: (data: string, format: string) =>
  invoke<number>('import_vault', { sessionToken: getToken(), data, format }),
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/bridge/tauri.ts
git commit -m "feat: add export/import vault bridge calls"
```

---

## Phase 3: Important — 安全加固

### Task 10: 移除 CommandPalette no-op handler [I-5]

**Files:**
- Modify: `src/components/search/CommandPalette.vue:141-154`

**问题:** `handleGlobalKeydown` 监听 Ctrl+K 但只调用 `preventDefault()`，不做任何事。

- [ ] **Step 1: 删除全局 keydown 监听**

在 `src/components/search/CommandPalette.vue` 中，删除：

1. `onMounted` 中的 `document.addEventListener('keydown', handleGlobalKeydown)`（第 142 行）
2. `onUnmounted` 中的 `document.removeEventListener('keydown', handleGlobalKeydown)`（第 146 行）
3. 整个 `handleGlobalKeydown` 函数（第 149-154 行）

如果 `onMounted`/`onUnmounted` 变为空，一并删除导入和调用。

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/components/search/CommandPalette.vue
git commit -m "fix: remove no-op global keydown handler from CommandPalette"
```

---

### Task 11: 修复 KvModal Escape 监听 [I-6]

**Files:**
- Modify: `src/components/ui/KvModal.vue:16-26`

**问题:** `handleKeydown` 在 `onMounted` 注册后始终活跃，即使 `open=false` 时也会触发。

- [ ] **Step 1: 改用 watch 控制监听器注册**

在 `src/components/ui/KvModal.vue` 中，替换 script setup 内容：

```typescript
import { watch, onUnmounted } from 'vue'

interface Props {
  open: boolean
  title?: string
  width?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
}>()

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    document.addEventListener('keydown', handleKeydown)
  } else {
    document.removeEventListener('keydown', handleKeydown)
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/components/ui/KvModal.vue
git commit -m "fix: only listen for Escape when modal is open"
```

---

### Task 12: 添加修改密码 UI [I-7]

**Files:**
- Modify: `src/views/SettingsView.vue`

**问题:** 设置页面缺少修改密码入口。

- [ ] **Step 1: 添加修改密码表单状态和逻辑**

在 `src/views/SettingsView.vue` 的 script setup 中添加：

```typescript
import { ref } from 'vue'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const showChangePassword = ref(false)
const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const changingPassword = ref(false)

async function handleChangePassword() {
  if (newPassword.value !== confirmPassword.value) {
    toast.error('两次密码不一致')
    return
  }
  if (newPassword.value.length < 8) {
    toast.error('密码长度至少 8 位')
    return
  }
  changingPassword.value = true
  try {
    await auth.changePassword(oldPassword.value, newPassword.value)
    toast.success('密码已修改')
    showChangePassword.value = false
    oldPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
  } catch (e: any) {
    toast.error(e.message || '修改失败')
  } finally {
    changingPassword.value = false
  }
}
```

- [ ] **Step 2: 添加修改密码 UI 到模板**

在 `settings-section` 安全区域（自动锁定设置之后）添加：

```html
<div class="setting-row">
  <div class="setting-label">
    <span class="label-text">修改密码</span>
    <span class="label-hint">更改主密码</span>
  </div>
  <button class="btn-secondary" @click="showChangePassword = !showChangePassword">
    {{ showChangePassword ? '取消' : '修改' }}
  </button>
</div>

<div v-if="showChangePassword" class="change-password-form">
  <input
    v-model="oldPassword"
    type="password"
    placeholder="当前密码"
    class="password-input"
  />
  <input
    v-model="newPassword"
    type="password"
    placeholder="新密码（至少 8 位）"
    class="password-input"
  />
  <input
    v-model="confirmPassword"
    type="password"
    placeholder="确认新密码"
    class="password-input"
  />
  <button
    class="btn-primary"
    :disabled="changingPassword"
    @click="handleChangePassword"
  >
    {{ changingPassword ? '修改中...' : '确认修改' }}
  </button>
</div>
```

- [ ] **Step 3: 添加样式**

在 style scoped 中添加：

```css
.btn-secondary {
  padding: var(--space-2) var(--space-4);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.btn-secondary:hover {
  border-color: var(--accent-blue);
}

.change-password-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4) 0;
}

.password-input {
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.password-input:focus {
  border-color: var(--accent-blue);
  outline: none;
}

.btn-primary {
  padding: var(--space-2) var(--space-4);
  background: var(--accent-blue);
  border-radius: var(--radius-md);
  color: #fff;
  font-size: var(--text-sm);
  font-weight: 500;
  align-self: flex-start;
}

.btn-primary:hover {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

- [ ] **Step 4: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 5: 提交**

```bash
git add src/views/SettingsView.vue
git commit -m "feat: add change password UI to settings"
```

---

### Task 13: 添加泄露检测和导出/导入 UI [I-8, I-9]

**Files:**
- Modify: `src/views/SettingsView.vue`

**问题:** 设置页面缺少泄露检测入口和导出/导入功能。

- [ ] **Step 1: 添加泄露检测和导出/导入逻辑**

在 script setup 中添加：

```typescript
import { security, vault as vaultBridge } from '@/bridge/tauri'

const checkPassword = ref('')
const breachResult = ref<string | null>(null)
const checkingBreach = ref(false)

async function handleBreachCheck() {
  if (!checkPassword.value) return
  checkingBreach.value = true
  breachResult.value = null
  try {
    const result = await security.checkBreach(checkPassword.value)
    breachResult.value = result.found
      ? `该密码已泄露 ${result.count.toLocaleString()} 次`
      : '该密码未在已知泄露数据库中'
  } catch (e: any) {
    breachResult.value = '检测失败: ' + (e.message || '未知错误')
  } finally {
    checkingBreach.value = false
  }
}

async function handleExport() {
  try {
    const data = await vaultBridge.exportVault('json')
    // 使用 Tauri 的 dialog 保存文件
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (path) {
      const { writeTextFile } = await import('@tauri-apps/plugin-fs')
      await writeTextFile(path, data)
      toast.success('导出成功')
    }
  } catch (e: any) {
    toast.error(e.message || '导出失败')
  }
}
```

- [ ] **Step 2: 添加泄露检测 UI**

在安全 section 中添加：

```html
<div class="setting-row">
  <div class="setting-label">
    <span class="label-text">密码泄露检测</span>
    <span class="label-hint">检查密码是否在已知泄露数据库中</span>
  </div>
</div>
<div class="breach-check-form">
  <input
    v-model="checkPassword"
    type="password"
    placeholder="输入要检测的密码"
    class="password-input"
  />
  <button
    class="btn-secondary"
    :disabled="checkingBreach"
    @click="handleBreachCheck"
  >
    {{ checkingBreach ? '检测中...' : '检测' }}
  </button>
  <p v-if="breachResult" class="breach-result">{{ breachResult }}</p>
</div>
```

- [ ] **Step 3: 添加导出/导入 UI**

在关于 section 之后添加：

```html
<section class="settings-section">
  <h2 class="section-title">数据</h2>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">导出密码库</span>
      <span class="label-hint">导出为 JSON 文件</span>
    </div>
    <button class="btn-secondary" @click="handleExport">导出</button>
  </div>
</section>
```

- [ ] **Step 4: 添加样式**

```css
.breach-check-form {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  padding: var(--space-3) 0;
  align-items: center;
}

.breach-result {
  width: 100%;
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
```

- [ ] **Step 5: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 6: 提交**

```bash
git add src/views/SettingsView.vue
git commit -m "feat: add breach check and export UI to settings"
```

---

## Phase 4: Minor — 质量改进

### Task 14: 修复 useClipboard store 调用位置 [M-1]

**Files:**
- Modify: `src/composables/useClipboard.ts`

**问题:** `useSettingsStore()` 在 `copy()` 函数体内调用，应移到 composable body 顶部。

- [ ] **Step 1: 移动 store 调用**

在 `src/composables/useClipboard.ts` 中，将：

```typescript
export function useClipboard() {
  const timer = ref<ReturnType<typeof setTimeout> | null>(null)
  const copiedField = ref<string | null>(null)

  async function copy(text: string, fieldName = 'text'): Promise<number> {
    await clipboard.copy(text)
    copiedField.value = fieldName

    if (timer.value) clearTimeout(timer.value)

    const settings = useSettingsStore()
    const seconds = settings.clipboardClearSeconds || 30
```

改为：

```typescript
export function useClipboard() {
  const settings = useSettingsStore()
  const timer = ref<ReturnType<typeof setTimeout> | null>(null)
  const copiedField = ref<string | null>(null)

  async function copy(text: string, fieldName = 'text'): Promise<number> {
    await clipboard.copy(text)
    copiedField.value = fieldName

    if (timer.value) clearTimeout(timer.value)

    const seconds = settings.clipboardClearSeconds || 30
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/composables/useClipboard.ts
git commit -m "fix: move useSettingsStore call to composable body"
```

---

### Task 15: 修复 clipboard.clear() 未 await [M-2]

**Files:**
- Modify: `src/composables/useClipboard.ts:29`

**问题:** `clipboard.clear()` 返回 Promise 但未 await。

- [ ] **Step 1: 添加 async/await**

将 `clearClipboard` 函数改为 async：

```typescript
async function clearClipboard() {
  await clipboard.clear()
  copiedField.value = null
  if (timer.value) {
    clearTimeout(timer.value)
    timer.value = null
  }
}
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/composables/useClipboard.ts
git commit -m "fix: await clipboard.clear() in clearClipboard"
```

---

### Task 16: 集成密码生成器到 ItemForm [M-3]

**Files:**
- Modify: `src/components/vault/ItemForm.vue`

**问题:** `usePasswordGenerator` 未集成到表单 UI。

- [ ] **Step 1: 添加密码生成器导入和集成**

在 `src/components/vault/ItemForm.vue` 的 import 区域添加：

```typescript
import { usePasswordGenerator } from '@/composables/usePasswordGen'
```

在 script setup 中初始化：

```typescript
const passwordGen = usePasswordGenerator()
```

- [ ] **Step 2: 添加生成按钮到密码字段**

在字段编辑行的密码类型输入框之后，添加生成按钮：

将第 161-166 行的：
```html
<input
  v-model="field.value"
  :type="field.fieldType === 'password' ? 'password' : 'text'"
  class="field-value-input"
  placeholder="值"
/>
```

改为：
```html
<input
  v-model="field.value"
  :type="field.fieldType === 'password' ? 'password' : 'text'"
  class="field-value-input"
  placeholder="值"
/>
<button
  v-if="field.fieldType === 'password'"
  class="generate-btn"
  title="生成密码"
  @click="field.value = passwordGen.generate()"
>
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/>
  </svg>
</button>
```

- [ ] **Step 3: 添加样式**

```css
.generate-btn {
  padding: var(--space-1);
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.generate-btn:hover {
  color: var(--accent-blue);
  background: var(--bg-elevated);
}
```

- [ ] **Step 4: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 5: 提交**

```bash
git add src/components/vault/ItemForm.vue
git commit -m "feat: integrate password generator into ItemForm"
```

---

### Task 17: 搜索输入防抖 [M-4]

**Files:**
- Modify: `src/views/VaultView.vue`

**问题:** 搜索 `@input` 每次按键触发 IPC 调用。

- [ ] **Step 1: 添加 useDebounceFn 导入**

在 `src/views/VaultView.vue` 顶部添加：

```typescript
import { useDebounceFn } from '@vueuse/core'
```

- [ ] **Step 2: 创建防抖搜索函数**

在 `handleSaved()` 函数之后添加：

```typescript
const debouncedSearch = useDebounceFn((query: string) => {
  vault.search(query)
}, 300)
```

- [ ] **Step 3: 替换模板中的 @input**

将第 163 行：
```html
@input="vault.search(vault.searchQuery)"
```

改为：
```html
@input="debouncedSearch(vault.searchQuery)"
```

- [ ] **Step 4: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 5: 提交**

```bash
git add src/views/VaultView.vue
git commit -m "perf: debounce search input with 300ms delay"
```

---

### Task 18: mousemove 事件节流 [M-5]

**Files:**
- Modify: `src/composables/useAutoLock.ts`

**问题:** `mousemove` 未节流，60+Hz 触发。

- [ ] **Step 1: 添加 useThrottleFn 导入**

在 `src/composables/useAutoLock.ts` 顶部添加：

```typescript
import { useThrottleFn } from '@vueuse/core'
```

- [ ] **Step 2: 节流 mousemove 处理函数**

将 `handleActivity` 函数和事件监听改为：

```typescript
const throttledActivity = useThrottleFn(() => {
  resetTimer()
}, 1000)

onMounted(() => {
  resetTimer()
  document.addEventListener('mousemove', throttledActivity)
  document.addEventListener('keydown', throttledActivity)
  document.addEventListener('click', throttledActivity)
  window.addEventListener('blur', handleBlur)
})

onUnmounted(() => {
  if (timer.value) clearTimeout(timer.value)
  document.removeEventListener('mousemove', throttledActivity)
  document.removeEventListener('keydown', throttledActivity)
  document.removeEventListener('click', throttledActivity)
  window.removeEventListener('blur', handleBlur)
})
```

并删除原来的 `handleActivity` 函数和旧的事件监听代码。

- [ ] **Step 3: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 4: 提交**

```bash
git add src/composables/useAutoLock.ts
git commit -m "perf: throttle mousemove activity handler to 1/sec"
```

---

### Task 19: 修复 useShortcuts meta 键检查 [M-6]

**Files:**
- Modify: `src/composables/useShortcuts.ts:17`

**问题:** `meta` 属性在接口中定义但匹配逻辑中未检查。

- [ ] **Step 1: 添加 meta 匹配逻辑**

在 `src/composables/useShortcuts.ts` 中，将第 17 行：
```typescript
const ctrlMatch = h.ctrl ? (e.ctrlKey || e.metaKey) : true
```

改为：
```typescript
const ctrlMatch = h.ctrl ? (e.ctrlKey || e.metaKey) : h.meta ? e.metaKey : true
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/composables/useShortcuts.ts
git commit -m "fix: check meta property in shortcut matching"
```

---

### Task 20: 添加全局错误处理器 [M-7]

**Files:**
- Modify: `src/main.ts`

**问题:** 未设置 `app.config.errorHandler`。

- [ ] **Step 1: 添加错误处理器**

在 `src/main.ts` 中，将：

```typescript
const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
```

改为：

```typescript
const app = createApp(App)

app.config.errorHandler = (err, instance, info) => {
  console.error('[Vue Error]', err, info)
}

app.use(createPinia())
app.use(router)
app.mount('#app')
```

- [ ] **Step 2: 类型检查**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add src/main.ts
git commit -m "feat: add global Vue error handler"
```

---

## 验证清单

所有任务完成后，执行最终验证：

- [ ] `npm run type-check` — 零错误
- [ ] `npm run dev` — 启动无报错
- [ ] 手动测试：未解锁状态访问 /vault → 重定向到 /login
- [ ] 手动测试：设置页面修改自动锁定时间 → 重启后设置保留
- [ ] 手动测试：新建条目 → 关闭 → 编辑不同条目 → 表单数据正确重置
- [ ] 手动测试：密码字段有生成按钮
