<script setup lang="ts">
import { watch, onUnmounted } from 'vue'

interface Props {
  open: boolean
  title?: string
  width?: string
  /** UNIFIED-SPEC §2.5：模态遮罩 blur */
  blur?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  blur: false,
})

const emit = defineEmits<{
  close: []
}>()

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    document.addEventListener('keydown', handleKeydown)
  } else {
    document.removeEventListener('keydown', handleKeydown)
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="open"
        class="kv-modal-overlay"
        :class="{ 'kv-modal-overlay--blur': blur }"
        @click.self="emit('close')"
      >
        <div class="kv-modal" :style="{ maxWidth: width || '480px' }">
          <div v-if="title || $slots.header" class="kv-modal-header">
            <slot name="header">
              <h3 class="kv-modal-title">{{ title }}</h3>
            </slot>
            <button class="kv-modal-close" @click="emit('close')">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                <path d="M1 1l12 12M13 1L1 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </button>
          </div>
          <div class="kv-modal-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="kv-modal-footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.kv-modal-overlay {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--bg-base) 80%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  padding: var(--space-5);
}

.kv-modal-overlay--blur {
  backdrop-filter: blur(8px);
}

.kv-modal {
  width: 100%;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  max-height: 80vh;
}

.kv-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.kv-modal-title {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
}

.kv-modal-close {
  color: var(--text-tertiary);
  padding: var(--space-1);
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast);
}

.kv-modal-close:hover {
  color: var(--text-primary);
  background: var(--bg-elevated);
}

.kv-modal-body {
  padding: var(--space-5);
  overflow-y: auto;
  flex: 1;
}

.kv-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-4) var(--space-5);
  border-top: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

/* Transitions */
.modal-enter-active {
  transition: opacity var(--duration-base) var(--ease-out-quart);
}

.modal-leave-active {
  transition: opacity var(--duration-fast);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .kv-modal {
  transition: transform var(--duration-base) var(--ease-spring), opacity var(--duration-base);
}

.modal-leave-active .kv-modal {
  transition: transform var(--duration-fast), opacity var(--duration-fast);
}

.modal-enter-from .kv-modal {
  transform: scale(0.95);
  opacity: 0;
}

.modal-leave-to .kv-modal {
  transform: scale(0.98);
  opacity: 0;
}
</style>
