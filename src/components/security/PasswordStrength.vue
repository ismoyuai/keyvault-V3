<script setup lang="ts">
import { computed } from 'vue'
import { usePasswordGenerator } from '@/composables/usePasswordGen'

const props = withDefaults(defineProps<{
  password: string
  variant?: 'bar' | 'segments'
}>(), {
  variant: 'bar',
})

const { evaluateStrength } = usePasswordGenerator()
const strength = computed(() => evaluateStrength(props.password))

const segmentCount = 4
const activeSegments = computed(() => {
  if (!props.password) return 0
  const score = strength.value.score
  if (score <= 2) return 1
  if (score <= 4) return 2
  if (score <= 5) return 3
  return 4
})

const segmentColors = [
  'var(--color-danger)',
  'var(--color-warning)',
  '#ffba42',
  'var(--color-success)',
]

function segmentColor(index: number): string {
  if (index >= activeSegments.value) return 'var(--bg-input)'
  const level = activeSegments.value
  if (level === 1) return segmentColors[0]
  if (level === 2) return segmentColors[1]
  if (level === 3) return segmentColors[2]
  return segmentColors[3]
}

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

const barWidth = computed(() => `${(strength.value.score / 6) * 100}%`)
</script>

<template>
  <div v-if="password" class="strength">
    <template v-if="variant === 'segments'">
      <div class="strength-segments">
        <span
          v-for="i in segmentCount"
          :key="i"
          class="strength-segment"
          :style="{ backgroundColor: segmentColor(i - 1) }"
        />
      </div>
      <span class="strength-label" :style="{ color: segmentColor(activeSegments - 1) }">{{ label }}</span>
    </template>
    <template v-else>
      <div class="strength-track">
        <div
          class="strength-fill"
          :style="{ width: barWidth, backgroundColor: barColor }"
        />
      </div>
      <span class="strength-label" :style="{ color: barColor }">{{ label }}</span>
    </template>
  </div>
</template>

<style scoped>
.strength {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-1);
}

.strength-segments {
  flex: 1;
  display: flex;
  gap: var(--space-1);
}

.strength-segment {
  flex: 1;
  height: 4px;
  border-radius: 2px;
  transition: background-color var(--duration-fast);
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
  min-width: 28px;
  text-align: right;
}
</style>
