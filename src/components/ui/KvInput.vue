<script setup lang="ts">
import { ref } from 'vue'

interface Props {
  modelValue?: string
  type?: 'text' | 'password' | 'email' | 'url' | 'number'
  placeholder?: string
  disabled?: boolean
  error?: string
  label?: string
  monospace?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  type: 'text',
  disabled: false,
  monospace: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'keydown': [e: KeyboardEvent]
}>()

const showPassword = ref(false)
const inputType = ref(props.type)

function togglePassword() {
  showPassword.value = !showPassword.value
  inputType.value = showPassword.value ? 'text' : 'password'
}

function handleInput(e: Event) {
  const target = e.target as HTMLInputElement
  emit('update:modelValue', target.value)
}
</script>

<template>
  <div class="kv-input-wrapper">
    <label v-if="label" class="kv-input-label">{{ label }}</label>
    <div class="kv-input-container" :class="{ 'kv-input--error': error }">
      <input
        :value="modelValue"
        :type="type === 'password' ? inputType : type"
        :placeholder="placeholder"
        :disabled="disabled"
        class="kv-input"
        :class="{ 'kv-input--mono': monospace }"
        @input="handleInput"
        @keydown="$emit('keydown', $event)"
      />
      <button
        v-if="type === 'password'"
        class="kv-input-toggle"
        tabindex="-1"
        @click="togglePassword"
      >
        {{ showPassword ? '🙈' : '👁' }}
      </button>
    </div>
    <p v-if="error" class="kv-input-error">{{ error }}</p>
  </div>
</template>

<style scoped>
.kv-input-wrapper {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.kv-input-label {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  font-weight: 500;
}

.kv-input-container {
  position: relative;
  display: flex;
  align-items: center;
}

.kv-input {
  width: 100%;
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-sm);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  outline: none;
  transition: border-color var(--duration-fast), box-shadow var(--duration-fast);
}

.kv-input:focus {
  border-color: var(--accent-blue);
  box-shadow: 0 0 0 2px var(--accent-blue-dim);
}

.kv-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.kv-input--mono {
  font-family: var(--font-mono);
  font-feature-settings: 'liga' 1, 'calt' 1;
  letter-spacing: 0.02em;
}

.kv-input--error .kv-input {
  border-color: var(--color-danger);
}

.kv-input--error .kv-input:focus {
  box-shadow: 0 0 0 2px rgba(248, 81, 73, 0.15);
}

.kv-input-toggle {
  position: absolute;
  right: var(--space-2);
  padding: var(--space-1);
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  opacity: 0.6;
}

.kv-input-toggle:hover {
  opacity: 1;
}

.kv-input-error {
  font-size: var(--text-xs);
  color: var(--text-danger);
}
</style>
