<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import KvAppLayout from '@/components/shell/KvAppLayout.vue'
import KvSettingsNav from '@/components/shell/KvSettingsNav.vue'
import type { SettingsTabId } from '@/components/shell/KvSettingsNav.vue'
import ChangePasswordModal from '@/components/settings/ChangePasswordModal.vue'
import EmergencyWipeModal from '@/components/settings/EmergencyWipeModal.vue'
import { useSettingsStore } from '@/stores/settings'
import { useVaultStore } from '@/stores/vault'
import { useAutoLock } from '@/composables/useAutoLock'
import { useToast } from '@/composables/useToast'
import { security, vault as vaultBridge, sync as syncBridge } from '@/bridge/tauri'
import type { SyncConfig, SyncStatus } from '@/bridge/tauri'

const router = useRouter()
const settings = useSettingsStore()
const vaultStore = useVaultStore()
const { lock } = useAutoLock()
const toast = useToast()

const activeTab = ref<SettingsTabId>('general')
const changePasswordOpen = ref(false)
const wipeOpen = ref(false)

const checkPassword = ref('')
const breachResult = ref<string | null>(null)
const checkingBreach = ref(false)

const syncUrl = ref('')
const syncUsername = ref('')
const syncPassword = ref('')
const syncConfig = ref<SyncConfig | null>(null)
const syncStatus = ref<SyncStatus | null>(null)
const syncLoading = ref(false)
const importing = ref(false)

async function handleBreachCheck() {
  if (!checkPassword.value) return
  checkingBreach.value = true
  breachResult.value = null
  try {
    const result = await security.checkBreach(checkPassword.value)
    breachResult.value = result.breached
      ? `该密码已泄露 ${result.count.toLocaleString()} 次`
      : '该密码未在已知泄露数据库中'
    checkPassword.value = ''
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '未知错误'
    breachResult.value = `检测失败: ${message}`
  } finally {
    checkingBreach.value = false
  }
}

async function handleExport() {
  try {
    const data = await vaultBridge.exportVault('json')
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (path) {
      const { writeTextFile } = await import('@tauri-apps/plugin-fs')
      await writeTextFile(path, data)
      toast.success('导出成功')
    }
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '导出失败'
    toast.error(message)
  }
}

async function handleImport() {
  importing.value = true
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      multiple: false,
    })
    if (!path || typeof path !== 'string') return
    const { readTextFile } = await import('@tauri-apps/plugin-fs')
    const content = await readTextFile(path)
    const count = await vaultBridge.importVault(content, 'json')
    await vaultStore.loadEntries()
    toast.success(`成功导入 ${count} 条记录`)
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '导入失败'
    toast.error(message)
  } finally {
    importing.value = false
  }
}

async function loadSyncConfig() {
  try {
    syncConfig.value = await syncBridge.getConfig()
    syncUrl.value = syncConfig.value.url
    syncUsername.value = syncConfig.value.username
    syncStatus.value = await syncBridge.getStatus()
  } catch {
    // 未配置时忽略
  }
}

async function handleSaveSyncConfig() {
  syncLoading.value = true
  try {
    await syncBridge.setConfig(syncUrl.value, syncUsername.value, syncPassword.value)
    syncPassword.value = ''
    await loadSyncConfig()
    toast.success('同步配置已保存')
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : '保存失败')
  } finally {
    syncLoading.value = false
  }
}

async function handleTestSync() {
  syncLoading.value = true
  try {
    await syncBridge.testConnection()
    toast.success('WebDAV 连接成功')
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : '连接失败')
  } finally {
    syncLoading.value = false
  }
}

async function handleSyncPush() {
  syncLoading.value = true
  try {
    const result = await syncBridge.push()
    await loadSyncConfig()
    toast.success(`已上传 ${result.entriesSent} 条变更`)
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : '上传失败')
  } finally {
    syncLoading.value = false
  }
}

async function handleSyncPull() {
  syncLoading.value = true
  try {
    const result = await syncBridge.pull()
    await vaultStore.loadEntries()
    await loadSyncConfig()
    toast.success(`已合并 ${result.entriesMerged} 条远程记录`)
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : '下载失败')
  } finally {
    syncLoading.value = false
  }
}

function handleNav(id: SettingsTabId) {
  activeTab.value = id
  if (id === 'data') loadSyncConfig()
}

onMounted(() => {
  if (activeTab.value === 'data') loadSyncConfig()
})
</script>

