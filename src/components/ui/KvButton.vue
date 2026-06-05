<script setup lang="ts">
interface Props {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
})
</script>

<template>
  <button
    class="kv-btn"
    :class="[`kv-btn--${variant}`, `kv-btn--${size}`]"
    :disabled="disabled || loading"
  >
    <span v-if="loading" class="kv-btn__spinner" />
    <slot />
  </button>
</template>

<style scoped>
.kv-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  border-radius: var(--radius-md);
  font-weight: 500;
  white-space: nowrap;
  transition: all var(--duration-fast);
  cursor: pointer;
}

.kv-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Sizes */
.kv-btn--sm {
  padding: var(--space-1) var(--space-3);
  font-size: var(--text-xs);
  height: 26px;
}

.kv-btn--md {
  padding: var(--space-2) var(--space-4);
  font-size: var(--text-sm);
  height: 32px;
}

.kv-btn--lg {
  padding: var(--space-3) var(--space-5);
  font-size: var(--text-base);
  height: 38px;
}

/* Variants */
.kv-btn--primary {
  background: var(--accent-blue);
  color: var(--text-primary);
}

.kv-btn--primary:not(:disabled):hover {
  background: var(--accent-blue);
}

.kv-btn--secondary {
  background: var(--bg-input);
  color: var(--text-primary);
  border: 1px solid var(--border-default);
}

.kv-btn--secondary:not(:disabled):hover {
  border-color: var(--border-strong);
  background: var(--bg-overlay);
}

.kv-btn--ghost {
  background: transparent;
  color: var(--text-secondary);
}

.kv-btn--ghost:not(:disabled):hover {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.kv-btn--danger {
  background: var(--color-danger);
  color: var(--text-primary);
}

.kv-btn--danger:not(:disabled):hover {
  opacity: 0.9;
}

/* Spinner */
.kv-btn__spinner {
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
