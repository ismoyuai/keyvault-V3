/**
 * 全局键盘快捷键 composable
 */
import { onMounted, onUnmounted } from 'vue'

interface ShortcutHandler {
  key: string
  ctrl?: boolean
  meta?: boolean
  shift?: boolean
  handler: (e: KeyboardEvent) => void
}

export function useShortcuts(handlers: ShortcutHandler[]) {
  function handleKeydown(e: KeyboardEvent) {
    for (const h of handlers) {
      const ctrlMatch = h.ctrl ? (e.ctrlKey || e.metaKey) : h.meta ? e.metaKey : true
      const shiftMatch = h.shift ? e.shiftKey : true
      const keyMatch = e.key.toLowerCase() === h.key.toLowerCase()

      if (keyMatch && ctrlMatch && shiftMatch) {
        e.preventDefault()
        h.handler(e)
        return
      }
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })
}
