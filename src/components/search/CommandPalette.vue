<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import KvIcon from '@/components/icons/KvIcon.vue'
import { useVaultStore } from '@/stores/vault'
import { useAuthStore } from '@/stores/auth'
import { useRouter } from 'vue-router'
import type { EntryMeta } from '@/types/vault'

interface Props {
  open: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  selectEntry: [entry: EntryMeta]
  newEntry: []
}>()

const router = useRouter()
const vault = useVaultStore()
const auth = useAuthStore()
const query = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const selectedIndex = ref(0)
const mode = ref<'search' | 'command'>('search')

interface CommandItem {
  id: string
  label: string
  shortcut?: string
  icon: string
  action: () => void
}

const commands: CommandItem[] = [
  {
    id: 'new',
    label: '新建条目',
    shortcut: 'Ctrl+N',
    icon: 'add',
    action: () => { emit('newEntry'); emit('close') },
  },
  {
    id: 'lock',
    label: '锁定密码库',
    shortcut: 'Ctrl+L',
    icon: 'lock',
    action: async () => {
      await auth.lock()
      router.push('/login')
      emit('close')
    },
  },
  {
    id: 'settings',
    label: '打开设置',
    shortcut: ',',
    icon: 'settings',
    action: () => {
      router.push('/settings')
      emit('close')
    },
  },
]

const filteredEntries = computed(() => {
  if (!query.value.trim()) return vault.entries.slice(0, 8)
  const q = query.value.toLowerCase()
  return vault.entries
    .filter(e =>
      e.title.toLowerCase().includes(q) ||
      e.subtitle?.toLowerCase().includes(q) ||
      e.tags.some(t => t.toLowerCase().includes(q)),
    )
    .slice(0, 10)
})

const filteredCommands = computed(() => {
  if (!query.value.trim()) return commands
  const q = query.value.toLowerCase()
  return commands.filter(c => c.label.toLowerCase().includes(q))
})

type PaletteRow =
  | { kind: 'entry'; data: EntryMeta }
  | { kind: 'command'; data: CommandItem }

const flatRows = computed((): PaletteRow[] => {
  if (mode.value === 'command') {
    return filteredCommands.value.map(c => ({ kind: 'command' as const, data: c }))
  }
  const rows: PaletteRow[] = filteredEntries.value.map(e => ({ kind: 'entry' as const, data: e }))
  if (!query.value.trim()) {
    for (const c of filteredCommands.value) {
      rows.push({ kind: 'command', data: c })
    }
  }
  return rows
})

watch(() => query.value, (val) => {
  if (val.startsWith('>')) {
    mode.value = 'command'
    query.value = val.slice(1).trimStart()
  } else {
    mode.value = 'search'
  }
  selectedIndex.value = 0
})

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    query.value = ''
    mode.value = 'search'
    selectedIndex.value = 0
    nextTick(() => inputRef.value?.focus())
  }
})

function executeRow(row: PaletteRow) {
  if (row.kind === 'command') {
    row.data.action()
  } else {
    emit('selectEntry', row.data)
    emit('close')
  }
}

function handleKeydown(e: KeyboardEvent) {
  const rows = flatRows.value
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % Math.max(rows.length, 1)
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value - 1 + rows.length) % Math.max(rows.length, 1)
      break
    case 'Enter':
      e.preventDefault()
      if (rows[selectedIndex.value]) executeRow(rows[selectedIndex.value])
      break
    case 'Escape':
      emit('close')
      break
  }
}

function sectionForIndex(index: number): 'entries' | 'commands' | null {
  if (mode.value === 'command') return 'commands'
  const entryCount = filteredEntries.value.length
  if (index < entryCount) return 'entries'
  if (index < flatRows.value.length) return 'commands'
  return null
}
</script>

