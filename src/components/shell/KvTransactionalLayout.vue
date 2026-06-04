<script setup lang="ts">
import KvTopBar from '@/components/shell/KvTopBar.vue'

withDefaults(defineProps<{
  iconName?: string
}>(), {
  iconName: 'shield_lock',
})
</script>

<template>
  <div class="kv-transactional-layout">
    <KvTopBar :icon-name="iconName" />
    <div class="kv-transactional-body">
      <div class="kv-transactional-content">
        <slot />
      </div>
    </div>
  </div>
</template>

<style scoped>
.kv-transactional-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.kv-transactional-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  justify-content: center;
  padding: var(--space-8) var(--space-5);
}

.kv-transactional-content {
  width: 100%;
  max-width: 560px;
}
</style>
