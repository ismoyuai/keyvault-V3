<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useDebounceFn, useVirtualList } from '@vueuse/core'
import { useRouter } from 'vue-router'
import { useVaultStore } from '@/stores/vault'
import { useUiStore } from '@/stores/ui'
import { useShortcuts } from '@/composables/useShortcuts'
import { useAutoLock } from '@/composables/useAutoLock'
import { useClipboard } from '@/composables/useClipboard'
import { useToast } from '@/composables/useToast'
import { useSettingsStore } from '@/stores/settings'
import KvAppLayout from '@/components/shell/KvAppLayout.vue'
import type { SideNavId } from '@/components/shell/KvSideNav.vue'
import VaultItem from '@/components/vault/VaultItem.vue'
import VaultEmptyState from '@/components/vault/VaultEmptyState.vue'
import VaultTrashBanner from '@/components/vault/VaultTrashBanner.vue'
import KvIcon from '@/components/icons/KvIcon.vue'
import ItemDetail from '@/components/vault/ItemDetail.vue'
import ItemForm from '@/components/vault/ItemForm.vue'
import CommandPalette from '@/components/search/CommandPalette.vue'
import PasswordGenerator from '@/components/generator/PasswordGenerator.vue'
import ClipboardTimer from '@/components/security/ClipboardTimer.vue'
import KvModal from '@/components/ui/KvModal.vue'
import DeleteConfirmModal from '@/components/vault/DeleteConfirmModal.vue'
import KvToast from '@/components/ui/KvToast.vue'
import type { EntryMeta } from '@/types/vault'

const router = useRouter()
const vault = useVaultStore()
const ui = useUiStore()
const settings = useSettingsStore()
const { lock } = useAutoLock()
const { copy } = useClipboard()
const toast = useToast()

const selectedEntry = ref<EntryMeta | null>(null)
const formOpen = ref(false)
const editEntry = ref<EntryMeta | null>(null)
const activeNav = ref<SideNavId>('all')
const showGenerator = ref(false)
const deleteTarget = ref<EntryMeta | null>(null)
const deleteModalOpen = ref(false)
const deleteModalMode = ref<'soft-delete' | 'permanent'>('soft-delete')
const emptyTrashModalOpen = ref(false)

onMounted(() => {
  vault.loadEntries()
})

useShortcuts([
  { key: 'k', ctrl: true, handler: () => ui.toggleCommandPalette() },
  { key: 'l', ctrl: true, handler: () => lock() },
  { key: 'n', ctrl: true, handler: () => { formOpen.value = true; editEntry.value = null } },
])

function openNewEntry() {
  formOpen.value = true
  editEntry.value = null
}

function selectEntry(entry: EntryMeta) {
  selectedEntry.value = entry
  vault.selectEntry(entry.id)
  ui.openDetailPanel()
}

function handleCopy(entry: EntryMeta) {
  copy(entry.title, 'title')
  toast.success('已复制标题')
}

function requestDelete(entry: EntryMeta) {
  deleteTarget.value = entry
  deleteModalMode.value = 'soft-delete'
  deleteModalOpen.value = true
}

async function confirmDelete() {
  const entry = deleteTarget.value
  if (!entry) return
  try {
    await vault.deleteEntry(entry.id)
    if (selectedEntry.value?.id === entry.id) {
      selectedEntry.value = null
      ui.closeDetailPanel()
    }
    toast.success('已移至回收站')
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '删除失败'
    toast.error(message)
  } finally {
    deleteTarget.value = null
  }
}

function handleEdit(entry: EntryMeta) {
  editEntry.value = entry
  formOpen.value = true
}

function handleSaved() {
  vault.loadEntries()
}

function handleGeneratedPassword(pwd: string) {
  navigator.clipboard.writeText(pwd)
  showGenerator.value = false
  toast.success('密码已复制到剪贴板')
}

const debouncedSearch = useDebounceFn((query: string) => {
  vault.search(query)
}, 300)

function matchesNavFilter(entry: EntryMeta, nav: SideNavId): boolean {
  switch (nav) {
    case 'favorites':
      return entry.favorited
    case 'passwords':
      return entry.entryType !== 'api_key' && entry.entryType !== 'note'
    case 'api-keys':
      return entry.entryType === 'api_key'
    case 'notes':
      return entry.entryType === 'note'
    case 'trash':
      return vault.isTrashView
    default:
      return !vault.isTrashView
  }
}

const filteredEntries = computed(() =>
  vault.entries.filter(e => matchesNavFilter(e, activeNav.value)),
)

const isTrashNav = computed(() => activeNav.value === 'trash')

