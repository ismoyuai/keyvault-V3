/**
 * 金库状态管理
 * 只存储元数据，绝不缓存解密后的字段值
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { vault as vaultBridge } from '@/bridge/tauri'
import type { EntryMeta } from '@/types/vault'

export const useVaultStore = defineStore('vault', () => {
  const entries = ref<EntryMeta[]>([])
  const selectedId = ref<string | null>(null)
  const isLoading = ref(false)
  const searchQuery = ref('')
  const isTrashView = ref(false)

  async function loadEntries() {
    isTrashView.value = false
    isLoading.value = true
    try {
      entries.value = await vaultBridge.listEntries()
    } catch {
      entries.value = []
    } finally {
      isLoading.value = false
    }
  }

  async function loadTrashEntries() {
    isTrashView.value = true
    searchQuery.value = ''
    isLoading.value = true
    try {
      entries.value = await vaultBridge.listTrashEntries()
    } catch {
      entries.value = []
    } finally {
      isLoading.value = false
    }
  }

  async function search(query: string) {
    searchQuery.value = query
    if (isTrashView.value) {
      if (!query.trim()) {
        await loadTrashEntries()
        return
      }
      const q = query.trim().toLowerCase()
      const all = await vaultBridge.listTrashEntries()
      entries.value = all.filter(
        e =>
          e.title.toLowerCase().includes(q)
          || (e.subtitle?.toLowerCase().includes(q) ?? false),
      )
      return
    }
    if (!query.trim()) {
      await loadEntries()
      return
    }
    isLoading.value = true
    try {
      entries.value = await vaultBridge.searchEntries(query)
    } catch {
      entries.value = []
    } finally {
      isLoading.value = false
    }
  }

  async function toggleFavorite(entryId: string) {
    await vaultBridge.toggleFavorite(entryId)
    const entry = entries.value.find(e => e.id === entryId)
    if (entry) entry.favorited = !entry.favorited
  }

  async function deleteEntry(entryId: string) {
    await vaultBridge.deleteEntry(entryId)
    entries.value = entries.value.filter(e => e.id !== entryId)
    if (selectedId.value === entryId) selectedId.value = null
  }

  async function restoreEntry(entryId: string) {
    await vaultBridge.restoreEntry(entryId)
    entries.value = entries.value.filter(e => e.id !== entryId)
    if (selectedId.value === entryId) selectedId.value = null
  }

  async function purgeEntry(entryId: string) {
    await vaultBridge.purgeEntry(entryId)
    entries.value = entries.value.filter(e => e.id !== entryId)
    if (selectedId.value === entryId) selectedId.value = null
  }

  async function emptyTrash(): Promise<number> {
    const count = await vaultBridge.emptyTrash()
    entries.value = []
    selectedId.value = null
    return count
  }

  function selectEntry(id: string | null) {
    selectedId.value = id
  }

  return {
    entries,
    selectedId,
    isLoading,
    searchQuery,
    isTrashView,
    loadEntries,
    loadTrashEntries,
    search,
    toggleFavorite,
    deleteEntry,
    restoreEntry,
    purgeEntry,
    emptyTrash,
    selectEntry,
  }
})
