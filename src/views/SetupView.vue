<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const step = ref(1)
const password = ref('')
const confirmPassword = ref('')
const error = ref('')

const passwordMatch = computed(() => password.value === confirmPassword.value)
const canProceed = computed(() => {
  if (step.value === 1) return password.value.length >= 8
  return passwordMatch.value && confirmPassword.value.length > 0
})

function nextStep() {
  if (step.value === 1) {
    if (password.value.length < 8) {
      error.value = '密码至少 8 个字符'
      return
    }
    error.value = ''
    step.value = 2
  }
}

async function handleSetup() {
  if (!passwordMatch.value) {
    error.value = '密码不一致'
    return
  }
  const success = await auth.setup(password.value)
  if (success) {
    router.push('/vault')
  } else {
    error.value = auth.error || '设置失败'
  }
}
</script>

<template>
  <div class="setup-view">
    <div class="setup-card">
      <h1 class="title">欢迎使用 KeyVault</h1>
      <p class="subtitle">设置主密码以保护你的数据</p>

      <div v-if="step === 1" class="step">
        <div class="input-group">
          <label class="label">主密码</label>
          <input
            v-model="password"
            type="password"
            placeholder="至少 8 个字符"
            class="input"
            autofocus
            @keydown.enter="nextStep"
          />
        </div>
        <button class="btn" :disabled="!canProceed" @click="nextStep">
          下一步
        </button>
      </div>

      <div v-else class="step">
        <div class="input-group">
          <label class="label">确认密码</label>
          <input
            v-model="confirmPassword"
            type="password"
            placeholder="再次输入密码"
            class="input"
            autofocus
            @keydown.enter="handleSetup"
          />
        </div>
        <p v-if="!passwordMatch && confirmPassword" class="hint error">
          密码不一致
        </p>
        <button
          class="btn"
          :disabled="!canProceed || auth.isLoading"
          @click="handleSetup"
        >
          {{ auth.isLoading ? '创建中...' : '创建密码库' }}
        </button>
        <button class="btn-link" @click="step = 1">返回</button>
      </div>

      <p v-if="error" class="error">{{ error }}</p>
    </div>
  </div>
</template>

<style scoped>
.setup-view {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: var(--bg-base);
}

.setup-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  padding: var(--space-8);
  width: 360px;
}

.title {
  font-size: var(--text-xl);
  font-weight: 600;
  text-align: center;
}

.subtitle {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  text-align: center;
}

.step {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.label {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.input {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  font-size: var(--text-md);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
}

.input:focus {
  border-color: var(--accent-blue);
  box-shadow: 0 0 0 2px var(--accent-blue-dim);
}

.btn {
  width: 100%;
  padding: var(--space-3);
  background: var(--accent-blue);
  color: #fff;
  border-radius: var(--radius-md);
  font-size: var(--text-base);
  font-weight: 500;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-link {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  text-align: center;
}

.btn-link:hover {
  color: var(--text-accent);
}

.error {
  color: var(--text-danger);
  font-size: var(--text-sm);
  text-align: center;
}
</style>
