/**
 * 剪贴板 composable
 * 复制后自动定时清空，支持字段级状态追踪
 * 从旧项目迁移并转换为 TypeScript
 */
import { ref, onUnmounted } from 'vue'
import { clipboard } from '@/bridge/tauri'
import { useSettingsStore } from '@/stores/settings'

export function useClipboard() {
  const settings = useSettingsStore()
  const timer = ref<ReturnType<typeof setTimeout> | null>(null)
  const copiedField = ref<string | null>(null)

  async function copy(text: string, fieldName = 'text'): Promise<number> {
    await clipboard.copy(text)
    copiedField.value = fieldName

    if (timer.value) clearTimeout(timer.value)

    const seconds = settings.clipboardClearSeconds || 30
    timer.value = setTimeout(() => {
      clearClipboard()
    }, seconds * 1000)

    return seconds
  }

  function clearClipboard() {
    clipboard.clear()
    copiedField.value = null
    if (timer.value) {
      clearTimeout(timer.value)
      timer.value = null
    }
  }

  onUnmounted(() => {
    if (timer.value) clearTimeout(timer.value)
  })

  return { copy, clearClipboard, copiedField }
}
