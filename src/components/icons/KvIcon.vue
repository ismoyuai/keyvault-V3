<script setup lang="ts">
withDefaults(defineProps<{
  name: string
  size?: number
  fill?: boolean
}>(), { size: 20, fill: false })
</script>

<template>
  <span
    class="material-symbols-outlined"
    :class="{ fill }"
    :style="{ fontSize: `${size}px` }"
  >{{ name }}</span>
</template>