const trashCount = computed(() =>
  isTrashNav.value ? filteredEntries.value.length : 0,
)

const emptyVariant = computed(() => {
  if (isTrashNav.value) return 'trash-empty' as const
  if (vault.searchQuery.trim()) return 'no-search' as const
  return 'no-items' as const
})

function clearSearch() {
  vault.search('')
}

async function reloadForNav(nav: SideNavId) {
  if (nav === 'trash') {
    await vault.loadTrashEntries()
  } else {
    await vault.loadEntries()
  }
}

watch(activeNav, async (nav) => {
  selectedEntry.value = null
  ui.closeDetailPanel()
  await reloadForNav(nav)
})

async function handleRestore(entry: EntryMeta) {
  try {
    await vault.restoreEntry(entry.id)
    if (selectedEntry.value?.id === entry.id) {
      selectedEntry.value = null
      ui.closeDetailPanel()
    }
    toast.success('已恢复至密码库')
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '恢复失败'
    toast.error(message)
  }
}

function requestPurge(entry: EntryMeta) {
  deleteTarget.value = entry
  deleteModalMode.value = 'permanent'
  deleteModalOpen.value = true
}

async function confirmPurge() {
  const entry = deleteTarget.value
  if (!entry) return
  try {
    await vault.purgeEntry(entry.id)
    if (selectedEntry.value?.id === entry.id) {
      selectedEntry.value = null
      ui.closeDetailPanel()
    }
    toast.success('已永久删除')
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '删除失败'
    toast.error(message)
  } finally {
    deleteTarget.value = null
    deleteModalMode.value = 'soft-delete'
  }
}

async function confirmEmptyTrash() {
  try {
    const count = await vault.emptyTrash()
    selectedEntry.value = null
    ui.closeDetailPanel()
    toast.success(count > 0 ? `已清空回收站（${count} 项）` : '回收站已为空')
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '清空失败'
    toast.error(message)
  } finally {
    emptyTrashModalOpen.value = false
  }
}

function onDeleteModalConfirm() {
  if (deleteModalMode.value === 'permanent') {
    confirmPurge()
  } else {
    confirmDelete()
  }
}

function onDeleteModalClose() {
  deleteModalOpen.value = false
  deleteTarget.value = null
  deleteModalMode.value = 'soft-delete'
}

// 虚拟滚动
const ITEM_HEIGHT = 48
const { list: virtualList, containerProps, wrapperProps } = useVirtualList(
  filteredEntries,
  {
    itemHeight: ITEM_HEIGHT,
    overscan: 5,
  },
)
</script>

