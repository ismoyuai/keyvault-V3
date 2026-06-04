<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import KvTransactionalLayout from '@/components/shell/KvTransactionalLayout.vue'
import KvIcon from '@/components/icons/KvIcon.vue'

const router = useRouter()
const auth = useAuthStore()
const settings = useSettingsStore()

const password = ref('')
const showPassword = ref(false)
const isShaking = ref(false)
const now = ref(Date.now())
let countdownTimer: ReturnType<typeof setInterval> | null = null

const lockoutRemainingMs = computed(() => {
  if (!auth.lockoutEndsAt) return 0
  return Math.max(0, auth.lockoutEndsAt - now.value)
})

const lockoutCountdown = computed(() => {
  const totalSec = Math.ceil(lockoutRemainingMs.value / 1000)
  const minutes = Math.floor(totalSec / 60)
    .toString()
    .padStart(2, '0')
  const seconds = (totalSec % 60).toString().padStart(2, '0')
  return `${minutes}:${seconds}`
})

const lockoutSubtitle = computed(() => {
  const n = auth.unlockFailureCount
  return `已连续 ${n} 次密码错误，请 5 分钟后再试`
})

function syncLockoutExpiry() {
  now.value = Date.now()
  if (auth.isLockedOut && auth.lockoutEndsAt && lockoutRemainingMs.value <= 0) {
    auth.resetUnlockLockout()
    auth.error = null
  }
}

async function handleUnlock() {
  if (!password.value || auth.isLockedOut) return
  const success = await auth.unlock(password.value)
  if (success) {
    await settings.load()
    router.push('/vault')
  } else {
    if (!auth.isLockedOut) {
      isShaking.value = true
      setTimeout(() => {
        isShaking.value = false
      }, 500)
    }
    password.value = ''
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') handleUnlock()
}

function togglePasswordVisibility() {
  showPassword.value = !showPassword.value
}

onMounted(async () => {
  await auth.syncUnlockStatus()
  countdownTimer = setInterval(syncLockoutExpiry, 1000)
})

onUnmounted(() => {
  if (countdownTimer) clearInterval(countdownTimer)
})
</script>

<template>
  <KvTransactionalLayout icon-name="vpn_key">
    <div class="unlock-wrap">
      <!-- 暴力破解锁定态 -->
      <div v-if="auth.isLockedOut" class="unlock-card unlock-card--lockout">
        <div class="lockout-icon-wrap">
          <KvIcon name="lock" :size="32" fill class="lockout-icon" />
        </div>
        <header class="unlock-header">
          <h1 class="unlock-title">尝试次数过多</h1>
          <p class="unlock-subtitle">{{ lockoutSubtitle }}</p>
        </header>
        <div class="countdown-panel">
          <span class="countdown-label">倒计时</span>
          <span class="countdown-value">{{ lockoutCountdown }}</span>
          <span class="countdown-hint">后可重试</span>
        </div>
        <div class="unlock-form unlock-form--disabled" aria-hidden="true">
          <div class="input-shell">
            <input
              type="password"
              class="input input-mono"
              value="••••••••"
              disabled
            />
            <KvIcon name="visibility_off" :size="18" class="input-trailing-icon" />
          </div>
          <button type="button" class="btn btn-disabled btn-block" disabled>
            解锁
          </button>
        </div>
      </div>

      <!-- 正常解锁 -->
      <div
        v-else
        class="unlock-card"
        :class="{ shake: isShaking }"
      >
        <div class="unlock-hero-icon">
          <KvIcon name="lock_person" :size="40" class="hero-icon" />
        </div>
        <header class="unlock-header">
          <h1 class="unlock-title">保险库已锁定</h1>
          <p class="unlock-subtitle">输入主密码解锁</p>
        </header>

        <form class="unlock-form" @submit.prevent="handleUnlock">
          <div class="input-group">
            <label class="field-label" for="unlock-password">主密码</label>
            <div class="input-shell input-shell--interactive">
              <KvIcon name="key" :size="20" class="input-leading-icon" />
              <input
                id="unlock-password"
                v-model="password"
                :type="showPassword ? 'text' : 'password'"
                placeholder="主密码"
                class="input input-mono input-inset"
                autofocus
                @keydown="handleKeydown"
              />
              <button
                type="button"
                class="visibility-btn"
                :aria-label="showPassword ? '隐藏密码' : '显示密码'"
                @click="togglePasswordVisibility"
              >
                <KvIcon
                  :name="showPassword ? 'visibility_off' : 'visibility'"
                  :size="20"
                />
              </button>
            </div>
            <p
              v-if="auth.unlockFailureCount > 0"
              class="attempts-warning"
              role="alert"
            >
              <KvIcon name="warning" :size="16" />
              <span>剩余重试次数: {{ auth.remainingAttempts }}</span>
            </p>
          </div>

          <button
            type="submit"
            class="btn btn-primary btn-block"
            :disabled="!password || auth.isLoading"
          >
            <KvIcon name="lock_open" :size="22" fill />
            <span>{{ auth.isLoading ? '解锁中...' : '解锁' }}</span>
          </button>

          <p v-if="auth.error && !auth.isLockedOut" class="form-error">
            {{ auth.error }}
          </p>
        </form>

        <footer class="unlock-footer">
          <KvIcon name="shield" :size="16" class="footer-icon" />
          <span>本地 AES-256 加密 · 零知识架构</span>
        </footer>
      </div>

      <p class="unlock-page-footer">AES-256 · 零知识加密架构</p>
    </div>
  </KvTransactionalLayout>
</template>

<style scoped>
.unlock-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  max-width: 480px;
  margin: 0 auto;
}

