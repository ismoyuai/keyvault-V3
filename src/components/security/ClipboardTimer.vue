<script setup lang="ts">
import { ref, onUnmounted, watch } from 'vue'
import KvIcon from '@/components/icons/KvIcon.vue'
import { useClipboard } from '@/composables/useClipboard'

const props = defineProps<{
  /** Total seconds for the countdown */
  seconds: number
}>()

const { clearClipboard } = useClipboard()
const remaining = ref(0)
let intervalId: ReturnType<typeof setInterval> | null = null

function start() {
  stop()
  remaining.value = props.seconds
  intervalId = setInterval(() => {
    remaining.value--
    if (remaining.value <= 0) {
      stop()
      clearClipboard()
    }
  }, 1000)
}

function stop() {
  if (intervalId) {
    clearInterval(intervalId)
    intervalId = null
  }
  remaining.value = 0
}

watch(() => props.seconds, (val) => {
  if (val > 0) start()
  else stop()
})

onUnmounted(stop)

defineExpose({ start, stop, remaining })
</script>

<template>
  <div v-if="remaining > 0" class="clipboard-timer" role="status">
    <KvIcon name="schedule" :size="14" />
    <span class="clipboard-timer-text">剪贴板将在 <strong>{{ remaining }}</strong> 秒后清空</span>
  </div>
</template>

<style scoped>
.clipboard-timer {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1) var(--space-3);
  background: var(--accent-blue-dim);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-full);
  color: var(--text-accent);
  font-size: var(--text-xs);
  font-variant-numeric: tabular-nums;
}

.clipboard-timer-text strong {
  font-weight: 700;
  color: var(--text-primary);
}
</style>
