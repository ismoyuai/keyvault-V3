<script setup lang="ts">
/**
 * 占位组件 - 待后续实现
 */
</script>

<template>
  <div class="placeholder">
    <slot />
  </div>
</template>

<style scoped>
.placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}
</style>
