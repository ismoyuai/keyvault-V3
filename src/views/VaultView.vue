<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useDebounceFn, useVirtualList } from '@vueuse/core'
import { useRouter } from 'vue-router'
import { Plus, Lock, Settings, Star, Clock, Key, Search, Wand2 } from 'lucide-vue-next'
import { useVaultStore } from '@/stores/vault'
import { useUiStore } from '@/stores/ui'
import { useShortcuts } from '@/composables/useShortcuts'
import { useAutoLock } from '@/composables/useAutoLock'
import { useClipboard } from '@/composables/useClipboard'
import { useToast } from '@/composables/useToast'
import { useSettingsStore } from '@/stores/settings'
import { window as windowBridge } from '@/bridge/tauri'
import VaultItem from '@/components/vault/VaultItem.vue'
import ItemDetail from '@/components/vault/ItemDetail.vue'
import ItemForm from '@/components/vault/ItemForm.vue'
import CommandPalette from '@/components/search/CommandPalette.vue'
import PasswordGenerator from '@/components/generator/PasswordGenerator.vue'
import ClipboardTimer from '@/components/security/ClipboardTimer.vue'
import KvModal from '@/components/ui/KvModal.vue'
import KvToast from '@/components/ui/KvToast.vue'
import type { EntryMeta } from '@/types/vault'

const router = useRouter()
const vault = useVaultStore()
const ui = useUiStore()
const settings = useSettingsStore()
const { lock } = useAutoLock()
const { copy } = useClipboard()
const toast = useToast()

const selectedEntry = ref<EntryMeta | null>(null)
const formOpen = ref(false)
const editEntry = ref<EntryMeta | null>(null)
const activeView = ref<'all' | 'favorites' | 'recent'>('all')
const showGenerator = ref(false)

onMounted(() => {
  vault.loadEntries()
})

useShortcuts([
  { key: 'k', ctrl: true, handler: () => ui.toggleCommandPalette() },
  { key: 'l', ctrl: true, handler: () => lock() },
  { key: 'n', ctrl: true, handler: () => { formOpen.value = true; editEntry.value = null } },
])

function selectEntry(entry: EntryMeta) {
  selectedEntry.value = entry
  vault.selectEntry(entry.id)
  ui.openDetailPanel()
}

function handleCopy(entry: EntryMeta) {
  copy(entry.title, 'title')
  toast.success('已复制标题')
}

async function handleDelete(entry: EntryMeta) {
  try {
    await vault.deleteEntry(entry.id)
    if (selectedEntry.value?.id === entry.id) {
      selectedEntry.value = null
      ui.closeDetailPanel()
    }
    toast.success('条目已删除')
  } catch (e: any) {
    toast.error(e.message || '删除失败')
  }
}

function handleEdit(entry: EntryMeta) {
  editEntry.value = entry
  formOpen.value = true
}

function handleSaved() {
  vault.loadEntries()
}

function handleGeneratedPassword(pwd: string) {
  navigator.clipboard.writeText(pwd)
  showGenerator.value = false
  toast.success('密码已复制到剪贴板')
}

const debouncedSearch = useDebounceFn((query: string) => {
  vault.search(query)
}, 300)

const favoritesCount = computed(() => vault.entries.filter(e => e.favorited).length)

const filteredEntries = computed(() => {
  if (activeView.value === 'favorites') return vault.entries.filter(e => e.favorited)
  if (activeView.value === 'recent') return vault.entries.slice(0, 20)
  return vault.entries
})

// 虚拟滚动
const ITEM_HEIGHT = 52
const { list: virtualList, containerProps, wrapperProps } = useVirtualList(
  filteredEntries,
  {
    itemHeight: ITEM_HEIGHT,
    overscan: 5,
  }
)
</script>

