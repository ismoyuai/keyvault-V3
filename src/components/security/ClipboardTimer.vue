<script setup lang="ts">
import { ref, onUnmounted, watch } from 'vue'
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
  <div v-if="remaining > 0" class="clipboard-timer">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <polyline points="12 6 12 12 16 14"/>
    </svg>
    <span>{{ remaining }}s</span>
  </div>
</template>

<style scoped>
.clipboard-timer {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-variant-numeric: tabular-nums;
}
</style>
