<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import { Copy, Eye, EyeOff, Edit, Trash2, ExternalLink } from 'lucide-vue-next'
import { vault as vaultBridge } from '@/bridge/tauri'
import { useClipboard } from '@/composables/useClipboard'
import { useToast } from '@/composables/useToast'
import { useVaultStore } from '@/stores/vault'
import BreachBadge from '@/components/security/BreachBadge.vue'
import type { EntryMeta, EntrySecrets } from '@/types/vault'

interface Props {
  entry: EntryMeta | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  edit: [entry: EntryMeta]
  delete: [entry: EntryMeta]
}>()

const secrets = ref<EntrySecrets | null>(null)
const isLoading = ref(false)
const visibleFields = ref<Set<string>>(new Set())
const { copy, copiedField } = useClipboard()
const toast = useToast()
const vault = useVaultStore()

async function loadSecrets() {
  if (!props.entry) return
  isLoading.value = true
  try {
    secrets.value = await vaultBridge.getEntrySecrets(props.entry.id)
  } catch (e: any) {
    toast.error(e.message || '加载失败')
  } finally {
    isLoading.value = false
  }
}

function toggleVisibility(fieldKey: string) {
  if (visibleFields.value.has(fieldKey)) {
    visibleFields.value.delete(fieldKey)
  } else {
    visibleFields.value.add(fieldKey)
  }
}

async function copyField(value: string, fieldKey: string) {
  await copy(value, fieldKey)
  toast.success('已复制到剪贴板')
}

watch(() => props.entry, (newEntry) => {
  secrets.value = null
  visibleFields.value.clear()
  if (newEntry) loadSecrets()
}, { immediate: true })

onUnmounted(() => {
  secrets.value = null
})
</script>

<template>
  <div class="detail-panel">
    <div class="detail-header">
      <h3 class="detail-title">{{ entry?.title || '条目详情' }}</h3>
      <button class="detail-close" @click="emit('close')">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M1 1l12 12M13 1L1 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div v-if="entry" class="detail-body">
      <div class="detail-meta">
        <span class="detail-type">{{ entry.entryType }}</span>
        <span v-if="entry.subtitle" class="detail-subtitle">{{ entry.subtitle }}</span>
      </div>

      <div v-if="isLoading" class="detail-loading">
        <div class="spinner" />
        <span>解密中...</span>
      </div>

      <div v-else-if="secrets" class="detail-fields">
        <div
          v-for="field in secrets.fields"
          :key="field.fieldKey"
          class="field-row"
        >
          <div class="field-label">{{ field.fieldKey }}</div>
          <div class="field-value-wrapper">
            <template v-if="field.isSensitive">
              <span v-if="visibleFields.has(field.fieldKey)" class="field-value font-mono">
                {{ field.value }}
              </span>
              <span v-else class="field-value field-masked font-mono">
                {{ '•'.repeat(Math.min(field.value.length, 20)) }}
              </span>
            </template>
            <span v-else class="field-value">{{ field.value }}</span>

            <BreachBadge v-if="field.fieldType === 'password'" :password="field.value" />

            <div class="field-actions">
              <button
                v-if="field.isSensitive"
                class="field-action"
                title="显示/隐藏"
                @click="toggleVisibility(field.fieldKey)"
              >
                <Eye v-if="!visibleFields.has(field.fieldKey)" :size="13" />
                <EyeOff v-else :size="13" />
              </button>
              <button
                class="field-action"
                :class="{ 'field-action--copied': copiedField === field.fieldKey }"
                title="复制"
                @click="copyField(field.value, field.fieldKey)"
              >
                <Copy :size="13" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="entry.tags.length > 0" class="detail-tags">
        <span v-for="tag in entry.tags" :key="tag" class="tag">{{ tag }}</span>
      </div>

      <div class="detail-actions">
        <button class="action-btn" @click="emit('edit', entry)">
          <Edit :size="14" />
          <span>编辑</span>
        </button>
        <button class="action-btn action-btn--danger" @click="emit('delete', entry)">
          <Trash2 :size="14" />
          <span>删除</span>
        </button>
      </div>
    </div>

    <div v-else class="detail-empty">
      <p>选择一个条目查看详情</p>
    </div>
  </div>
</template>

<style scoped>
.detail-panel {
  width: var(--detail-panel-width);
  background: var(--bg-surface);
  border-left: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  animation: slideInRight var(--duration-slow) var(--ease-out-quart);
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.detail-title {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
}

.detail-close {
  color: var(--text-tertiary);
  padding: var(--space-1);
  border-radius: var(--radius-sm);
}

.detail-close:hover {
  color: var(--text-primary);
  background: var(--bg-elevated);
}

.detail-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-4);
}

.detail-meta {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  margin-bottom: var(--space-4);
}

.detail-type {
  font-size: var(--text-xs);
  color: var(--text-accent);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.detail-subtitle {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.detail-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-8);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-default);
  border-top-color: var(--accent-blue);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.detail-fields {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.field-row {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.field-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.field-value-wrapper {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  min-height: 34px;
}

.field-value {
  flex: 1;
  font-size: var(--text-sm);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.field-masked {
  color: var(--text-tertiary);
  letter-spacing: 0.1em;
}

.field-actions {
  display: flex;
  gap: var(--space-1);
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--duration-fast);
}

.field-value-wrapper:hover .field-actions {
  opacity: 1;
}

.field-action {
  padding: var(--space-1);
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
}

.field-action:hover {
  color: var(--text-primary);
  background: var(--bg-elevated);
}

.field-action--copied {
  color: var(--color-success);
}

.detail-tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-4);
  padding-top: var(--space-4);
  border-top: 1px solid var(--border-subtle);
}

.tag {
  padding: var(--space-1) var(--space-2);
  background: var(--bg-elevated);
  border-radius: var(--radius-sm);
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.detail-actions {
  display: flex;
  gap: var(--space-2);
  margin-top: var(--space-5);
  padding-top: var(--space-4);
  border-top: 1px solid var(--border-subtle);
}

.action-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  transition: all var(--duration-fast);
}

.action-btn:hover {
  border-color: var(--border-strong);
  color: var(--text-primary);
}

.action-btn--danger:hover {
  border-color: var(--color-danger);
  color: var(--color-danger);
}

.detail-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}
</style>
