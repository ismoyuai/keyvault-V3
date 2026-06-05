# Phase 3: Missing UI Components Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the remaining UI components specified in the migration doc: PasswordGenerator, PasswordStrength, BreachBadge, ClipboardTimer, and virtual scrolling for the entry list.

**Architecture:** Each component follows the existing pattern — Vue 3 SFC with `<script setup lang="ts">`, scoped styles using design tokens from `src/design/tokens.css`, and composable functions from `src/composables/`.

**Tech Stack:** Vue 3, TypeScript, Vite, design tokens

---

### Task 1: PasswordStrength Component

**Files:**
- Create: `src/components/security/PasswordStrength.vue`
- Modify: `src/composables/usePasswordGen.ts` (if evaluateStrength needs export)

- [ ] **Step 1: Verify evaluateStrength is exported**

Read `src/composables/usePasswordGen.ts` and confirm `evaluateStrength` is exported. If not, add `export` to the function.

- [ ] **Step 2: Create PasswordStrength.vue**

```vue
<!-- src/components/security/PasswordStrength.vue -->
<script setup lang="ts">
import { computed } from 'vue'
import { evaluateStrength } from '@/composables/usePasswordGen'

const props = defineProps<{
  password: string
}>()

const strength = computed(() => evaluateStrength(props.password))

const barColor = computed(() => {
  const colors: Record<number, string> = {
    0: 'var(--text-disabled)',
    1: 'var(--color-danger)',
    2: 'var(--color-danger)',
    3: 'var(--color-warning)',
    4: 'var(--color-warning)',
    5: 'var(--color-success)',
    6: 'var(--color-success)',
  }
  return colors[strength.value.score] || 'var(--text-disabled)'
})

const label = computed(() => {
  const labels: Record<string, string> = {
    empty: '',
    weak: '弱',
    fair: '一般',
    good: '良好',
    strong: '强',
  }
  return labels[strength.value.level] || ''
})

const barWidth = computed(() => `${(strength.value.score / 6) * 100}%`)
</script>

<template>
  <div v-if="password" class="strength-bar">
    <div class="strength-track">
      <div
        class="strength-fill"
        :style="{ width: barWidth, backgroundColor: barColor }"
      />
    </div>
    <span class="strength-label" :style="{ color: barColor }">{{ label }}</span>
  </div>
</template>

<style scoped>
.strength-bar {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-1);
}
.strength-track {
  flex: 1;
  height: 3px;
  background: var(--bg-input);
  border-radius: 2px;
  overflow: hidden;
}
.strength-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 200ms var(--ease-out-quart), background-color 200ms;
}
.strength-label {
  font-size: var(--text-xs);
  font-weight: 500;
  min-width: 24px;
  text-align: right;
}
</style>
```

- [ ] **Step 3: Verify TypeScript compiles**

Run: `npm run type-check`
Expected: Clean.

- [ ] **Step 4: Commit**

```bash
git add src/components/security/PasswordStrength.vue
git commit -m "feat: add PasswordStrength visual indicator component"
```

---

### Task 2: BreachBadge Component

**Files:**
- Create: `src/components/security/BreachBadge.vue`

- [ ] **Step 1: Create BreachBadge.vue**

```vue
<!-- src/components/security/BreachBadge.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue'
import { security } from '@/bridge/tauri'

const props = defineProps<{
  password: string
}>()

const breached = ref(false)
const count = ref(0)
const loading = ref(false)
const checked = ref(false)

watch(() => props.password, async (pwd) => {
  if (!pwd || pwd.length < 1) {
    checked.value = false
    return
  }
  loading.value = true
  try {
    const result = await security.checkBreach(pwd)
    breached.value = result.breached
    count.value = result.count
    checked.value = true
  } catch {
    // Silently fail — breach check is non-critical
    checked.value = false
  } finally {
    loading.value = false
  }
}, { immediate: true })
</script>

<template>
  <div v-if="checked && breached" class="breach-badge" :title="`此密码已泄露 ${count.toLocaleString()} 次`">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
      <line x1="12" y1="9" x2="12" y2="13"/>
      <line x1="12" y1="17" x2="12.01" y2="17"/>
    </svg>
    <span>已泄露</span>
  </div>
</template>

<style scoped>
.breach-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  background: rgba(248, 81, 73, 0.15);
  color: var(--color-danger);
  font-size: var(--text-xs);
  font-weight: 500;
}
</style>
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npm run type-check`

- [ ] **Step 3: Commit**

```bash
git add src/components/security/BreachBadge.vue
git commit -m "feat: add BreachBadge component for password leak detection"
```

---

### Task 3: ClipboardTimer Component

**Files:**
- Create: `src/components/security/ClipboardTimer.vue`

- [ ] **Step 1: Create ClipboardTimer.vue**

