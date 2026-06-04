<script setup lang="ts">
import KvIcon from '@/components/icons/KvIcon.vue'

export type SettingsTabId = 'general' | 'security' | 'data' | 'about'

const props = withDefaults(defineProps<{
  active?: SettingsTabId
}>(), {
  active: 'general',
})

const emit = defineEmits<{
  navigate: [id: SettingsTabId]
}>()

const tabs: { id: SettingsTabId; label: string; icon: string }[] = [
  { id: 'general', label: '常规', icon: 'tune' },
  { id: 'security', label: '安全', icon: 'shield' },
  { id: 'data', label: '数据', icon: 'database' },
  { id: 'about', label: '关于', icon: 'info' },
]
</script>

<template>
  <nav class="settings-nav">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      type="button"
      class="settings-nav-item"
      :class="{ active: props.active === tab.id }"
      @click="emit('navigate', tab.id)"
    >
      <KvIcon :name="tab.icon" :size="18" />
      <span>{{ tab.label }}</span>
    </button>
  </nav>
</template>

<style scoped>
.settings-nav {
  width: 192px;
  flex-shrink: 0;
  padding: var(--space-4) var(--space-2);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  background: var(--bg-surface);
}

.settings-nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  text-align: left;
  transition: all var(--duration-fast);
}

.settings-nav-item:hover {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.settings-nav-item.active {
  background: var(--bg-elevated);
  color: var(--text-accent);
  font-weight: 600;
  border-left: 2px solid var(--accent-blue);
  padding-left: calc(var(--space-3) - 2px);
}
</style>
