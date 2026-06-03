<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Props {
  message: string
  type?: 'info' | 'success' | 'warning' | 'error'
  duration?: number
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  duration: 3000,
})

const emit = defineEmits<{
  close: []
}>()

const visible = ref(false)

onMounted(() => {
  requestAnimationFrame(() => { visible.value = true })
  if (props.duration > 0) {
    setTimeout(() => {
      visible.value = false
      setTimeout(() => emit('close'), 200)
    }, props.duration)
  }
})
</script>

<template>
  <div
    class="kv-toast"
    :class="[`kv-toast--${type}`, { 'kv-toast--visible': visible }]"
  >
    <span class="kv-toast-icon">
      <template v-if="type === 'success'">&#10003;</template>
      <template v-else-if="type === 'error'">&#10007;</template>
      <template v-else-if="type === 'warning'">&#9888;</template>
      <template v-else>&#8505;</template>
    </span>
    <span class="kv-toast-msg">{{ message }}</span>
  </div>
</template>

<style scoped>
.kv-toast {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  background: var(--bg-overlay);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  color: var(--text-primary);
  box-shadow: var(--shadow-md);
  transform: translateY(8px);
  opacity: 0;
  transition: all var(--duration-base) var(--ease-out-quart);
  pointer-events: auto;
}

.kv-toast--visible {
  transform: translateY(0);
  opacity: 1;
}

.kv-toast-icon {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  font-size: 10px;
}

.kv-toast--success .kv-toast-icon { color: var(--color-success); }
.kv-toast--error .kv-toast-icon { color: var(--color-danger); }
.kv-toast--warning .kv-toast-icon { color: var(--color-warning); }
.kv-toast--info .kv-toast-icon { color: var(--color-info); }

.kv-toast-msg {
  flex: 1;
}
</style>
