<script setup lang="ts">
import KvIcon from '@/components/icons/KvIcon.vue'

export type SideNavId =
  | 'all'
  | 'favorites'
  | 'passwords'
  | 'api-keys'
  | 'notes'
  | 'trash'

const props = withDefaults(defineProps<{
  activeNav?: SideNavId
}>(), {
  activeNav: 'all',
})

const emit = defineEmits<{
  'new-entry': []
  navigate: [id: SideNavId]
  settings: []
  lock: []
}>()

const navItems: { id: SideNavId; label: string; icon: string }[] = [
  { id: 'all', label: '全部条目', icon: 'inventory_2' },
  { id: 'favorites', label: '收藏', icon: 'star' },
  { id: 'passwords', label: '密码', icon: 'key' },
  { id: 'api-keys', label: 'API 密钥', icon: 'code' },
  { id: 'notes', label: '安全笔记', icon: 'description' },
  { id: 'trash', label: '回收站', icon: 'delete' },
]

function isActive(id: SideNavId) {
  return props.activeNav === id
}
</script>

<template>
  <aside class="kv-sidenav">
    <div class="kv-sidenav-vault">
      <div class="kv-sidenav-avatar">KV</div>
      <div class="kv-sidenav-vault-info">
        <span class="kv-sidenav-vault-name">Local Vault</span>
      </div>
      <KvIcon name="expand_more" :size="18" class="kv-sidenav-vault-chevron" />
    </div>

    <button type="button" class="kv-sidenav-cta" @click="emit('new-entry')">
      <KvIcon name="add" :size="18" />
      <span>新建条目</span>
    </button>

    <nav class="kv-sidenav-nav">
      <button
        v-for="item in navItems"
        :key="item.id"
        type="button"
        class="kv-sidenav-item"
        :class="{ active: isActive(item.id) }"
        @click="emit('navigate', item.id)"
      >
        <KvIcon :name="item.icon" :size="18" />
        <span class="kv-sidenav-item-label">{{ item.label }}</span>
      </button>
    </nav>

    <footer class="kv-sidenav-footer">
      <button type="button" class="kv-sidenav-footer-btn" @click="emit('settings')">
        <KvIcon name="settings" :size="18" />
        <span>设置</span>
      </button>
      <button type="button" class="kv-sidenav-footer-btn" @click="emit('lock')">
        <KvIcon name="lock" :size="18" />
        <span>锁定</span>
      </button>
    </footer>
  </aside>
</template>

<style scoped>
.kv-sidenav {
  display: flex;
  flex-direction: column;
  width: var(--sidebar-width);
  flex-shrink: 0;
  background: var(--bg-surface);
  border-right: 1px solid var(--border-subtle);
  overflow: hidden;
}

.kv-sidenav-vault {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-4) var(--space-3);
  cursor: default;
  opacity: 0.85;
}

.kv-sidenav-avatar {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: var(--accent-blue-dim);
  color: var(--text-accent);
  font-size: var(--text-xs);
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.kv-sidenav-vault-info {
  flex: 1;
  min-width: 0;
}

.kv-sidenav-vault-name {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.kv-sidenav-vault-chevron {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.kv-sidenav-cta {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  margin: 0 var(--space-4) var(--space-4);
  padding: var(--space-3) var(--space-4);
  width: calc(100% - var(--space-4) * 2);
  background: var(--accent-blue);
  color: var(--bg-base);
  font-size: var(--text-sm);
  font-weight: 600;
  border-radius: var(--radius-md);
  transition: opacity var(--duration-fast);
}

.kv-sidenav-cta:hover {
  opacity: 0.9;
}

.kv-sidenav-nav {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow-y: auto;
  padding: var(--space-2) 0;
}

.kv-sidenav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  width: 100%;
  padding: var(--space-3) var(--space-4);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  text-align: left;
  border-left: 2px solid transparent;
  transition:
    background var(--duration-fast),
    color var(--duration-fast);
}

.kv-sidenav-item:hover {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.kv-sidenav-item.active {
  border-left-color: var(--accent-blue);
  background: var(--bg-elevated);
  color: var(--text-accent);
  font-weight: 600;
}

.kv-sidenav-item-label {
  flex: 1;
}

.kv-sidenav-footer {
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--color-outline-variant);
  padding: var(--space-2) 0;
}

.kv-sidenav-footer-btn {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  width: 100%;
  padding: var(--space-3) var(--space-4);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  text-align: left;
  transition:
    background var(--duration-fast),
    color var(--duration-fast);
}

.kv-sidenav-footer-btn:hover {
  background: var(--bg-elevated);
  color: var(--text-primary);
}
</style>
