<script setup lang="ts">
import { ref, watch } from 'vue'
import { security } from '@/bridge/tauri'

const props = defineProps<{
  password: string
}>()

const breached = ref(false)
const count = ref(0)
const loading = ref(false)
const checked = ref(false)

watch(() => props.password, async (pwd) => {
  if (!pwd || pwd.length < 1) {
    checked.value = false
    return
  }
  loading.value = true
  try {
    const result = await security.checkBreach(pwd)
    breached.value = result.breached
    count.value = result.count
    checked.value = true
  } catch {
    // Silently fail — breach check is non-critical
    checked.value = false
  } finally {
    loading.value = false
  }
}, { immediate: true })
</script>

<template>
  <div v-if="checked && breached" class="breach-badge" :title="`此密码已泄露 ${count.toLocaleString()} 次`">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
      <line x1="12" y1="9" x2="12" y2="13"/>
      <line x1="12" y1="17" x2="12.01" y2="17"/>
    </svg>
    <span>已泄露</span>
  </div>
</template>

<style scoped>
.breach-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  background: rgba(248, 81, 73, 0.15);
  color: var(--color-danger);
  font-size: var(--text-xs);
  font-weight: 500;
}
</style>
