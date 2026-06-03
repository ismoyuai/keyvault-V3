<script setup lang="ts">
import { ref } from 'vue'
import { usePasswordGenerator } from '@/composables/usePasswordGen'
import PasswordStrength from '@/components/security/PasswordStrength.vue'
import KvButton from '@/components/ui/KvButton.vue'

const { generate, length, options, excludeAmbiguous } = usePasswordGenerator()

const emit = defineEmits<{
  select: [password: string]
}>()

const password = ref('')
const copied = ref(false)

function generatePassword() {
  password.value = generate()
  copied.value = false
}

async function copyPassword() {
  try {
    await navigator.clipboard.writeText(password.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch {
    // Fallback: emit for parent to handle
  }
}

// Generate on mount
generatePassword()
</script>

<template>
  <div class="password-generator">
    <!-- Generated password -->
    <div class="password-display">
      <code class="password-text">{{ password }}</code>
      <div class="password-actions">
        <KvButton size="sm" variant="ghost" @click="copyPassword">
          {{ copied ? '✓ 已复制' : '复制' }}
        </KvButton>
        <KvButton size="sm" variant="ghost" @click="generatePassword">
          刷新
        </KvButton>
      </div>
    </div>

    <PasswordStrength :password="password" />

    <!-- Options -->
    <div class="options">
      <div class="option-row">
        <label>长度: {{ length }}</label>
        <input v-model.number="length" type="range" min="8" max="128" @input="generatePassword()" />
      </div>
      <div class="option-row">
        <label><input v-model="options.uppercase" type="checkbox" @change="generatePassword()" /> 大写 A-Z</label>
        <label><input v-model="options.lowercase" type="checkbox" @change="generatePassword()" /> 小写 a-z</label>
      </div>
      <div class="option-row">
        <label><input v-model="options.numbers" type="checkbox" @change="generatePassword()" /> 数字 0-9</label>
        <label><input v-model="options.symbols" type="checkbox" @change="generatePassword()" /> 符号 !@#$</label>
      </div>
      <div class="option-row">
        <label><input v-model="excludeAmbiguous" type="checkbox" @change="generatePassword()" /> 排除易混淆字符</label>
      </div>
    </div>

    <!-- Use button -->
    <KvButton variant="primary" @click="emit('select', password)">使用此密码</KvButton>
  </div>
</template>

<style scoped>
.password-generator {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.password-display {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
}
.password-text {
  flex: 1;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  word-break: break-all;
  user-select: all;
}
.password-actions {
  display: flex;
  gap: var(--space-1);
  flex-shrink: 0;
}
.options {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.option-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
.option-row input[type="range"] {
  flex: 1;
  accent-color: var(--accent-blue);
}
.option-row label {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  cursor: pointer;
}
</style>
