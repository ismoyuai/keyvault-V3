/**
 * 自动锁定 composable
 * 空闲超时后自动锁定，窗口失焦时清空剪贴板
 * 从旧项目迁移并转换为 TypeScript
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useThrottleFn } from '@vueuse/core'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import { useClipboard } from './useClipboard'

export function useAutoLock() {
  const router = useRouter()
  const auth = useAuthStore()
  const settings = useSettingsStore()
  const { clearClipboard } = useClipboard()
  const timer = ref<ReturnType<typeof setTimeout> | null>(null)

  function resetTimer() {
    if (timer.value) clearTimeout(timer.value)
    const minutes = settings.autoLockMinutes || 15
    timer.value = setTimeout(() => {
      lock()
    }, minutes * 60 * 1000)
  }

  function lock() {
    clearClipboard()
    auth.lock()
    router.push('/login')
  }

  const throttledActivity = useThrottleFn(() => {
    resetTimer()
  }, 1000)

  function handleBlur() {
    clearClipboard()
  }

  onMounted(() => {
    resetTimer()
    document.addEventListener('mousemove', throttledActivity)
    document.addEventListener('keydown', throttledActivity)
    document.addEventListener('click', throttledActivity)
    window.addEventListener('blur', handleBlur)
  })

  onUnmounted(() => {
    if (timer.value) clearTimeout(timer.value)
    document.removeEventListener('mousemove', throttledActivity)
    document.removeEventListener('keydown', throttledActivity)
    document.removeEventListener('click', throttledActivity)
    window.removeEventListener('blur', handleBlur)
  })

  return { lock, resetTimer }
}