```vue
<!-- src/components/security/ClipboardTimer.vue -->
<script setup lang="ts">
import { ref, onUnmounted, watch } from 'vue'
import { useClipboard } from '@/composables/useClipboard'

const props = defineProps<{
  /** Total seconds for the countdown */
  seconds: number
}>()

const { clearClipboard } = useClipboard()
const remaining = ref(0)
let intervalId: ReturnType<typeof setInterval> | null = null

function start() {
  stop()
  remaining.value = props.seconds
  intervalId = setInterval(() => {
    remaining.value--
    if (remaining.value <= 0) {
      stop()
      clearClipboard()
    }
  }, 1000)
}

function stop() {
  if (intervalId) {
    clearInterval(intervalId)
    intervalId = null
  }
  remaining.value = 0
}

watch(() => props.seconds, (val) => {
  if (val > 0) start()
  else stop()
})

onUnmounted(stop)

defineExpose({ start, stop, remaining })
</script>

<template>
  <div v-if="remaining > 0" class="clipboard-timer">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <polyline points="12 6 12 12 16 14"/>
    </svg>
    <span>{{ remaining }}s</span>
  </div>
</template>

<style scoped>
.clipboard-timer {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-variant-numeric: tabular-nums;
}
</style>
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npm run type-check`

- [ ] **Step 3: Commit**

```bash
git add src/components/security/ClipboardTimer.vue
git commit -m "feat: add ClipboardTimer countdown component"
```

---

### Task 4: PasswordGenerator Component

**Files:**
- Create: `src/components/generator/PasswordGenerator.vue`

- [ ] **Step 1: Create PasswordGenerator.vue**

```vue
<!-- src/components/generator/PasswordGenerator.vue -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import { usePasswordGen } from '@/composables/usePasswordGen'
import PasswordStrength from '@/components/security/PasswordStrength.vue'
import KvButton from '@/components/ui/KvButton.vue'

const { generate, evaluateStrength } = usePasswordGen()

const emit = defineEmits<{
  select: [password: string]
}>()

const mode = ref<'random' | 'diceware'>('random')
const length = ref(20)
const uppercase = ref(true)
const lowercase = ref(true)
const numbers = ref(true)
const symbols = ref(true)
const excludeAmbiguous = ref(false)
const wordCount = ref(4)

const password = ref('')
const copied = ref(false)

function generatePassword() {
  if (mode.value === 'random') {
    password.value = generate({
      length: length.value,
      uppercase: uppercase.value,
      lowercase: lowercase.value,
      numbers: numbers.value,
      symbols: symbols.value,
      excludeAmbiguous: excludeAmbiguous.value,
      mode: 'random',
    })
  } else {
    password.value = generate({
      length: 0,
      uppercase: false,
      lowercase: false,
      numbers: false,
      symbols: false,
      excludeAmbiguous: false,
      mode: 'diceware',
      wordCount: wordCount.value,
    })
  }
  copied.value = false
}

async function copyPassword() {
  try {
    await navigator.clipboard.writeText(password.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch {
    // Fallback: emit for parent to handle
  }
}

// Generate on mount
generatePassword()
</script>

<template>
  <div class="password-generator">
    <!-- Mode toggle -->
    <div class="mode-toggle">
      <button :class="{ active: mode === 'random' }" @click="mode = 'random'; generatePassword()">随机</button>
      <button :class="{ active: mode === 'diceware' }" @click="mode = 'diceware'; generatePassword()">Diceware</button>
    </div>

    <!-- Generated password -->
    <div class="password-display">
      <code class="password-text">{{ password }}</code>
      <div class="password-actions">
        <KvButton size="sm" variant="ghost" @click="copyPassword">
          {{ copied ? '✓ 已复制' : '复制' }}
        </KvButton>
        <KvButton size="sm" variant="ghost" @click="generatePassword">
          刷新
        </KvButton>
      </div>
    </div>

    <PasswordStrength :password="password" />

    <!-- Options -->
    <div v-if="mode === 'random'" class="options">
      <div class="option-row">
        <label>长度: {{ length }}</label>
        <input v-model.number="length" type="range" min="8" max="128" @input="generatePassword()" />
      </div>
      <div class="option-row">
        <label><input v-model="uppercase" type="checkbox" @change="generatePassword()" /> 大写 A-Z</label>
        <label><input v-model="lowercase" type="checkbox" @change="generatePassword()" /> 小写 a-z</label>
      </div>
      <div class="option-row">
        <label><input v-model="numbers" type="checkbox" @change="generatePassword()" /> 数字 0-9</label>
        <label><input v-model="symbols" type="checkbox" @change="generatePassword()" /> 符号 !@#$</label>
      </div>
      <div class="option-row">
        <label><input v-model="excludeAmbiguous" type="checkbox" @change="generatePassword()" /> 排除易混淆字符</label>
      </div>
    </div>

    <div v-else class="options">
      <div class="option-row">
        <label>词数: {{ wordCount }}</label>
        <input v-model.number="wordCount" type="range" min="3" max="8" @input="generatePassword()" />
      </div>
    </div>

    <!-- Use button -->
    <KvButton variant="primary" @click="emit('select', password)">使用此密码</KvButton>
  </div>
</template>

<style scoped>
.password-generator {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.mode-toggle {
  display: flex;
  gap: 2px;
  background: var(--bg-input);
  border-radius: var(--radius-sm);
  padding: 2px;
}
.mode-toggle button {
  flex: 1;
  padding: var(--space-1) var(--space-2);
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: var(--text-sm);
  border-radius: 3px;
  cursor: pointer;
}
.mode-toggle button.active {
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.password-display {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
}
.password-text {
  flex: 1;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  word-break: break-all;
  user-select: all;
}
.password-actions {
  display: flex;
  gap: var(--space-1);
  flex-shrink: 0;
}
.options {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.option-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
.option-row input[type="range"] {
  flex: 1;
  accent-color: var(--accent-blue);
}
.option-row label {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  cursor: pointer;
}
</style>
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npm run type-check`

