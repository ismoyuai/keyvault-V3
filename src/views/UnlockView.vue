<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()
const password = ref('')
const isShaking = ref(false)

async function handleUnlock() {
  if (!password.value) return
  const success = await auth.unlock(password.value)
  if (success) {
    router.push('/vault')
  } else {
    isShaking.value = true
    setTimeout(() => { isShaking.value = false }, 500)
    password.value = ''
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') handleUnlock()
}
</script>

<template>
  <div class="unlock-view">
    <div class="unlock-card" :class="{ shake: isShaking }">
      <div class="logo">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
          <rect width="48" height="48" rx="12" fill="var(--accent-blue-dim)" />
          <path
            d="M24 14a8 8 0 0 0-8 8v4h-2a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h20a2 2 0 0 0 2-2V28a2 2 0 0 0-2-2h-2v-4a8 8 0 0 0-8-8zm-4 12v-4a4 4 0 1 1 8 0v4h-8zm6 8a2 2 0 1 1-4 0 2 2 0 0 1 4 0z"
            fill="var(--accent-blue)"
          />
        </svg>
      </div>
      <h1 class="title">KeyVault</h1>
      <p class="subtitle">输入主密码解锁</p>
      <div class="input-group">
        <input
          v-model="password"
          type="password"
          placeholder="主密码"
          class="password-input"
          autofocus
          @keydown="handleKeydown"
        />
      </div>
      <button
        class="unlock-btn"
        :disabled="!password || auth.isLoading"
        @click="handleUnlock"
      >
        {{ auth.isLoading ? '解锁中...' : '解锁' }}
      </button>
      <p v-if="auth.error" class="error">{{ auth.error }}</p>
    </div>
  </div>
</template>

<style scoped>
.unlock-view {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: var(--bg-base);
}

.unlock-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-8);
  width: 320px;
}

.logo {
  margin-bottom: var(--space-2);
}

.title {
  font-size: var(--text-xl);
  font-weight: 600;
  color: var(--text-primary);
}

.subtitle {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.input-group {
  width: 100%;
}

.password-input {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  font-size: var(--text-md);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  text-align: center;
  letter-spacing: 0.1em;
}

.password-input:focus {
  border-color: var(--accent-blue);
  box-shadow: 0 0 0 2px var(--accent-blue-dim);
}

.unlock-btn {
  width: 100%;
  padding: var(--space-3);
  background: var(--accent-blue);
  color: #fff;
  border-radius: var(--radius-md);
  font-size: var(--text-base);
  font-weight: 500;
  transition: opacity var(--duration-fast);
}

.unlock-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.unlock-btn:not(:disabled):hover {
  opacity: 0.9;
}

.error {
  color: var(--text-danger);
  font-size: var(--text-sm);
}

.shake {
  animation: shake 0.5s ease-in-out;
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  20%, 60% { transform: translateX(-8px); }
  40%, 80% { transform: translateX(8px); }
}
</style>