<template>
  <Teleport to="body">
    <Transition name="palette">
      <div v-if="open" class="palette-overlay" @click.self="emit('close')">
        <div class="palette">
          <div class="palette-input-wrapper">
            <KvIcon name="search" :size="18" class="palette-search-icon" />
            <input
              ref="inputRef"
              v-model="query"
              class="palette-input"
              :placeholder="mode === 'command' ? '输入命令…' : '搜索条目，或输入 > 进入命令模式'"
              @keydown="handleKeydown"
            />
          </div>

          <div class="palette-results">
            <template v-if="flatRows.length === 0">
              <div class="palette-empty">无匹配结果</div>
            </template>
            <template v-else>
              <template v-for="(row, i) in flatRows" :key="row.kind === 'entry' ? row.data.id : row.data.id + '-cmd'">
                <div
                  v-if="i === 0 || sectionForIndex(i) !== sectionForIndex(i - 1)"
                  class="palette-section-title"
                >
                  {{
                    sectionForIndex(i) === 'entries'
                      ? (query ? '搜索结果' : '最近条目')
                      : '常用命令'
                  }}
                </div>
                <div
                  class="palette-item"
                  :class="{ 'palette-item--selected': selectedIndex === i }"
                  @click="executeRow(row)"
                  @mouseenter="selectedIndex = i"
                >
                  <template v-if="row.kind === 'entry'">
                    <span class="palette-entry-dot" />
                    <div class="palette-item-info">
                      <span class="palette-item-label">{{ row.data.title }}</span>
                      <span v-if="row.data.subtitle" class="palette-item-sub">{{ row.data.subtitle }}</span>
                    </div>
                  </template>
                  <template v-else>
                    <KvIcon :name="row.data.icon" :size="18" class="palette-item-icon" />
                    <span class="palette-item-label">{{ row.data.label }}</span>
                    <span v-if="row.data.shortcut" class="palette-item-shortcut">{{ row.data.shortcut }}</span>
                  </template>
                </div>
              </template>
            </template>
          </div>

          <div class="palette-footer">
            <span class="palette-hint">
              <kbd>↑↓</kbd> 导航
              <kbd>Enter</kbd> 选择
              <kbd>Esc</kbd> 关闭
            </span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.palette-overlay {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--bg-base) 80%, transparent);
  backdrop-filter: blur(8px);
  display: flex;
  justify-content: center;
  padding-top: 12vh;
  z-index: var(--z-palette);
}

.palette {
  width: 100%;
  max-width: 600px;
  max-height: 420px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  margin: 0 var(--space-4);
}

.palette-input-wrapper {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
}

.palette-search-icon {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.palette-input {
  flex: 1;
  background: transparent;
  border: none;
  font-size: var(--text-md);
  color: var(--text-primary);
  outline: none;
}

.palette-input::placeholder {
  color: var(--text-tertiary);
}

.palette-results {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-2);
}

.palette-section-title {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: var(--space-2) var(--space-3);
}

.palette-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--duration-fast);
}

.palette-item:hover,
.palette-item--selected {
  background: var(--bg-surface);
}

.palette-item-icon {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.palette-entry-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-blue);
  flex-shrink: 0;
}

.palette-item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.palette-item-label {
  font-size: var(--text-sm);
  color: var(--text-primary);
}

.palette-item-sub {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.palette-item-shortcut {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  flex-shrink: 0;
  margin-left: auto;
}

.palette-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-8);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.palette-footer {
  padding: var(--space-2) var(--space-4);
  border-top: 1px solid var(--border-subtle);
  display: flex;
  justify-content: flex-end;
}

.palette-hint {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  display: flex;
  gap: var(--space-3);
}

.palette-hint kbd {
  padding: 1px 4px;
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: 3px;
  font-family: var(--font-mono);
  font-size: 10px;
}

.palette-enter-active {
  transition: opacity var(--duration-base) var(--ease-out-quart);
}

.palette-leave-active {
  transition: opacity var(--duration-fast);
}

.palette-enter-from,
.palette-leave-to {
  opacity: 0;
}

.palette-enter-active .palette {
  transition: transform var(--duration-base) var(--ease-spring), opacity var(--duration-base);
}

.palette-leave-active .palette {
  transition: transform var(--duration-fast), opacity var(--duration-fast);
}

.palette-enter-from .palette {
  transform: scale(0.98) translateY(-8px);
  opacity: 0;
}

.palette-leave-to .palette {
  transform: scale(0.98);
  opacity: 0;
}
</style>
