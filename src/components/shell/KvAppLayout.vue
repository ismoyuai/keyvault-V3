<script setup lang="ts">
import KvTopBar from '@/components/shell/KvTopBar.vue'
import KvSideNav, { type SideNavId } from '@/components/shell/KvSideNav.vue'

withDefaults(defineProps<{
  activeNav?: SideNavId
}>(), {
  activeNav: 'all',
})

defineEmits<{
  'new-entry': []
  navigate: [id: SideNavId]
  settings: []
  lock: []
}>()
</script>

<template>
  <div class="kv-app-layout">
    <KvTopBar />
    <div class="kv-app-body">
      <KvSideNav
        :active-nav="activeNav"
        @new-entry="$emit('new-entry')"
        @navigate="$emit('navigate', $event)"
        @settings="$emit('settings')"
        @lock="$emit('lock')"
      />
      <main class="kv-app-main">
        <slot />
      </main>
    </div>
  </div>
</template>

<style scoped>
.kv-app-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.kv-app-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.kv-app-main {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}
</style>
