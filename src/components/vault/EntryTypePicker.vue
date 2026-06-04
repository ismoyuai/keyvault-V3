<script setup lang="ts">
import KvIcon from '@/components/icons/KvIcon.vue'
import { TEMPLATES } from '@/constants/templates'
import { ENTRY_TYPE_ICONS, VISIBLE_ENTRY_TYPES } from '@/constants/entryTypeIcons'
import type { EntryType } from '@/types/vault'

const emit = defineEmits<{
  select: [type: EntryType]
}>()

const types = VISIBLE_ENTRY_TYPES.map(id => ({
  id,
  name: TEMPLATES[id].name,
  icon: ENTRY_TYPE_ICONS[id],
}))
</script>

<template>
  <div class="type-picker">
    <p class="type-picker-hint">选择要创建的条目类型</p>
    <div class="type-grid">
      <button
        v-for="item in types"
        :key="item.id"
        type="button"
        class="type-card"
        @click="emit('select', item.id)"
      >
        <span class="type-card-icon">
          <KvIcon :name="item.icon" :size="22" />
        </span>
        <span class="type-card-name">{{ item.name }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.type-picker-hint {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  margin-bottom: var(--space-4);
}

.type-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--space-2);
}

.type-card {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  text-align: left;
  transition:
    border-color var(--duration-fast),
    background var(--duration-fast);
}

.type-card:hover {
  border-color: var(--accent-blue);
  background: var(--accent-blue-glow);
}

.type-card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-default);
  background: var(--bg-elevated);
  color: var(--text-accent);
  flex-shrink: 0;
}

.type-card-name {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-primary);
}
</style>