<template>
  <div class="settings-page">
    <KvAppLayout
      active-nav="all"
      @new-entry="router.push('/vault')"
      @settings="() => {}"
      @lock="lock"
      @navigate="router.push('/vault')"
    >
      <div class="settings-layout">
        <KvSettingsNav :active="activeTab" @navigate="handleNav" />
        <div class="settings-content">
          <header class="settings-content-header">
            <h1 class="settings-page-title">
              {{
                activeTab === 'general' ? '常规'
                  : activeTab === 'security' ? '安全'
                    : activeTab === 'data' ? '数据'
                      : '关于'
              }}
            </h1>
            <button type="button" class="back-link" @click="router.push('/vault')">
              返回密码库
            </button>
          </header>

          <div class="settings-panel">
            <section v-if="activeTab === 'general'" class="settings-section">
              <div class="setting-row">
                <div class="setting-label">
                  <span class="label-text">自动锁定</span>
                  <span class="label-hint">空闲后自动锁定密码库</span>
                </div>
                <select
                  class="setting-select"
                  :value="settings.autoLockMinutes"
                  @change="settings.setAutoLock(Number(($event.target as HTMLSelectElement).value))"
                >
                  <option :value="5">5 分钟</option>
                  <option :value="15">15 分钟</option>
                  <option :value="30">30 分钟</option>
                  <option :value="60">60 分钟</option>
                </select>
              </div>
              <p class="setting-note">主题：暗黑模式（v1 已锁定，不可切换）</p>
            </section>

            <section v-if="activeTab === 'security'" class="settings-section">
              <div class="setting-row">
                <div class="setting-label">
                  <span class="label-text">剪贴板自动清空</span>
                  <span class="label-hint">复制敏感内容后倒计时清空</span>
                </div>
                <select
                  class="setting-select"
                  :value="settings.clipboardClearSeconds"
                  @change="settings.setClipboardClear(Number(($event.target as HTMLSelectElement).value))"
                >
                  <option :value="10">10 秒</option>
                  <option :value="30">30 秒</option>
                  <option :value="60">60 秒</option>
                  <option :value="0">禁用</option>
                </select>
              </div>
              <div class="setting-row">
                <div class="setting-label">
                  <span class="label-text">主密码</span>
                  <span class="label-hint">更改解锁密码库的密码</span>
                </div>
                <button type="button" class="btn-secondary" @click="changePasswordOpen = true">
                  修改主密码
                </button>
              </div>
              <div class="setting-row">
                <div class="setting-label">
                  <span class="label-text">紧急擦除</span>
                  <span class="label-hint">永久删除所有本地数据，不可恢复</span>
                </div>
                <button type="button" class="btn-danger-outline" @click="wipeOpen = true">
                  紧急擦除
                </button>
              </div>
              <div class="setting-block">
                <div class="setting-label">
                  <span class="label-text">密码泄露检测 (HIBP)</span>
                  <span class="label-hint">k-匿名查询，密码原文不会离开本机</span>
                </div>
                <div class="breach-row">
                  <input
                    v-model="checkPassword"
                    type="password"
                    class="setting-input"
                    placeholder="输入要检测的密码"
                    autocomplete="off"
                  />
                  <button
                    type="button"
                    class="btn-secondary"
                    :disabled="checkingBreach"
                    @click="handleBreachCheck"
                  >
                    {{ checkingBreach ? '检测中…' : '检测' }}
                  </button>
                </div>
                <p v-if="breachResult" class="breach-result">{{ breachResult }}</p>
              </div>
            </section>

            <section v-if="activeTab === 'data'" class="settings-section">
              <div class="setting-row">
                <div class="setting-label">
                  <span class="label-text">导出密码库</span>
                  <span class="label-hint">解密后导出为 JSON（请妥善保管）</span>
                </div>
                <button type="button" class="btn-secondary" @click="handleExport">导出 JSON</button>
              </div>
              <div class="setting-row">
                <div class="setting-label">
                  <span class="label-text">导入密码库</span>
                  <span class="label-hint">从 JSON 文件导入条目（追加，不覆盖已有）</span>
                </div>
                <button
                  type="button"
                  class="btn-secondary"
                  :disabled="importing"
                  @click="handleImport"
                >
                  {{ importing ? '导入中…' : '导入 JSON' }}
                </button>
              </div>

              <div class="setting-block sync-block">
                <div class="setting-label">
                  <span class="label-text">WebDAV 同步</span>
                  <span class="label-hint">通过私有 NAS 同步加密库（群晖、威联通等）</span>
                </div>
                <div class="sync-form">
                  <input
                    v-model="syncUrl"
                    type="url"
                    class="setting-input sync-input"
                    placeholder="https://nas.example.com:5006"
                  />
                  <input
                    v-model="syncUsername"
                    type="text"
                    class="setting-input sync-input"
                    placeholder="用户名"
                  />
                  <input
                    v-model="syncPassword"
                    type="password"
                    class="setting-input sync-input"
                    :placeholder="syncConfig?.configured ? '留空则保留原密码' : '密码'"
                    autocomplete="off"
                  />
                </div>
                <p v-if="syncStatus?.lastRemoteSync" class="sync-status">
                  远程最后修改：{{ syncStatus.lastRemoteSync }}
                </p>
                <div class="sync-actions">
                  <button
                    type="button"
                    class="btn-secondary"
                    :disabled="syncLoading"
                    @click="handleSaveSyncConfig"
                  >
                    保存配置
                  </button>
                  <button
                    type="button"
                    class="btn-secondary"
                    :disabled="syncLoading"
                    @click="handleTestSync"
                  >
                    测试连接
                  </button>
                  <button
                    type="button"
                    class="btn-secondary"
                    :disabled="syncLoading"
                    @click="handleSyncPush"
                  >
                    上传到 NAS
                  </button>
                  <button
                    type="button"
                    class="btn-secondary"
                    :disabled="syncLoading"
                    @click="handleSyncPull"
                  >
                    从 NAS 下载
                  </button>
                </div>
              </div>
            </section>

            <section v-if="activeTab === 'about'" class="settings-section">
              <div class="about-card">
                <h2>KeyVault</h2>
                <p class="about-version">版本 3.0.0</p>
                <p class="about-desc">
                  本地离线优先的密码与 API Key 管理器。数据使用 AES-256-GCM 字段级加密，
                  主密码经 Argon2id 派生，数据库由 SQLCipher 保护。
                </p>
              </div>
            </section>

            <div class="settings-footer-actions">
              <button type="button" class="btn-danger" @click="lock">锁定密码库</button>
            </div>
          </div>
        </div>
      </div>
    </KvAppLayout>

    <ChangePasswordModal
      :open="changePasswordOpen"
      @close="changePasswordOpen = false"
    />
    <EmergencyWipeModal :open="wipeOpen" @close="wipeOpen = false" />
  </div>
