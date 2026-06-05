<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import KvTransactionalLayout from '@/components/shell/KvTransactionalLayout.vue'
import KvIcon from '@/components/icons/KvIcon.vue'
import PasswordStrength from '@/components/security/PasswordStrength.vue'

const router = useRouter()
const auth = useAuthStore()
const settings = useSettingsStore()

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

function goBack() {
  error.value = ''
  step.value = 1
}

onMounted(() => {
  if (auth.error) error.value = auth.error
})

async function handleSetup() {
  if (!passwordMatch.value) {
    error.value = '密码不一致'
    return
  }
  const success = await auth.setup(password.value)
  if (success) {
    await settings.load()
    router.push('/vault')
  } else {
    error.value = auth.error || '设置失败'
  }
}
</script>

<template>
  <KvTransactionalLayout icon-name="vpn_key">
    <div class="setup-card">
      <!-- Step 1: 创建主密码 -->
      <template v-if="step === 1">
        <div class="setup-card-body">
          <header class="setup-header">
            <h1 class="setup-title">欢迎使用 KeyVault</h1>
            <p class="setup-subtitle">设置您的主密码</p>
          </header>

          <div class="setup-warning" role="note">
            <KvIcon name="warning" :size="20" class="setup-warning-icon" />
            <p class="setup-warning-text">
              主密码无法找回，务必牢记。它是加密您所有数据的唯一凭证。
            </p>
          </div>

          <div class="input-group">
            <label class="field-label" for="setup-password">设置主密码</label>
            <input
              id="setup-password"
              v-model="password"
              type="password"
              placeholder="输入高强度密码"
              class="input input-mono"
              autofocus
              @keydown.enter="nextStep"
            />
            <PasswordStrength v-if="password" :password="password" />
          </div>

          <p v-if="error" class="form-error">{{ error }}</p>

          <button
            type="button"
            class="btn btn-primary btn-block"
            :disabled="!canProceed"
            @click="nextStep"
          >
            <span>继续</span>
            <KvIcon name="arrow_forward" :size="20" class="btn-icon" />
          </button>
        </div>

        <footer class="setup-footer">
          <span class="progress-text">第 1 步，共 2 步</span>
          <div class="progress-dots" aria-hidden="true">
            <span class="dot dot-active" />
            <span class="dot" />
          </div>
        </footer>
      </template>

      <!-- Step 2: 确认主密码 -->
      <template v-else>
        <header class="setup-card-header">
          <h1 class="setup-step-title">确认主密码</h1>
          <p class="setup-step-desc">请再次输入以确认无误</p>
        </header>

        <div class="setup-card-body setup-card-body--compact">
          <div class="input-group">
            <label class="field-label" for="setup-confirm">确认主密码</label>
            <input
              id="setup-confirm"
              v-model="confirmPassword"
              type="password"
              placeholder="再次输入主密码"
              class="input input-mono"
              :class="{ 'input-error': confirmPassword && !passwordMatch }"
              autofocus
              @keydown.enter="handleSetup"
            />
            <div
              v-if="confirmPassword && !passwordMatch"
              class="inline-error"
              role="alert"
            >
              <KvIcon name="error" :size="14" />
              <span>两次密码不一致</span>
            </div>
          </div>

          <p v-if="error" class="form-error">{{ error }}</p>
        </div>

        <footer class="setup-footer setup-footer--actions">
          <span class="progress-text">第 2 步，共 2 步</span>
          <div class="footer-actions">
            <button type="button" class="btn btn-secondary" @click="goBack">
              返回
            </button>
            <button
              type="button"
              class="btn btn-primary"
              :disabled="!canProceed || auth.isLoading"
              @click="handleSetup"
            >
              {{ auth.isLoading ? '创建中...' : '创建密码库' }}
            </button>
          </div>
        </footer>
      </template>
    </div>
  </KvTransactionalLayout>
</template>

<style scoped>
.setup-card {
  display: flex;
  flex-direction: column;
  background: var(--bg-surface);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.setup-card-header {
  padding: var(--space-8) var(--space-10) var(--space-6);
  border-bottom: 1px solid var(--color-outline-variant);
}

.setup-card-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
  padding: var(--space-10);
}

.setup-card-body--compact {
  gap: var(--space-5);
  padding: var(--space-8) var(--space-10);
}

.setup-header {
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.setup-title {
  font-size: var(--text-2xl);
  font-weight: 600;
  line-height: var(--leading-tight);
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.setup-subtitle {
  font-size: var(--text-md);
  color: var(--text-secondary);
}

.setup-warning {
  display: flex;
  gap: var(--space-4);
  align-items: flex-start;
  padding: var(--space-5);
  background: color-mix(in srgb, var(--text-warning) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--text-warning) 30%, transparent);
  border-radius: var(--radius-lg);
}

.setup-warning-icon {
  flex-shrink: 0;
  color: var(--text-warning);
}

.setup-warning-text {
  font-size: var(--text-sm);
  line-height: var(--leading-tight);
  color: var(--text-warning);
}

.setup-step-title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--text-primary);
}

.setup-step-desc {
  font-size: var(--text-md);
  color: var(--text-secondary);
  margin-top: var(--space-1);
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.field-label {
  font-size: var(--text-xs);
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-secondary);
}

.input {
  width: 100%;
  padding: var(--space-4) var(--space-5);
  font-size: var(--text-md);
  background: var(--bg-base);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-md);
  color: var(--text-primary);
}

.input-mono {
  font-family: var(--font-mono);
  font-size: var(--text-base);
}

.input::placeholder {
  color: var(--text-tertiary);
}

.input:focus {
  outline: none;
  border-color: var(--accent-blue);
  box-shadow: 0 0 0 2px var(--accent-blue-dim);
}

.input-error,
.input-error:focus {
  border-color: var(--text-danger);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--text-danger) 20%, transparent);
}

.inline-error {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-sm);
  color: var(--text-danger);
}

.form-error {
  font-size: var(--text-sm);
  color: var(--text-danger);
  text-align: center;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-6);
  border-radius: var(--radius-lg);
  font-size: var(--text-md);
  font-weight: 600;
  transition: background-color var(--duration-fast) var(--ease-out-quart);
}

.btn-primary {
  background: var(--accent-blue);
  color: var(--bg-base);
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: transparent;
  border: 1px solid var(--color-outline-variant);
  color: var(--text-primary);
  font-weight: 500;
}

.btn-secondary:hover {
  background: var(--bg-overlay);
}

.btn-block {
  width: 100%;
  padding: var(--space-5);
}

.btn-icon {
  transition: transform var(--duration-fast) var(--ease-out-quart);
}

.btn-primary:hover:not(:disabled) .btn-icon {
  transform: translateX(2px);
}

.setup-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-5) var(--space-10);
  border-top: 1px solid var(--color-outline-variant);
  background: var(--color-surface-container);
}

.setup-footer--actions {
  gap: var(--space-5);
  flex-wrap: wrap;
}

.progress-text {
  font-size: var(--text-xs);
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-secondary);
}

.progress-dots {
  display: flex;
  gap: var(--space-3);
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
  background: var(--color-outline-variant);
}

.dot-active {
  background: var(--accent-blue);
}

.footer-actions {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  margin-left: auto;
}
</style>
