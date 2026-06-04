<script setup lang="ts">
import { ref } from 'vue'
import KvIcon from '@/components/icons/KvIcon.vue'
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
    // parent may handle
  }
}

generatePassword()
</script>

<template>
  <div class="password-generator">
    <div class="password-display">
      <code class="password-text">{{ password }}</code>
      <div class="password-actions">
        <button type="button" class="icon-btn" :title="copied ? '已复制' : '复制'" @click="copyPassword">
          <KvIcon :name="copied ? 'check' : 'content_copy'" :size="16" />
        </button>
        <button type="button" class="icon-btn" title="重新生成" @click="generatePassword">
          <KvIcon name="autorenew" :size="16" />
        </button>
      </div>
    </div>

    <PasswordStrength :password="password" variant="segments" />

    <div class="options">
      <div class="option-row option-row--length">
        <label>长度 {{ length }}</label>
        <input v-model.number="length" type="range" min="8" max="128" @input="generatePassword()" />
      </div>
      <div class="option-grid">
        <label><input v-model="options.uppercase" type="checkbox" @change="generatePassword()" /> 大写 A-Z</label>
        <label><input v-model="options.lowercase" type="checkbox" @change="generatePassword()" /> 小写 a-z</label>
        <label><input v-model="options.numbers" type="checkbox" @change="generatePassword()" /> 数字 0-9</label>
        <label><input v-model="options.symbols" type="checkbox" @change="generatePassword()" /> 符号</label>
      </div>
      <label class="option-ambiguous">
        <input v-model="excludeAmbiguous" type="checkbox" @change="generatePassword()" />
        排除易混淆字符 (0/O, 1/l)
      </label>
    </div>

    <KvButton variant="primary" class="use-btn" @click="emit('select', password)">
      使用此密码
    </KvButton>
  </div>
</template>

<style scoped>
.password-generator {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.password-display {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  padding: var(--space-3);
  background: var(--bg-base);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-default);
}

.password-text {
  flex: 1;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  word-break: break-all;
  user-select: all;
  color: var(--text-primary);
  line-height: 1.5;
}

.password-actions {
  display: flex;
  gap: var(--space-1);
  flex-shrink: 0;
}

.icon-btn {
  padding: var(--space-1);
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
}

.icon-btn:hover {
  color: var(--text-accent);
  background: var(--bg-elevated);
}

.options {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.option-row--length label {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  display: block;
  margin-bottom: var(--space-1);
}

.option-row--length input[type='range'] {
  width: 100%;
  accent-color: var(--accent-blue);
}

.option-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-2);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.option-grid label,
.option-ambiguous {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  cursor: pointer;
}

.option-ambiguous {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.use-btn {
  width: 100%;
}
</style>
