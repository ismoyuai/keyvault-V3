<script setup lang="ts">
import { computed } from 'vue'
import KvIcon from '@/components/icons/KvIcon.vue'

export type VaultEmptyVariant = 'no-items' | 'no-search' | 'trash-empty'

const props = defineProps<{
  variant: VaultEmptyVariant
  searchQuery?: string
}>()

const emit = defineEmits<{
  create: []
  clearSearch: []
}>()

const config = computed(() => {
  switch (props.variant) {
    case 'no-search':
      return {
        icon: 'search_off',
        iconSize: 24,
        iconWrap: 'compact' as const,
        title: '未找到匹配条目',
        showQuery: true,
        showCreate: false,
        showClear: true,
      }
    case 'trash-empty':
      return {
        icon: 'delete',
        iconSize: 40,
        iconWrap: 'plain' as const,
        title: '回收站是空的',
        description:
          'v1 当前为永久删除，软删除 API 尚未接入。接入后，已删除条目将在此保留 30 天，之后永久销毁。',
        showQuery: false,
        showCreate: false,
        showClear: false,
      }
    default:
      return {
        icon: 'inventory_2',
        iconSize: 32,
        iconWrap: 'circle' as const,
        title: '暂无条目',
        description: '点击「新建条目」添加第一条，您的数据将进行本地端到端加密存储。',
        showQuery: false,
        showCreate: true,
        showClear: false,
      }
  }
})
</script>

<template>
  <div class="vault-empty" :class="`vault-empty--${variant}`">
    <div
      v-if="config.iconWrap === 'circle'"
      class="vault-empty__icon vault-empty__icon--circle"
    >
      <KvIcon :name="config.icon" :size="config.iconSize" />
    </div>
    <div
      v-else-if="config.iconWrap === 'compact'"
      class="vault-empty__icon vault-empty__icon--compact"
    >
      <KvIcon :name="config.icon" :size="config.iconSize" />
    </div>
    <KvIcon
      v-else
      :name="config.icon"
      :size="config.iconSize"
      class="vault-empty__icon-trash"
    />

    <h2 class="vault-empty__title">{{ config.title }}</h2>

    <p v-if="variant === 'no-items'" class="vault-empty__desc">
      {{ config.description }}
    </p>

    <p v-else-if="variant === 'trash-empty'" class="vault-empty__desc vault-empty__desc--center">
      {{ config.description }}
    </p>

    <div v-else-if="config.showQuery" class="vault-empty__query-line">
      没有找到包含
      <code class="vault-empty__query">{{ searchQuery || '' }}</code>
      的结果。
    </div>

    <button
      v-if="config.showCreate"
      type="button"
      class="vault-empty__cta"
      @click="emit('create')"
    >
      <KvIcon name="add" :size="18" />
      新建条目
    </button>

    <button
      v-if="config.showClear"
      type="button"
      class="vault-empty__clear"
      @click="emit('clearSearch')"
    >
      清除搜索
    </button>
  </div>
</template>

<style scoped>
.vault-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 240px;
  padding: var(--space-8) var(--space-5);
  text-align: center;
}

.vault-empty__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: var(--space-5);
  color: var(--text-tertiary);
}

.vault-empty__icon--circle {
  width: 64px;
  height: 64px;
  border-radius: var(--radius-full);
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  box-shadow: var(--shadow-sm);
}

.vault-empty__icon--compact {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-md);
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  margin-bottom: var(--space-4);
}

.vault-empty__icon-trash {
  margin-bottom: var(--space-5);
  color: var(--text-tertiary);
}

.vault-empty__title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 var(--space-2);
  letter-spacing: -0.01em;
}

.vault-empty__desc {
  font-size: var(--text-md);
  color: var(--text-secondary);
  max-width: 22rem;
  margin: 0 0 var(--space-6);
  line-height: var(--leading-normal);
}

.vault-empty__desc--center {
  margin-bottom: 0;
}

.vault-empty__query-line {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  margin-bottom: var(--space-4);
}

.vault-empty__query {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  padding: 2px var(--space-2);
  margin: 0 var(--space-1);
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
}

.vault-empty__cta {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: 10px var(--space-6);
  font-size: var(--text-md);
  font-weight: 500;
  color: var(--text-primary);
  background: var(--accent-blue);
  border: 1px solid rgba(88, 166, 255, 0.5);
  border-radius: var(--radius-md);
  box-shadow: 0 1px 2px rgba(88, 166, 255, 0.2);
  transition: filter var(--duration-fast);
}

.vault-empty__cta:hover {
  filter: brightness(1.1);
}

.vault-empty__clear {
  margin-top: var(--space-2);
  font-size: var(--text-sm);
  color: var(--accent-blue);
  transition: color var(--duration-fast);
}

.vault-empty__clear:hover {
  color: var(--text-accent);
  text-decoration: underline;
}
</style>
