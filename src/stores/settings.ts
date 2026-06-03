/**
 * 设置状态管理
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { settings as settingsBridge } from '@/bridge/tauri'

export const useSettingsStore = defineStore('settings', () => {
  const autoLockMinutes = ref(15)
  const clipboardClearSeconds = ref(30)
  const theme = ref<'dark' | 'light' | 'system'>('dark')

  async function load() {
    try {
      const autoLock = await settingsBridge.get('auto_lock_minutes')
      if (autoLock) autoLockMinutes.value = parseInt(autoLock, 10)

      const clipClear = await settingsBridge.get('clipboard_clear_seconds')
      if (clipClear) clipboardClearSeconds.value = parseInt(clipClear, 10)

      const savedTheme = await settingsBridge.get('theme')
      if (savedTheme) theme.value = savedTheme as typeof theme.value
    } catch {
      // 使用默认值
    }
  }

  async function setAutoLock(minutes: number) {
    autoLockMinutes.value = minutes
    await settingsBridge.set('auto_lock_minutes', String(minutes))
  }

  async function setClipboardClear(seconds: number) {
    clipboardClearSeconds.value = seconds
    await settingsBridge.set('clipboard_clear_seconds', String(seconds))
  }

  async function setTheme(t: typeof theme.value) {
    theme.value = t
    await settingsBridge.set('theme', t)
  }

  return {
    autoLockMinutes,
    clipboardClearSeconds,
    theme,
    load,
    setAutoLock,
    setClipboardClear,
    setTheme,
  }
})
