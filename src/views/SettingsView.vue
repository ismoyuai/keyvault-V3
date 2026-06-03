<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import { useAutoLock } from '@/composables/useAutoLock'
import { useToast } from '@/composables/useToast'
import { security, vault as vaultBridge } from '@/bridge/tauri'

const router = useRouter()
const auth = useAuthStore()
const settings = useSettingsStore()
const { lock } = useAutoLock()
const toast = useToast()

const showChangePassword = ref(false)
const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const changingPassword = ref(false)

async function handleChangePassword() {
  if (newPassword.value !== confirmPassword.value) {
    toast.error('两次密码不一致')
    return
  }
  if (newPassword.value.length < 8) {
    toast.error('密码长度至少 8 位')
    return
  }
  changingPassword.value = true
  try {
    await auth.changePassword(oldPassword.value, newPassword.value)
    toast.success('密码已修改')
    showChangePassword.value = false
  } catch (e: any) {
    toast.error(e.message || '修改失败')
  } finally {
    changingPassword.value = false
    oldPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
  }
}

const checkPassword = ref('')
const breachResult = ref<string | null>(null)
const checkingBreach = ref(false)

async function handleBreachCheck() {
  if (!checkPassword.value) return
  checkingBreach.value = true
  breachResult.value = null
  try {
    const result = await security.checkBreach(checkPassword.value)
    breachResult.value = result.breached
      ? `该密码已泄露 ${result.count.toLocaleString()} 次`
      : '该密码未在已知泄露数据库中'
  } catch (e: any) {
    breachResult.value = '检测失败: ' + (e.message || '未知错误')
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
  } catch (e: any) {
    toast.error(e.message || '导出失败')
  }
}
</script>

<template>
  <div class="settings-view">
    <div class="settings-header">
      <button class="back-btn" @click="router.push('/vault')">← 返回</button>
      <h1>设置</h1>
    </div>

    <div class="settings-body">
      <section class="settings-section">
        <h2 class="section-title">安全</h2>
        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">自动锁定</span>
            <span class="label-hint">空闲后自动锁定（分钟）</span>
          </div>
          <select :value="settings.autoLockMinutes" class="setting-select" @change="settings.setAutoLock(Number(($event.target as HTMLSelectElement).value))">
            <option :value="5">5 分钟</option>
            <option :value="15">15 分钟</option>
            <option :value="30">30 分钟</option>
            <option :value="60">60 分钟</option>
          </select>
        </div>
        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">剪贴板自动清空</span>
            <span class="label-hint">复制后自动清空（秒）</span>
          </div>
          <select :value="settings.clipboardClearSeconds" class="setting-select" @change="settings.setClipboardClear(Number(($event.target as HTMLSelectElement).value))">
            <option :value="10">10 秒</option>
            <option :value="30">30 秒</option>
            <option :value="60">60 秒</option>
            <option :value="0">禁用</option>
          </select>
        </div>
        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">修改密码</span>
            <span class="label-hint">更改主密码</span>
          </div>
          <button class="btn-secondary" @click="showChangePassword = !showChangePassword">
            {{ showChangePassword ? '取消' : '修改' }}
          </button>
        </div>

        <div v-if="showChangePassword" class="change-password-form">
          <input
            v-model="oldPassword"
            type="password"
            placeholder="当前密码"
            class="password-input"
          />
          <input
            v-model="newPassword"
            type="password"
            placeholder="新密码（至少 8 位）"
            class="password-input"
          />
          <input
            v-model="confirmPassword"
            type="password"
            placeholder="确认新密码"
            class="password-input"
          />
          <button
            class="btn-primary"
            :disabled="changingPassword"
            @click="handleChangePassword"
          >
            {{ changingPassword ? '修改中...' : '确认修改' }}
          </button>
        </div>
        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">密码泄露检测</span>
            <span class="label-hint">检查密码是否在已知泄露数据库中</span>
          </div>
        </div>
        <div class="breach-check-form">
          <input
            v-model="checkPassword"
            type="password"
            placeholder="输入要检测的密码"
            class="password-input"
          />
          <button
            class="btn-secondary"
            :disabled="checkingBreach"
            @click="handleBreachCheck"
          >
            {{ checkingBreach ? '检测中...' : '检测' }}
          </button>
          <p v-if="breachResult" class="breach-result">{{ breachResult }}</p>
        </div>
      </section>

      <section class="settings-section">
        <h2 class="section-title">关于</h2>
        <div class="setting-row">
          <span class="label-text">版本</span>
          <span class="label-hint">3.0.0</span>
        </div>
      </section>

      <section class="settings-section">
        <h2 class="section-title">数据</h2>
        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">导出密码库</span>
            <span class="label-hint">导出为 JSON 文件</span>
          </div>
          <button class="btn-secondary" @click="handleExport">导出</button>
        </div>
      </section>

      <div class="settings-actions">
        <button class="btn-danger" @click="lock">锁定</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  height: 100vh;
  background: var(--bg-base);
  overflow-y: auto;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-5);
  border-bottom: 1px solid var(--border-subtle);
}

.settings-header h1 {
  font-size: var(--text-lg);
  font-weight: 600;
}

.back-btn {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.back-btn:hover {
  color: var(--text-accent);
}

.settings-body {
  padding: var(--space-5);
  max-width: 600px;
}

.settings-section {
  margin-bottom: var(--space-6);
}

.section-title {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-4);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) 0;
  border-bottom: 1px solid var(--border-subtle);
}

.setting-label {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.label-text {
  font-size: var(--text-base);
  color: var(--text-primary);
}

.label-hint {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.setting-select {
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.settings-actions {
  margin-top: var(--space-8);
}

.btn-danger {
  padding: var(--space-3) var(--space-5);
  background: var(--color-danger);
  color: #fff;
  border-radius: var(--radius-md);
  font-size: var(--text-base);
  font-weight: 500;
}

.btn-danger:hover {
  opacity: 0.9;
}

.btn-secondary {
  padding: var(--space-2) var(--space-4);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.btn-secondary:hover {
  border-color: var(--accent-blue);
}

.change-password-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4) 0;
}

.password-input {
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.password-input:focus {
  border-color: var(--accent-blue);
  outline: none;
}

.btn-primary {
  padding: var(--space-2) var(--space-4);
  background: var(--accent-blue);
  border-radius: var(--radius-md);
  color: #fff;
  font-size: var(--text-sm);
  font-weight: 500;
  align-self: flex-start;
}

.btn-primary:hover {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.breach-check-form {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  padding: var(--space-3) 0;
  align-items: center;
}

.breach-result {
  width: 100%;
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
</style>