.unlock-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  padding: var(--space-10);
  background: var(--bg-surface);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
  text-align: center;
}

.unlock-card--lockout {
  gap: var(--space-6);
}

.unlock-hero-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 80px;
  margin-bottom: var(--space-4);
  border-radius: var(--radius-full);
  background: var(--bg-elevated);
  border: 1px solid var(--color-outline-variant);
  box-shadow: inset 0 1px 0 var(--border-subtle);
}

.hero-icon {
  color: var(--text-primary);
  opacity: 0.8;
}

.lockout-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  border-radius: var(--radius-full);
  background: color-mix(in srgb, var(--text-danger) 20%, transparent);
}

.lockout-icon {
  color: var(--text-danger);
}

.unlock-header {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.unlock-title {
  font-size: var(--text-2xl);
  font-weight: 600;
  line-height: var(--leading-tight);
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.unlock-subtitle {
  font-size: var(--text-md);
  color: var(--text-secondary);
}

.countdown-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  padding: var(--space-6);
  background: var(--bg-base);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-lg);
}

.countdown-label,
.countdown-hint {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  color: var(--text-secondary);
}

.countdown-value {
  font-family: var(--font-mono);
  font-size: var(--text-2xl);
  font-weight: 700;
  letter-spacing: 0.15em;
  color: var(--accent-blue);
}

.unlock-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
  width: 100%;
  margin-top: var(--space-4);
}

.unlock-form--disabled {
  opacity: 0.5;
  pointer-events: none;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  text-align: left;
}

.field-label {
  font-size: var(--text-xs);
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-secondary);
}

.input-shell {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  min-height: 56px;
  background: var(--bg-base);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-md);
}

.input-shell--interactive:focus-within {
  border-color: var(--accent-blue);
  box-shadow: 0 0 0 2px var(--accent-blue-dim);
}

.input-leading-icon,
.input-trailing-icon {
  flex-shrink: 0;
  color: var(--text-secondary);
}

.input {
  flex: 1;
  min-width: 0;
  padding: 0;
  font-size: var(--text-md);
  background: transparent;
  border: none;
  color: var(--text-primary);
}

.input-mono {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  letter-spacing: 0.12em;
}

.input-inset:focus {
  outline: none;
}

.input::placeholder {
  color: var(--text-tertiary);
  letter-spacing: normal;
  font-family: var(--font-sans);
}

.visibility-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-2);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  flex-shrink: 0;
}

.visibility-btn:hover {
  color: var(--text-primary);
  background: var(--bg-overlay);
}

.attempts-warning {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-sm);
  color: var(--text-danger);
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
  transition: background-color var(--duration-fast) var(--ease-out-quart),
    transform var(--duration-fast) var(--ease-out-quart);
}

.btn-primary {
  min-height: 56px;
  background: var(--accent-blue);
  color: var(--bg-base);
  box-shadow: 0 4px 14px color-mix(in srgb, var(--accent-blue) 25%, transparent);
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.08);
  transform: translateY(-1px);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.btn-disabled {
  min-height: 48px;
  background: var(--color-outline-variant);
  color: var(--text-secondary);
  cursor: not-allowed;
}

.btn-block {
  width: 100%;
}

.form-error {
  font-size: var(--text-sm);
  color: var(--text-danger);
  text-align: center;
}

.unlock-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  width: 100%;
  margin-top: var(--space-6);
  padding-top: var(--space-6);
  border-top: 1px solid var(--color-outline-variant);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.footer-icon {
  opacity: 0.8;
}

.unlock-page-footer {
  margin-top: var(--space-8);
  font-family: var(--font-mono);
  font-size: var(--text-base);
  color: var(--text-tertiary);
  text-align: center;
}

.shake {
  animation: shake 0.5s ease-in-out;
}

@keyframes shake {
  0%,
  100% {
    transform: translateX(0);
  }
  20%,
  60% {
    transform: translateX(-8px);
  }
  40%,
  80% {
    transform: translateX(8px);
  }
}
</style>
