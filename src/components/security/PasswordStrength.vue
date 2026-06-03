<script setup lang="ts">
import { computed } from 'vue'
import { usePasswordGenerator } from '@/composables/usePasswordGen'

const props = defineProps<{
  password: string
}>()

const { evaluateStrength } = usePasswordGenerator()
const strength = computed(() => evaluateStrength(props.password))

const barColor = computed(() => {
  const colors: Record<number, string> = {
    0: 'var(--text-disabled)',
    1: 'var(--color-danger)',
    2: 'var(--color-danger)',
    3: 'var(--color-warning)',
    4: 'var(--color-warning)',
    5: 'var(--color-success)',
    6: 'var(--color-success)',
  }
  return colors[strength.value.score] || 'var(--text-disabled)'
})

const label = computed(() => {
  const labels: Record<string, string> = {
    empty: '',
    weak: '弱',
    fair: '一般',
    good: '良好',
    strong: '强',
  }
  return labels[strength.value.level] || ''
})

const barWidth = computed(() => `${(strength.value.score / 10) * 100}%`)
</script>

<template>
  <div v-if="password" class="strength-bar">
    <div class="strength-track">
      <div
        class="strength-fill"
        :style="{ width: barWidth, backgroundColor: barColor }"
      />
    </div>
    <span class="strength-label" :style="{ color: barColor }">{{ label }}</span>
  </div>
</template>

<style scoped>
.strength-bar {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-1);
}
.strength-track {
  flex: 1;
  height: 3px;
  background: var(--bg-input);
  border-radius: 2px;
  overflow: hidden;
}
.strength-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 200ms var(--ease-out-quart), background-color 200ms;
}
.strength-label {
  font-size: var(--text-xs);
  font-weight: 500;
  min-width: 24px;
  text-align: right;
}
</style>
