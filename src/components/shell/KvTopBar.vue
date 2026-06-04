<script setup lang="ts">
import { window as windowBridge } from '@/bridge/tauri'
import KvIcon from '@/components/icons/KvIcon.vue'

withDefaults(defineProps<{
  iconName?: string
}>(), {
  iconName: 'shield_lock',
})
</script>

<template>
  <header class="kv-topbar">
    <div class="kv-topbar-brand">
      <KvIcon :name="iconName" :size="18" fill />
      <span class="kv-topbar-title">KeyVault</span>
    </div>
    <div class="kv-topbar-drag" data-tauri-drag-region />
    <div class="kv-topbar-controls">
      <button
        type="button"
        class="kv-topbar-btn"
        title="最小化"
        @click="windowBridge.minimize()"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
          <line x1="1" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
      <button
        type="button"
        class="kv-topbar-btn"
        title="最大化"
        @click="windowBridge.maximize()"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
          <rect
            x="1.5"
            y="1.5"
            width="9"
            height="9"
            rx="1"
            stroke="currentColor"
            stroke-width="1.2"
            fill="none"
          />
        </svg>
      </button>
      <button
        type="button"
        class="kv-topbar-btn kv-topbar-btn--close"
        title="关闭"
        @click="windowBridge.close()"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
          <path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.kv-topbar {
  display: flex;
  align-items: center;
  height: var(--titlebar-height);
  background: var(--bg-base);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  z-index: var(--z-titlebar);
}

.kv-topbar-brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding-left: var(--space-4);
  -webkit-app-region: no-drag;
}

.kv-topbar-title {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
}

.kv-topbar-drag {
  flex: 1;
  height: 100%;
  -webkit-app-region: drag;
}

.kv-topbar-controls {
  display: flex;
  -webkit-app-region: no-drag;
}

.kv-topbar-btn {
  width: 46px;
  height: var(--titlebar-height);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  transition: background var(--duration-fast);
}

.kv-topbar-btn:hover {
  background: var(--bg-elevated);
}

.kv-topbar-btn--close:hover {
  background: var(--color-danger);
  color: var(--text-primary);
}
</style>
