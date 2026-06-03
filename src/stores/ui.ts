/**
 * UI 状态管理
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUiStore = defineStore('ui', () => {
  const commandPaletteOpen = ref(false)
  const detailPanelOpen = ref(false)
  const sidebarCollapsed = ref(false)

  function toggleCommandPalette() {
    commandPaletteOpen.value = !commandPaletteOpen.value
  }

  function openDetailPanel() {
    detailPanelOpen.value = true
  }

  function closeDetailPanel() {
    detailPanelOpen.value = false
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  return {
    commandPaletteOpen,
    detailPanelOpen,
    sidebarCollapsed,
    toggleCommandPalette,
    openDetailPanel,
    closeDetailPanel,
    toggleSidebar,
  }
})
