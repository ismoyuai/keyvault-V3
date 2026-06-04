<script setup lang="ts">
import { ref, watch, onUnmounted, withDefaults } from 'vue'
import { vault as vaultBridge } from '@/bridge/tauri'
import { useClipboard } from '@/composables/useClipboard'
import { useToast } from '@/composables/useToast'
import KvIcon from '@/components/icons/KvIcon.vue'
import BreachBadge from '@/components/security/BreachBadge.vue'
import type { DecryptedField, EntryMeta, EntrySecrets } from '@/types/vault'

interface Props {
  entry: EntryMeta | null
  trashMode?: boolean
}

const props = withDefaults(defineProps<Props>(), { trashMode: false })

const emit = defineEmits<{
  close: []
  edit: [entry: EntryMeta]
  delete: [entry: EntryMeta]
  restore: [entry: EntryMeta]
  purge: [entry: EntryMeta]
}>()

const FIELD_LABELS: Record<string, string> = {
  username: '用户名',
  password: '密码',
  url: '网址',
  api_key: 'API Key',
  notes: '内容',
  public_key: '公钥',
  private_key: '私钥',
  email: '邮箱',
  phone: '电话',
  address: '地址',
  host: '主机',
  port: '端口',
  ssh_key: 'SSH Key',
  expiry_date: '有效期',
  cvv: 'CVV',
  mnemonic: '助记词',
  rate_limit: 'Rate Limit',
}

const secrets = ref<EntrySecrets | null>(null)
const isLoading = ref(false)
const visibleFields = ref<Set<string>>(new Set())
const { copy, copiedField } = useClipboard()
const toast = useToast()

function fieldLabel(fieldKey: string): string {
  return FIELD_LABELS[fieldKey] ?? fieldKey
}

function isUrlField(field: DecryptedField): boolean {
  return field.fieldType === 'url' || field.fieldKey === 'url'
}

function isPasswordField(field: DecryptedField): boolean {
  return field.fieldType === 'password'
}

function maskedValue(field: DecryptedField): string {
  const len = Math.min(field.value.length, 20)
  return '•'.repeat(Math.max(len, 8))
}

async function loadSecrets() {
  if (!props.entry) return
  isLoading.value = true
  try {
    secrets.value = await vaultBridge.getEntrySecrets(props.entry.id)
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '加载失败'
    toast.error(message)
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

function openUrl(raw: string) {
  const url = /^https?:\/\//i.test(raw) ? raw : `https://${raw}`
  window.open(url, '_blank', 'noopener,noreferrer')
}

watch(
  () => props.entry,
  (newEntry) => {
    secrets.value = null
    visibleFields.value.clear()
    if (newEntry) loadSecrets()
  },
  { immediate: true },
)

onUnmounted(() => {
  secrets.value = null
})
</script>

<template>
  <div class="detail-panel">
    <div class="detail-header">
      <h3 class="detail-title">{{ entry?.title || '条目详情' }}</h3>
      <button type="button" class="detail-close" title="关闭" @click="emit('close')">
        <KvIcon name="close" :size="16" />
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
          class="field-group"
        >
          <div class="field-label">{{ fieldLabel(field.fieldKey) }}</div>
          <div class="secure-field">
            <template v-if="field.fieldType === 'textarea'">
              <pre
                v-if="field.isSensitive && !visibleFields.has(field.fieldKey)"
                class="field-value field-value--textarea font-mono field-value--masked"
              >{{ maskedValue(field) }}</pre>
              <pre
                v-else
                class="field-value field-value--textarea font-mono"
              >{{ field.value }}</pre>
            </template>
            <template v-else-if="field.isSensitive">
              <span
                v-if="visibleFields.has(field.fieldKey)"
                class="field-value font-mono"
                :class="{ 'field-value--password': isPasswordField(field) }"
              >{{ field.value }}</span>
              <span
                v-else
                class="field-value font-mono field-value--masked"
                :class="{ 'field-value--password': isPasswordField(field) }"
              >{{ maskedValue(field) }}</span>
            </template>
            <span v-else class="field-value" :class="{ 'font-mono': field.isSensitive }">
              {{ field.value }}
            </span>

            <BreachBadge v-if="field.fieldType === 'password'" :password="field.value" />

            <div class="field-actions">
              <button
                v-if="field.isSensitive"
                type="button"
                class="field-action"
                title="显示/隐藏"
                @click="toggleVisibility(field.fieldKey)"
              >
                <KvIcon
                  :name="visibleFields.has(field.fieldKey) ? 'visibility_off' : 'visibility'"
                  :size="16"
                />
              </button>
              <button
                v-if="isUrlField(field)"
                type="button"
                class="field-action"
                title="在浏览器中打开"
                @click="openUrl(field.value)"
              >
                <KvIcon name="open_in_new" :size="16" />
              </button>
              <button
                type="button"
                class="field-action"
                :class="{ 'field-action--copied': copiedField === field.fieldKey }"
                title="复制"
                @click="copyField(field.value, field.fieldKey)"
              >
                <KvIcon name="content_copy" :size="16" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="entry.tags.length > 0" class="detail-tags">
        <span v-for="tag in entry.tags" :key="tag" class="tag">{{ tag }}</span>
      </div>

      <div class="detail-actions">
        <template v-if="trashMode">
          <button type="button" class="action-btn" @click="emit('restore', entry)">
            <KvIcon name="restore" :size="16" />
            <span>恢复</span>
          </button>
          <button type="button" class="action-btn action-btn--danger" @click="emit('purge', entry)">
            <KvIcon name="delete_forever" :size="16" />
            <span>永久删除</span>
          </button>
        </template>
        <template v-else>
          <button type="button" class="action-btn" @click="emit('edit', entry)">
            <KvIcon name="edit" :size="16" />
            <span>编辑</span>
          </button>
          <button type="button" class="action-btn action-btn--danger" @click="emit('delete', entry)">
            <KvIcon name="delete" :size="16" />
            <span>删除</span>
          </button>
        </template>
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
  display: flex;
  align-items: center;
  justify-content: center;
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
  gap: var(--space-4);
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.field-label {
  font-family: var(--font-sans);
  font-size: 11px;
  font-weight: 700;
  line-height: 16px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.secure-field {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  min-height: 34px;
  transition:
    border-color var(--duration-fast),
    box-shadow var(--duration-fast);
}

.secure-field:focus-within {
  border-color: var(--border-accent);
  box-shadow: 0 0 0 2px var(--accent-blue-dim);
}

.field-value {
  flex: 1;
  font-size: var(--text-base);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.field-value--textarea {
  white-space: pre-wrap;
  word-break: break-all;
  overflow-x: auto;
  margin: 0;
  font-size: var(--text-base);
  line-height: var(--leading-normal);
}

.field-value--masked {
  color: var(--text-tertiary);
}

.field-value--password.field-value--masked {
  letter-spacing: 0.2em;
}

.font-mono {
  font-family: var(--font-mono);
}

.field-actions {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--duration-fast);
}

.field-group:hover .field-actions,
.secure-field:focus-within .field-actions {
  opacity: 1;
}

.field-action {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-1);
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
}

.field-action:hover {
  color: var(--text-accent);
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
