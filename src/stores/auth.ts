/**
 * 认证状态管理
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { auth as authBridge, setSessionToken, clearSessionToken } from '@/bridge/tauri'

const MAX_UNLOCK_ATTEMPTS = 5
const LOCKOUT_DURATION_MS = 5 * 60 * 1000

function parseUnlockErrorMessage(message: string): {
  failureCount: number
  locked: boolean
} | null {
  const lockedMatch = message.match(/密码错误次数过多（(\d+)次），请5分钟后重试/)
  if (lockedMatch) {
    return {
      failureCount: Number.parseInt(lockedMatch[1], 10),
      locked: true,
    }
  }
  const failMatch = message.match(/密码错误（已失败(\d+)次/)
  if (failMatch) {
    const failureCount = Number.parseInt(failMatch[1], 10)
    return {
      failureCount,
      locked: failureCount >= MAX_UNLOCK_ATTEMPTS,
    }
  }
  return null
}

export const useAuthStore = defineStore('auth', () => {
  const isUnlocked = ref(false)
  const isInitialized = ref(false)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const unlockFailureCount = ref(0)
  const isLockedOut = ref(false)
  const lockoutEndsAt = ref<number | null>(null)

  const remainingAttempts = computed(() =>
    Math.max(0, MAX_UNLOCK_ATTEMPTS - unlockFailureCount.value),
  )

  function resetUnlockLockout() {
    unlockFailureCount.value = 0
    isLockedOut.value = false
    lockoutEndsAt.value = null
  }

  function applyUnlockError(message: string) {
    const parsed = parseUnlockErrorMessage(message)
    if (parsed) {
      unlockFailureCount.value = parsed.failureCount
      if (parsed.locked) {
        isLockedOut.value = true
        lockoutEndsAt.value = Date.now() + LOCKOUT_DURATION_MS
      }
    }
  }

  function applyUnlockStatus(status: {
    failureCount: number
    locked: boolean
    secondsRemaining: number
  }) {
    unlockFailureCount.value = status.failureCount
    if (status.locked && status.secondsRemaining > 0) {
      isLockedOut.value = true
      lockoutEndsAt.value = Date.now() + status.secondsRemaining * 1000
      error.value = `密码错误次数过多（${status.failureCount}次），请5分钟后重试`
    } else if (!status.locked) {
      if (isLockedOut.value) resetUnlockLockout()
    }
  }

  async function syncUnlockStatus() {
    try {
      const status = await authBridge.getUnlockStatus()
      applyUnlockStatus(status)
    } catch {
      // 应用未就绪时忽略
    }
  }

  async function checkInitialized() {
    try {
      isInitialized.value = await authBridge.isInitialized()
      await syncUnlockStatus()
    } catch {
      isInitialized.value = false
    }
  }

  async function setup(password: string): Promise<boolean> {
    isLoading.value = true
    error.value = null
    try {
      const token = await authBridge.setup(password)
      setSessionToken(token)
      isUnlocked.value = true
      isInitialized.value = true
      return true
    } catch (e: any) {
      error.value = e.message || '初始化失败'
      return false
    } finally {
      isLoading.value = false
    }
  }

  async function unlock(password: string): Promise<boolean> {
    if (isLockedOut.value) {
      error.value = `密码错误次数过多（${unlockFailureCount.value}次），请5分钟后重试`
      return false
    }
    isLoading.value = true
    error.value = null
    try {
      const token = await authBridge.unlock(password)
      setSessionToken(token)
      isUnlocked.value = true
      resetUnlockLockout()
      return true
    } catch (e: any) {
      const message = e.message || '密码错误'
      error.value = message
      applyUnlockError(message)
      return false
    } finally {
      isLoading.value = false
    }
  }

  async function lock() {
    try {
      await authBridge.lock()
    } catch {
      // 忽略锁定错误
    }
    clearSessionToken()
    isUnlocked.value = false
    resetUnlockLockout()
  }

  async function changePassword(oldPassword: string, newPassword: string) {
    await authBridge.changePassword(oldPassword, newPassword)
  }

  return {
    isUnlocked,
    isInitialized,
    isLoading,
    error,
    unlockFailureCount,
    isLockedOut,
    lockoutEndsAt,
    remainingAttempts,
    resetUnlockLockout,
    syncUnlockStatus,
    checkInitialized,
    setup,
    unlock,
    lock,
    changePassword,
  }
})
