<script setup lang="ts">
/**
 * 主界面 - 三栏布局
 * 左侧：侧边栏（分组/标签）
 * 中间：条目列表（虚拟滚动）
 * 右侧：详情面板（按需加载 secrets）
 */
import { onMounted } from 'vue'
import { useVaultStore } from '@/stores/vault'
import { useUiStore } from '@/stores/ui'
import { useShortcuts } from '@/composables/useShortcuts'

const vault = useVaultStore()
const ui = useUiStore()

onMounted(() => {
  vault.loadEntries()
})

useShortcuts([
  { key: 'k', ctrl: true, handler: () => ui.toggleCommandPalette() },
  { key: 'l', ctrl: true, handler: () => {/* lock */} },
  { key: 'n', ctrl: true, handler: () => {/* new entry */} },
])
</script>

<template>
  <div class="vault-layout">
    <!-- 标题栏 -->
    <header class="titlebar" data-tauri-drag-region>
      <div class="titlebar-drag" data-tauri-drag-region />
      <div class="titlebar-controls">
        <button class="titlebar-btn" @click="/* minimize */">─</button>
        <button class="titlebar-btn" @click="/* maximize */">□</button>
        <button class="titlebar-btn close" @click="/* close */">✕</button>
      </div>
    </header>

    <div class="vault-body">
      <!-- 侧边栏 -->
      <aside class="sidebar" :class="{ collapsed: ui.sidebarCollapsed }">
        <div class="sidebar-header">
          <h2 class="sidebar-title">KeyVault</h2>
        </div>
        <nav class="sidebar-nav">
          <button class="nav-item active">
            <span class="nav-icon">🔐</span>
            <span class="nav-label">全部</span>
          </button>
          <button class="nav-item">
            <span class="nav-icon">⭐</span>
            <span class="nav-label">收藏</span>
          </button>
          <button class="nav-item">
            <span class="nav-icon">🕐</span>
            <span class="nav-label">最近</span>
          </button>
        </nav>
        <div class="sidebar-footer">
          <button class="nav-item" @click="/* settings */">
            <span class="nav-icon">⚙️</span>
            <span class="nav-label">设置</span>
          </button>
        </div>
      </aside>

      <!-- 条目列表 -->
      <main class="entry-list">
        <div class="list-header">
          <input
            v-model="vault.searchQuery"
            type="text"
            placeholder="搜索条目..."
            class="search-input"
            @input="vault.search(vault.searchQuery)"
          />
          <button class="add-btn" title="新建条目 (Ctrl+N)">+</button>
        </div>
        <div class="list-body">
          <div
            v-for="entry in vault.entries"
            :key="entry.id"
            class="entry-item"
            :class="{ selected: vault.selectedId === entry.id }"
            @click="vault.selectEntry(entry.id)"
          >
            <div class="entry-icon">{{ entry.entryType === 'login' ? '🌐' : '🔑' }}</div>
            <div class="entry-info">
              <div class="entry-title">{{ entry.title }}</div>
              <div class="entry-subtitle">{{ entry.subtitle }}</div>
            </div>
          </div>
          <div v-if="!vault.isLoading && vault.entries.length === 0" class="empty-state">
            <p>暂无条目</p>
            <p class="hint">按 Ctrl+N 新建</p>
          </div>
        </div>
      </main>

      <!-- 详情面板 -->
      <aside v-if="ui.detailPanelOpen" class="detail-panel">
        <div class="detail-header">
          <h3>条目详情</h3>
          <button class="close-btn" @click="ui.closeDetailPanel()">✕</button>
        </div>
        <div class="detail-body">
          <p class="hint">选择一个条目查看详情</p>
        </div>
      </aside>
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
  justify-content: flex-end;
  height: var(--titlebar-height);
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
  -webkit-app-region: drag;
}

.titlebar-drag {
  flex: 1;
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
  font-size: var(--text-xs);
  color: var(--text-secondary);
  transition: background var(--duration-fast);
}

.titlebar-btn:hover {
  background: var(--bg-elevated);
}

.titlebar-btn.close:hover {
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

.sidebar-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
}

.sidebar-title {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
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

.sidebar-footer {
  padding: var(--space-2);
  border-top: 1px solid var(--border-subtle);
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
}

.search-input {
  flex: 1;
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-sm);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
}

.search-input:focus {
  border-color: var(--accent-blue);
}

.add-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--text-lg);
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  transition: all var(--duration-fast);
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

.entry-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  height: var(--list-item-height);
  padding: 0 var(--space-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--duration-fast);
}

.entry-item:hover {
  background: var(--bg-elevated);
}

.entry-item.selected {
  background: var(--bg-elevated);
  border-left: 2px solid var(--accent-blue);
}

.entry-icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  background: var(--type-login);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.entry-info {
  flex: 1;
  min-width: 0;
}

.entry-title {
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.entry-subtitle {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.hint {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  margin-top: var(--space-2);
}

/* 详情面板 */
.detail-panel {
  width: var(--detail-panel-width);
  background: var(--bg-surface);
  border-left: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  animation: slideInRight var(--duration-slow) var(--ease-out-quart);
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
}

.detail-header h3 {
  font-size: var(--text-md);
  font-weight: 600;
}

.close-btn {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.close-btn:hover {
  color: var(--text-primary);
}

.detail-body {
  flex: 1;
  padding: var(--space-4);
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