<template>
  <div class="vault-view">
    <KvAppLayout
      :active-nav="activeNav"
      class="vault-shell"
      @new-entry="openNewEntry"
      @settings="router.push('/settings')"
      @lock="lock()"
      @navigate="activeNav = $event"
    >
      <div class="vault-pane">
        <main class="entry-list">
          <div v-if="isTrashNav" class="list-title-row">
            <h1 class="list-title">
              回收站
              <span class="list-title-badge">{{ trashCount }}</span>
            </h1>
            <div class="search-wrapper search-wrapper--trash">
              <KvIcon name="search" :size="18" class="search-icon" />
              <input
                v-model="vault.searchQuery"
                type="text"
                class="search-input"
                placeholder="搜索已删除项..."
                @input="debouncedSearch(vault.searchQuery)"
              />
            </div>
            <button
              v-if="trashCount > 0"
              type="button"
              class="trash-empty-btn"
              title="清空回收站"
              @click="emptyTrashModalOpen = true"
            >
              清空回收站
            </button>
          </div>

          <div v-else class="list-header">
            <div class="search-wrapper">
              <KvIcon name="search" :size="14" class="search-icon" />
              <input
                v-model="vault.searchQuery"
                type="text"
                placeholder="搜索条目..."
                class="search-input"
                @input="debouncedSearch(vault.searchQuery)"
              />
            </div>
            <ClipboardTimer :seconds="settings.clipboardClearSeconds" />
            <button
              class="add-btn"
              title="密码生成器"
              @click="showGenerator = true"
            >
              <KvIcon name="auto_awesome" :size="16" />
            </button>
            <button
              class="add-btn"
              title="新建条目 (Ctrl+N)"
              @click="openNewEntry"
            >
              <KvIcon name="add" :size="16" />
            </button>
          </div>

          <VaultTrashBanner v-if="isTrashNav" />

          <div v-bind="containerProps" class="list-body">
            <div v-bind="wrapperProps">
              <VaultItem
                v-for="item in virtualList"
                :key="item.data.id"
                :entry="item.data"
                :selected="selectedEntry?.id === item.data.id"
                @select="selectEntry(item.data)"
                @copy="handleCopy(item.data)"
                @toggle-favorite="vault.toggleFavorite(item.data.id)"
              />
            </div>

            <VaultEmptyState
              v-if="!vault.isLoading && filteredEntries.length === 0"
              :variant="emptyVariant"
              :search-query="vault.searchQuery"
              @create="openNewEntry"
              @clear-search="clearSearch"
            />
          </div>
        </main>

        <ItemDetail
          v-if="ui.detailPanelOpen && selectedEntry"
          :entry="selectedEntry"
          :trash-mode="isTrashNav"
          @close="ui.closeDetailPanel(); selectedEntry = null"
          @edit="handleEdit"
          @delete="requestDelete"
          @restore="handleRestore"
          @purge="requestPurge"
        />
      </div>
    </KvAppLayout>

    <CommandPalette
      :open="ui.commandPaletteOpen"
      @close="ui.commandPaletteOpen = false"
      @select-entry="selectEntry"
      @new-entry="openNewEntry"
    />

    <ItemForm
      :open="formOpen"
      :edit-entry="editEntry"
      @close="formOpen = false; editEntry = null"
      @saved="handleSaved"
    />

    <DeleteConfirmModal
      :open="deleteModalOpen"
      :entry="deleteTarget"
      :mode="deleteModalMode"
      @close="onDeleteModalClose"
      @confirm="onDeleteModalConfirm"
    />

    <KvModal
      :open="emptyTrashModalOpen"
      title="清空回收站"
      width="420px"
      blur
      @close="emptyTrashModalOpen = false"
    >
      <p class="empty-trash-text">
        将永久删除回收站中的全部 {{ trashCount }} 项，此操作不可撤销。
      </p>
      <template #footer>
        <button type="button" class="modal-btn modal-btn--ghost" @click="emptyTrashModalOpen = false">
          取消
        </button>
        <button type="button" class="modal-btn modal-btn--danger" @click="confirmEmptyTrash">
          清空回收站
        </button>
      </template>
    </KvModal>

    <KvModal
      :open="showGenerator"
      title="密码生成器"
      width="480px"
      blur
      @close="showGenerator = false"
    >
      <PasswordGenerator @select="handleGeneratedPassword" />
    </KvModal>

    <div class="toast-container">
      <KvToast
        v-for="t in toast.toasts.value"
        :key="t.id"
        :message="t.message"
        :type="t.type"
        :duration="t.duration"
        @close="toast.remove(t.id)"
      />
    </div>
  </div>
</template>

<style scoped>
.vault-view {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: var(--bg-base);
}

.vault-shell {
  flex: 1;
  min-height: 0;
}

.vault-pane {
  display: flex;
  flex: 1;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}

.entry-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.list-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  height: 64px;
  padding: 0 var(--space-6);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  background: color-mix(in srgb, var(--bg-base) 80%, transparent);
  backdrop-filter: blur(8px);
}

.list-title {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin: 0;
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.list-title-badge {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: 500;
  padding: 2px var(--space-2);
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: var(--text-secondary);
}

.list-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.search-wrapper--trash {
  flex: 0 1 256px;
}

.trash-empty-btn {
  flex-shrink: 0;
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-sm);
  color: var(--text-danger);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  transition: background var(--duration-fast);
}

.trash-empty-btn:hover {
  background: rgba(248, 81, 73, 0.1);
}

.empty-trash-text {
  margin: 0;
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: var(--leading-normal);
}

.modal-btn {
  padding: var(--space-2) var(--space-4);
  font-size: var(--text-sm);
  border-radius: var(--radius-md);
}

.modal-btn--ghost {
  color: var(--text-secondary);
}

.modal-btn--ghost:hover {
  background: var(--bg-elevated);
}

.modal-btn--danger {
  color: var(--text-primary);
  background: var(--color-danger);
}

.modal-btn--danger:hover {
  opacity: 0.9;
}

.search-wrapper {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  transition: border-color var(--duration-fast);
}

.search-wrapper:focus-within {
  border-color: var(--accent-blue);
}

.search-icon {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  font-size: var(--text-sm);
  color: var(--text-primary);
  outline: none;
  padding: 0;
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.add-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  transition: all var(--duration-fast);
  flex-shrink: 0;
}

.add-btn:hover {
  background: var(--accent-blue-dim);
  color: var(--accent-blue);
}

.list-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-2);
}

.toast-container {
  position: fixed;
  bottom: var(--space-5);
  right: var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  z-index: var(--z-toast);
  pointer-events: none;
}
</style>
