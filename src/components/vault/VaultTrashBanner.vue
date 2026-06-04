<script setup lang="ts">
import KvIcon from '@/components/icons/KvIcon.vue'
</script>

<template>
  <div class="trash-banner" role="status">
    <KvIcon name="info" :size="18" class="trash-banner__icon" />
    <p class="trash-banner__text">
      软删除 API 尚未接入。当前删除为<strong>永久删除</strong>；回收站列表与恢复功能将在后续版本启用。
    </p>
  </div>
</template>

<style scoped>
.trash-banner {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  margin: 0 var(--space-3) var(--space-2);
  padding: var(--space-3) var(--space-4);
  background: var(--accent-blue-glow);
  border: 1px solid var(--border-default);
  border-left: 2px solid var(--accent-blue);
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.trash-banner__icon {
  color: var(--accent-blue);
  flex-shrink: 0;
  margin-top: 1px;
}

.trash-banner__text {
  margin: 0;
  font-size: var(--text-sm);
  line-height: var(--leading-normal);
  color: var(--text-secondary);
}

.trash-banner__text strong {
  color: var(--text-primary);
  font-weight: 600;
}
</style>