</template>

<style scoped>
.settings-page {
  height: 100vh;
  overflow: hidden;
}

.settings-layout {
  display: flex;
  flex: 1;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}

.settings-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
  background: var(--bg-base);
}

.settings-content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-5) var(--space-6);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.settings-page-title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--text-primary);
}

.back-link {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.back-link:hover {
  color: var(--text-accent);
}

.settings-panel {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
  max-width: 48rem;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-4) 0;
  border-bottom: 1px solid var(--border-subtle);
}

.setting-block {
  padding: var(--space-4) 0;
  border-bottom: 1px solid var(--border-subtle);
}

.setting-label {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.label-text {
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--text-primary);
}

.label-hint {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.setting-select,
.setting-input {
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--text-sm);
  min-width: 140px;
}

.setting-input {
  flex: 1;
  min-width: 0;
}

.setting-note {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  padding: var(--space-3) 0;
}

.breach-row {
  display: flex;
  gap: var(--space-3);
  margin-top: var(--space-3);
  flex-wrap: wrap;
}

.breach-result {
  margin-top: var(--space-2);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.about-card {
  padding: var(--space-5);
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
}

.about-card h2 {
  font-size: var(--text-xl);
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: var(--space-2);
}

.about-version {
  font-size: var(--text-sm);
  color: var(--text-accent);
  margin-bottom: var(--space-4);
}

.about-desc {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.6;
}

.settings-footer-actions {
  margin-top: var(--space-8);
  padding-top: var(--space-4);
}

.btn-secondary {
  padding: var(--space-2) var(--space-4);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--text-sm);
  white-space: nowrap;
}

.btn-secondary:hover {
  border-color: var(--accent-blue);
  color: var(--text-accent);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-danger {
  padding: var(--space-3) var(--space-5);
  background: var(--color-danger);
  color: var(--bg-base);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  font-weight: 600;
}

.btn-danger:hover {
  opacity: 0.9;
}

.btn-danger-outline {
  padding: var(--space-2) var(--space-4);
  background: transparent;
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-md);
  color: var(--color-danger);
  font-size: var(--text-sm);
  white-space: nowrap;
}

.btn-danger-outline:hover {
  background: color-mix(in srgb, var(--color-danger) 12%, transparent);
}

.sync-block {
  margin-top: var(--space-2);
}

.sync-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  margin-top: var(--space-3);
}

.sync-input {
  width: 100%;
  min-width: 0;
}

.sync-status {
  margin-top: var(--space-2);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.sync-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-3);
}
</style>
