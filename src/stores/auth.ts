/**
 * 认证状态管理
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { auth as authBridge, setSessionToken, clearSessionToken } from '@/bridge/tauri'

export const useAuthStore = defineStore('auth', () => {
  const isUnlocked = ref(false)
  const isInitialized = ref(false)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  async function checkInitialized() {
    try {
      isInitialized.value = await authBridge.isInitialized()
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
    isLoading.value = true
    error.value = null
    try {
      const token = await authBridge.unlock(password)
      setSessionToken(token)
      isUnlocked.value = true
      return true
    } catch (e: any) {
      error.value = e.message || '密码错误'
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
  }

  async function changePassword(oldPassword: string, newPassword: string) {
    await authBridge.changePassword(oldPassword, newPassword)
  }

  return {
    isUnlocked,
    isInitialized,
    isLoading,
    error,
    checkInitialized,
    setup,
    unlock,
    lock,
    changePassword,
  }
})