<template>
  <div class="vault-layout">
    <!-- 标题栏 -->
    <header class="titlebar" data-tauri-drag-region>
      <div class="titlebar-drag" data-tauri-drag-region>
        <span class="titlebar-title">KeyVault</span>
      </div>
      <div class="titlebar-controls">
        <button class="titlebar-btn" title="最小化" @click="windowBridge.minimize()">
          <svg width="12" height="12" viewBox="0 0 12 12"><line x1="1" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="1.2"/></svg>
        </button>
        <button class="titlebar-btn" title="最大化" @click="windowBridge.maximize()">
          <svg width="12" height="12" viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" rx="1" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
        </button>
        <button class="titlebar-btn titlebar-btn--close" title="关闭" @click="windowBridge.close()">
          <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.2"/></svg>
        </button>
      </div>
    </header>

    <div class="vault-body">
      <!-- 侧边栏 -->
      <aside class="sidebar">
        <nav class="sidebar-nav">
          <button
            class="nav-item"
            :class="{ active: activeView === 'all' }"
            @click="activeView = 'all'"
          >
            <Key :size="15" />
            <span class="nav-label">全部</span>
            <span class="nav-count">{{ vault.entries.length }}</span>
          </button>
          <button
            class="nav-item"
            :class="{ active: activeView === 'favorites' }"
            @click="activeView = 'favorites'"
          >
            <Star :size="15" />
            <span class="nav-label">收藏</span>
            <span class="nav-count">{{ favoritesCount }}</span>
          </button>
          <button
            class="nav-item"
            :class="{ active: activeView === 'recent' }"
            @click="activeView = 'recent'"
          >
            <Clock :size="15" />
            <span class="nav-label">最近</span>
          </button>
        </nav>

        <div class="sidebar-footer">
          <button class="nav-item" @click="router.push('/settings')">
            <Settings :size="15" />
            <span class="nav-label">设置</span>
          </button>
          <button class="nav-item" @click="lock">
            <Lock :size="15" />
            <span class="nav-label">锁定</span>
          </button>
        </div>
      </aside>

      <!-- 条目列表 -->
      <main class="entry-list">
        <div class="list-header">
          <div class="search-wrapper">
            <Search :size="14" class="search-icon" />
            <input
              v-model="vault.searchQuery"
              type="text"
              placeholder="搜索条目..."
              class="search-input"
              @input="debouncedSearch(vault.searchQuery)"
            />
          </div>
          <ClipboardTimer :seconds="settings.clipboardClearSeconds" />
          <button
            class="add-btn"
            title="密码生成器"
            @click="showGenerator = true"
          >
            <Wand2 :size="16" />
          </button>
          <button
            class="add-btn"
            title="新建条目 (Ctrl+N)"
            @click="formOpen = true; editEntry = null"
          >
            <Plus :size="16" />
          </button>
        </div>

        <div v-bind="containerProps" class="list-body">
          <div v-bind="wrapperProps">
            <VaultItem
              v-for="item in virtualList"
              :key="item.data.id"
              :entry="item.data"
              :selected="selectedEntry?.id === item.data.id"
              @select="selectEntry(item.data)"
              @copy="handleCopy(item.data)"
              @toggle-favorite="vault.toggleFavorite(item.data.id)"
            />
          </div>

          <div v-if="!vault.isLoading && filteredEntries.length === 0" class="empty-state">
            <Key :size="32" class="empty-icon" />
            <p class="empty-title">暂无条目</p>
            <p class="empty-hint">按 Ctrl+N 新建</p>
          </div>
        </div>
      </main>

      <!-- 详情面板 -->
      <ItemDetail
        v-if="ui.detailPanelOpen && selectedEntry"
        :entry="selectedEntry"
        @close="ui.closeDetailPanel(); selectedEntry = null"
        @edit="handleEdit"
        @delete="handleDelete"
      />
    </div>

    <!-- 命令面板 -->
    <CommandPalette
      :open="ui.commandPaletteOpen"
      @close="ui.commandPaletteOpen = false"
      @select-entry="selectEntry"
      @new-entry="formOpen = true; editEntry = null"
    />

    <!-- 新建/编辑表单 -->
    <ItemForm
      :open="formOpen"
      :edit-entry="editEntry"
      @close="formOpen = false; editEntry = null"
      @saved="handleSaved"
    />

    <!-- 密码生成器 -->
    <KvModal
      :open="showGenerator"
      title="密码生成器"
      @close="showGenerator = false"
    >
      <PasswordGenerator @select="handleGeneratedPassword" />
    </KvModal>

    <!-- Toast 容器 -->
    <div class="toast-container">
      <KvToast
        v-for="t in toast.toasts.value"
        :key="t.id"
        :message="t.message"
        :type="t.type"
        :duration="t.duration"
        @close="toast.remove(t.id)"
      />
    </div>
  </div>
</template>

<style scoped>
.vault-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-base);
}

/* 标题栏 */
.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--titlebar-height);
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
  -webkit-app-region: drag;
  flex-shrink: 0;
}

.titlebar-drag {
  flex: 1;
  padding-left: var(--space-4);
  display: flex;
  align-items: center;
}

.titlebar-title {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-weight: 500;
  letter-spacing: 0.05em;
}

.titlebar-controls {
  display: flex;
  -webkit-app-region: no-drag;
}

.titlebar-btn {
  width: 46px;
  height: var(--titlebar-height);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  transition: background var(--duration-fast);
}

.titlebar-btn:hover {
  background: var(--bg-elevated);
}

.titlebar-btn--close:hover {
  background: var(--color-danger);
  color: #fff;
}

/* 主体 */
.vault-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* 侧边栏 */
.sidebar {
  width: var(--sidebar-width);
  background: var(--bg-surface);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-nav {
  flex: 1;
  padding: var(--space-2);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  transition: all var(--duration-fast);
}

.nav-item:hover {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-blue-dim);
  color: var(--text-accent);
}

.nav-label {
  flex: 1;
}

.nav-count {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
}

.sidebar-footer {
  padding: var(--space-2);
  border-top: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

/* 条目列表 */
.entry-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.list-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.search-wrapper {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  transition: border-color var(--duration-fast);
}

.search-wrapper:focus-within {
  border-color: var(--accent-blue);
}

.search-icon {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  font-size: var(--text-sm);
  color: var(--text-primary);
  outline: none;
  padding: 0;
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.add-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  transition: all var(--duration-fast);
  flex-shrink: 0;
}

.add-btn:hover {
  background: var(--accent-blue-dim);
  color: var(--accent-blue);
}

.list-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-2);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 240px;
  gap: var(--space-2);
}

.empty-icon {
  color: var(--text-disabled);
  margin-bottom: var(--space-2);
}

.empty-title {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
}

.empty-hint {
  font-size: var(--text-xs);
  color: var(--text-disabled);
}

/* Toast 容器 */
.toast-container {
  position: fixed;
  bottom: var(--space-5);
  right: var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  z-index: var(--z-toast);
  pointer-events: none;
}
</style>
