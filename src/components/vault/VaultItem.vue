<script setup lang="ts">
import { computed } from 'vue'
import KvIcon from '@/components/icons/KvIcon.vue'
import type { EntryMeta, EntryType } from '@/types/vault'

interface Props {
  entry: EntryMeta
  selected?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  selected: false,
})

const emit = defineEmits<{
  select: []
  copy: []
  toggleFavorite: []
}>()

const TYPE_ICONS: Record<EntryType, string> = {
  login: 'key',
  api_key: 'code',
  ssh_key: 'terminal',
  server: 'dns',
  note: 'description',
  identity: 'person',
  card: 'credit_card',
  license: 'verified',
  crypto: 'account_balance_wallet',
  custom: 'draft',
}

const TYPE_COLORS: Record<EntryType, string> = {
  login: 'var(--type-login)',
  api_key: 'var(--type-api)',
  ssh_key: 'var(--type-ssh)',
  server: 'var(--type-server)',
  note: 'var(--type-note)',
  identity: 'var(--type-identity)',
  card: 'var(--type-card)',
  license: 'var(--type-license)',
  crypto: 'var(--type-crypto)',
  custom: 'var(--type-custom)',
}

const iconName = computed(() => TYPE_ICONS[props.entry.entryType] ?? 'draft')
const iconBg = computed(() => TYPE_COLORS[props.entry.entryType] ?? 'var(--type-custom)')
</script>

<template>
  <div
    class="vault-item"
    :class="{ 'vault-item--selected': selected }"
    @click="emit('select')"
  >
    <div class="vault-item__icon" :style="{ background: iconBg }">
      <KvIcon :name="iconName" :size="18" />
    </div>
    <div class="vault-item__info">
      <div class="vault-item__title">{{ entry.title }}</div>
      <div v-if="entry.subtitle" class="vault-item__subtitle">{{ entry.subtitle }}</div>
    </div>
    <div class="vault-item__actions">
      <button
        class="vault-item__action"
        title="复制"
        @click.stop="emit('copy')"
      >
        <KvIcon name="content_copy" :size="14" />
      </button>
      <button
        class="vault-item__action"
        :class="{ 'vault-item__action--active': entry.favorited }"
        title="收藏"
        @click.stop="emit('toggleFavorite')"
      >
        <KvIcon
          name="star"
          :size="14"
          :fill="entry.favorited"
        />
      </button>
    </div>
  </div>
</template>

<style scoped>
.vault-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  height: var(--list-item-height);
  padding: 0 var(--space-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--duration-fast);
  position: relative;
}

.vault-item:hover {
  background: var(--bg-elevated);
}

.vault-item--selected {
  background: var(--bg-elevated);
  border-left: 2px solid var(--accent-blue);
  padding-left: calc(var(--space-3) - 2px);
}

.vault-item__icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  flex-shrink: 0;
  color: var(--text-primary);
}

.vault-item__info {
  flex: 1;
  min-width: 0;
}

.vault-item__title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.vault-item__subtitle {
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 1px;
}

.vault-item__actions {
  display: flex;
  gap: var(--space-1);
  opacity: 0;
  transition: opacity var(--duration-fast);
}

.vault-item:hover .vault-item__actions {
  opacity: 1;
}

.vault-item__action {
  padding: var(--space-1);
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast);
}

.vault-item__action:hover {
  color: var(--text-primary);
  background: var(--bg-overlay);
}

.vault-item__action--active {
  color: var(--color-warning);
}
</style>