- [ ] **Step 3: Commit**

```bash
git add src/components/generator/PasswordGenerator.vue
git commit -m "feat: add PasswordGenerator UI component with random and diceware modes"
```

---

### Task 5: Virtual Scrolling for Entry List

**Files:**
- Modify: `src/views/VaultView.vue`

- [ ] **Step 1: Install @vueuse/core if not already installed**

Check `package.json` for `@vueuse/core`. If missing:
```bash
npm install @vueuse/core
```

- [ ] **Step 2: Implement virtual scrolling in VaultView.vue**

Replace the entry list section in `VaultView.vue` (the part that uses `v-for="entry in filteredEntries"`) with virtual scrolling.

The key change is in the template and script:

In `<script setup>`, add:
```typescript
import { useVirtualList } from '@vueuse/core'

const ITEM_HEIGHT = 52 // matches --list-item-height

const { list, containerProps, wrapperProps } = useVirtualList(
  filteredEntries,
  {
    itemHeight: ITEM_HEIGHT,
    overscan: 5,
  }
)
```

In the template, replace the entry list container:
```html
<!-- Before: plain v-for -->
<!-- <div class="entry-list">
  <VaultItem v-for="entry in filteredEntries" ... />
</div> -->

<!-- After: virtual scrolling -->
<div v-bind="containerProps" class="entry-list">
  <div v-bind="wrapperProps">
    <VaultItem
      v-for="item in list"
      :key="item.data.id"
      :entry="item.data"
      :selected="item.data.id === vaultStore.selectedId"
      @click="vaultStore.selectEntry(item.data.id)"
    />
  </div>
</div>
```

Ensure the `.entry-list` container has a fixed height (not `height: auto`):
```css
.entry-list {
  flex: 1;
  overflow-y: auto;
  /* The containerProps will handle the scroll */
}
```

- [ ] **Step 3: Verify TypeScript compiles**

Run: `npm run type-check`

- [ ] **Step 4: Commit**

```bash
git add src/views/VaultView.vue package.json
git commit -m "feat: add virtual scrolling to entry list for performance"
```

---

### Task 6: Integrate Components into Existing Views

**Files:**
- Modify: `src/components/vault/ItemDetail.vue` (add BreachBadge)
- Modify: `src/views/VaultView.vue` (add PasswordGenerator modal, ClipboardTimer)

- [ ] **Step 1: Add BreachBadge to ItemDetail.vue**

In `ItemDetail.vue`, after the password field display, add the BreachBadge:

```vue
<script setup>
import BreachBadge from '@/components/security/BreachBadge.vue'
</script>

<!-- In the template, after the password field value display: -->
<BreachBadge v-if="field.fieldType === 'password'" :password="field.value" />
```

- [ ] **Step 2: Add ClipboardTimer to VaultView.vue header**

In `VaultView.vue`, add the ClipboardTimer near the titlebar or status area:

```vue
<script setup>
import ClipboardTimer from '@/components/security/ClipboardTimer.vue'
import { useSettingsStore } from '@/stores/settings'

const settingsStore = useSettingsStore()
const clipboardTimerRef = ref()
</script>

<!-- In template, near the window controls: -->
<ClipboardTimer
  ref="clipboardTimerRef"
  :seconds="settingsStore.clipboardClearSeconds"
/>
```

- [ ] **Step 3: Add PasswordGenerator modal trigger**

In `VaultView.vue`, add a modal that shows the PasswordGenerator (triggered from sidebar or command palette):

```vue
<script setup>
import PasswordGenerator from '@/components/generator/PasswordGenerator.vue'
import KvModal from '@/components/ui/KvModal.vue'

const showGenerator = ref(false)
</script>

<!-- In template: -->
<KvModal v-model:visible="showGenerator" title="密码生成器">
  <PasswordGenerator @select="handleGeneratedPassword" />
</KvModal>
```

Add a handler:
```typescript
async function handleGeneratedPassword(pwd: string) {
  await navigator.clipboard.writeText(pwd)
  showGenerator.value = false
  toast.success('密码已复制到剪贴板')
}
```

- [ ] **Step 4: Verify TypeScript compiles**

Run: `npm run type-check`

- [ ] **Step 5: Commit**

```bash
git add src/components/vault/ItemDetail.vue src/views/VaultView.vue
git commit -m "feat: integrate PasswordStrength, BreachBadge, ClipboardTimer, and PasswordGenerator"
```
