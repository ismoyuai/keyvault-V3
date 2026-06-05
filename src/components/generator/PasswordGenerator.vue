<script setup lang="ts">
import { ref } from 'vue'
import KvIcon from '@/components/icons/KvIcon.vue'
import { usePasswordGenerator } from '@/composables/usePasswordGen'
import PasswordStrength from '@/components/security/PasswordStrength.vue'
import KvButton from '@/components/ui/KvButton.vue'
import { security } from '@/bridge/tauri'
import { useToast } from '@/composables/useToast'
import type { PasswordOptions } from '@/types/vault'

const { length, options, excludeAmbiguous, generate: generateLocal } = usePasswordGenerator()
const toast = useToast()

const emit = defineEmits<{
  select: [password: string]
}>()

const password = ref('')
const copied = ref(false)
const mode = ref<'random' | 'diceware'>('random')
const generating = ref(false)

async function generatePassword() {
  generating.value = true
  try {
    const wordCount = Math.max(6, Math.min(length.value, 12))
    const opts: PasswordOptions = {
      length: mode.value === 'diceware' ? wordCount : length.value,
      uppercase: options.value.uppercase,
      lowercase: options.value.lowercase,
      numbers: options.value.numbers,
      symbols: options.value.symbols,
      excludeAmbiguous: excludeAmbiguous.value,
      mode: mode.value,
    }
    password.value = await security.generatePassword(opts)
    copied.value = false
  } catch (e: unknown) {
    if (mode.value === 'diceware') {
      const message = e instanceof Error ? e.message : 'Diceware 生成失败'
      toast.error(message)
      return
    }
    password.value = generateLocal()
    copied.value = false
  } finally {
    generating.value = false
  }
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
    <div class="mode-tabs">
      <button
        type="button"
        class="mode-tab"
        :class="{ active: mode === 'random' }"
        @click="mode = 'random'; generatePassword()"
      >
        随机密码
      </button>
      <button
        type="button"
        class="mode-tab"
        :class="{ active: mode === 'diceware' }"
        @click="mode = 'diceware'; generatePassword()"
      >
        Diceware 词组
      </button>
    </div>

    <div class="password-display">
      <code class="password-text">{{ password }}</code>
      <div class="password-actions">
        <button type="button" class="icon-btn" :title="copied ? '已复制' : '复制'" @click="copyPassword">
          <KvIcon :name="copied ? 'check' : 'content_copy'" :size="16" />
        </button>
        <button
          type="button"
          class="icon-btn"
          title="重新生成"
          :disabled="generating"
          @click="generatePassword"
        >
          <KvIcon name="autorenew" :size="16" />
        </button>
      </div>
    </div>

    <PasswordStrength v-if="mode === 'random'" :password="password" variant="segments" />

    <div class="options">
      <div class="option-row option-row--length">
        <label>{{ mode === 'diceware' ? `词数 ${Math.max(6, Math.min(length, 12))}` : `长度 ${length}` }}</label>
        <input
          v-model.number="length"
          type="range"
          :min="mode === 'diceware' ? 6 : 8"
          :max="mode === 'diceware' ? 12 : 128"
          @input="generatePassword()"
        />
      </div>
      <template v-if="mode === 'random'">
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
      </template>
      <p v-else class="diceware-hint">
        256 词 EFF 子集，建议 6–12 词（约 48–96 bit 熵）
      </p>
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

.mode-tabs {
  display: flex;
  gap: var(--space-1);
  padding: var(--space-1);
  background: var(--bg-base);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-default);
}

.mode-tab {
  flex: 1;
  padding: var(--space-2);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast);
}

.mode-tab.active {
  background: var(--bg-elevated);
  color: var(--text-accent);
  font-weight: 600;
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

.diceware-hint {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  line-height: 1.5;
}

.use-btn {
  width: 100%;
}
</style>
