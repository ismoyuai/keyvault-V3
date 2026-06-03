<script setup lang="ts">
import { computed } from 'vue'
import { Key, Code, Terminal, Server, FileText, User, CreditCard, Package, Wallet, File } from 'lucide-vue-next'
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

const TYPE_ICONS: Record<EntryType, typeof Key> = {
  login: Key,
  api_key: Code,
  ssh_key: Terminal,
  server: Server,
  note: FileText,
  identity: User,
  card: CreditCard,
  license: Package,
  crypto: Wallet,
  custom: File,
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

const icon = computed(() => TYPE_ICONS[props.entry.entryType] || File)
const iconBg = computed(() => TYPE_COLORS[props.entry.entryType] || 'var(--type-custom)')
</script>

<template>
  <div
    class="vault-item"
    :class="{ 'vault-item--selected': selected }"
    @click="emit('select')"
  >
    <div class="vault-item__icon" :style="{ background: iconBg }">
      <component :is="icon" :size="14" />
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
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
          <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
        </svg>
      </button>
      <button
        class="vault-item__action"
        :class="{ 'vault-item__action--active': entry.favorited }"
        title="收藏"
        @click.stop="emit('toggleFavorite')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" :fill="entry.favorited ? 'var(--color-warning)' : 'none'" stroke="currentColor" stroke-width="2">
          <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
        </svg>
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
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  color: var(--text-primary);
}

.vault-item__info {
  flex: 1;
  min-width: 0;
}

.vault-item__title {
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.vault-item__subtitle {
  font-size: var(--text-xs);
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
