<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { Search, Plus, Lock, Settings, Key, Star, Clock } from 'lucide-vue-next'
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
  icon: typeof Key
  action: () => void
}

const commands: CommandItem[] = [
  {
    id: 'new',
    label: '新建条目',
    shortcut: '⌘N',
    icon: Plus,
    action: () => { emit('newEntry'); emit('close') },
  },
  {
    id: 'lock',
    label: '锁定',
    shortcut: '⌘L',
    icon: Lock,
    action: async () => {
      await auth.lock()
      router.push('/login')
      emit('close')
    },
  },
  {
    id: 'settings',
    label: '设置',
    icon: Settings,
    action: () => {
      router.push('/settings')
      emit('close')
    },
  },
]

const filteredEntries = computed(() => {
  if (!query.value.trim()) return vault.entries.slice(0, 10)
  const q = query.value.toLowerCase()
  return vault.entries
    .filter(e =>
      e.title.toLowerCase().includes(q) ||
      e.subtitle?.toLowerCase().includes(q) ||
      e.tags.some(t => t.toLowerCase().includes(q))
    )
    .slice(0, 10)
})

const filteredCommands = computed(() => {
  if (!query.value.trim()) return commands
  const q = query.value.toLowerCase()
  return commands.filter(c => c.label.toLowerCase().includes(q))
})

const allItems = computed(() => {
  if (mode.value === 'command') return filteredCommands.value.map(c => ({ type: 'command' as const, data: c }))
  return [
    ...filteredEntries.value.map(e => ({ type: 'entry' as const, data: e })),
  ]
})

watch(() => query.value, (val) => {
  mode.value = val.startsWith('>') ? 'command' : 'search'
  if (mode.value === 'command') {
    query.value = val.slice(1).trimStart()
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

function handleKeydown(e: KeyboardEvent) {
  const items = allItems.value
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % Math.max(items.length, 1)
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value - 1 + items.length) % Math.max(items.length, 1)
      break
    case 'Enter':
      e.preventDefault()
      if (items[selectedIndex.value]) {
        executeItem(items[selectedIndex.value])
      }
      break
    case 'Escape':
      emit('close')
      break
  }
}

function executeItem(item: { type: 'command' | 'entry'; data: any }) {
  if (item.type === 'command') {
    item.data.action()
  } else {
    emit('selectEntry', item.data)
    emit('close')
  }
}

</script>

<template>
  <Teleport to="body">
    <Transition name="palette">
      <div v-if="open" class="palette-overlay" @click.self="emit('close')">
        <div class="palette">
          <div class="palette-input-wrapper">
            <Search :size="16" class="palette-search-icon" />
            <input
              ref="inputRef"
              v-model="query"
              class="palette-input"
              :placeholder="mode === 'command' ? '输入命令...' : '搜索条目或输入 > 进入命令模式...'"
              @keydown="handleKeydown"
            />
          </div>

          <div class="palette-results">
            <!-- 命令模式 -->
            <template v-if="mode === 'command'">
              <div class="palette-section-title">命令</div>
              <div
                v-for="(cmd, i) in filteredCommands"
                :key="cmd.id"
                class="palette-item"
                :class="{ 'palette-item--selected': selectedIndex === i }"
                @click="cmd.action()"
                @mouseenter="selectedIndex = i"
              >
                <component :is="cmd.icon" :size="14" class="palette-item-icon" />
                <span class="palette-item-label">{{ cmd.label }}</span>
                <span v-if="cmd.shortcut" class="palette-item-shortcut">{{ cmd.shortcut }}</span>
              </div>
            </template>

            <!-- 搜索模式 -->
            <template v-else>
              <div v-if="filteredEntries.length > 0" class="palette-section-title">
                {{ query ? '搜索结果' : '最近使用' }}
              </div>
              <div
                v-for="(entry, i) in filteredEntries"
                :key="entry.id"
                class="palette-item"
                :class="{ 'palette-item--selected': selectedIndex === i }"
                @click="emit('selectEntry', entry); emit('close')"
                @mouseenter="selectedIndex = i"
              >
                <div class="palette-item-icon entry-type-dot" />
                <div class="palette-item-info">
                  <span class="palette-item-label">{{ entry.title }}</span>
                  <span v-if="entry.subtitle" class="palette-item-sub">{{ entry.subtitle }}</span>
                </div>
              </div>

              <div v-if="filteredEntries.length === 0 && query" class="palette-empty">
                无匹配结果
              </div>
            </template>
          </div>

          <div class="palette-footer">
            <span class="palette-hint">
              <kbd>&uarr;&darr;</kbd> 导航
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
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  padding-top: 15vh;
  z-index: var(--z-palette);
}

.palette {
  width: 520px;
  max-height: 420px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
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
  background: var(--bg-elevated);
}

.palette-item-icon {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.entry-type-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-blue);
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

/* Transitions */
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
