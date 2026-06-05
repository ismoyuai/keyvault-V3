/**
 * Tauri IPC 桥接层
 * 统一封装所有 invoke() 调用，管理会话令牌
 */
import { invoke } from '@tauri-apps/api/core'
import type {
  EntryMeta,
  EntrySecrets,
  CreateEntryInput,
  BreachResult,
  PasswordOptions,
  Group,
} from '@/types/vault'

const NOT_TAURI_MSG =
  '请在 Tauri 桌面应用中运行（npm run tauri dev），浏览器模式无法访问密码库'

/** 是否在 Tauri WebView 环境内 */
export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function ipcInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return Promise.reject(new Error(NOT_TAURI_MSG))
  }
  return invoke<T>(cmd, args)
}

// ============================================
// 会话令牌管理
// ============================================
let _sessionToken: string | null = null

export function setSessionToken(token: string) {
  _sessionToken = token
}

export function clearSessionToken() {
  _sessionToken = null
}

function getToken(): string {
  if (!_sessionToken) throw new Error('未解锁')
  return _sessionToken
}

// ============================================
// 认证
// ============================================
export interface UnlockStatus {
  failureCount: number
  locked: boolean
  secondsRemaining: number
}

export const auth = {
  isInitialized: () => ipcInvoke<boolean>('is_initialized'),
  getUnlockStatus: () => ipcInvoke<UnlockStatus>('get_unlock_status'),
  setup: (password: string) => ipcInvoke<string>('setup', { password }),
  unlock: (password: string) => ipcInvoke<string>('unlock', { password }),
  lock: () => ipcInvoke<void>('lock', { sessionToken: getToken() }),
  changePassword: (oldPassword: string, newPassword: string) =>
    ipcInvoke<void>('change_password', {
      sessionToken: getToken(),
      oldPassword,
      newPassword,
    }),
  emergencyWipe: (password: string, confirmation: string) =>
    ipcInvoke<void>('emergency_wipe', {
      sessionToken: getToken(),
      password,
      confirmation,
    }),
}

// ============================================
// 金库操作
// ============================================
export const vault = {
  listEntries: () =>
    ipcInvoke<EntryMeta[]>('list_entries', { sessionToken: getToken() }),

  getEntrySecrets: (entryId: string) =>
    ipcInvoke<EntrySecrets>('get_entry_secrets', {
      sessionToken: getToken(),
      entryId,
    }),

  createEntry: (input: CreateEntryInput) =>
    ipcInvoke<string>('create_entry', { sessionToken: getToken(), input }),

  updateEntry: (entryId: string, input: CreateEntryInput) =>
    ipcInvoke<void>('update_entry', { sessionToken: getToken(), entryId, input }),

  deleteEntry: (entryId: string) =>
    ipcInvoke<void>('delete_entry', { sessionToken: getToken(), entryId }),

  listTrashEntries: () =>
    ipcInvoke<EntryMeta[]>('list_trash_entries', { sessionToken: getToken() }),

  restoreEntry: (entryId: string) =>
    ipcInvoke<void>('restore_entry', { sessionToken: getToken(), entryId }),

  purgeEntry: (entryId: string) =>
    ipcInvoke<void>('purge_entry', { sessionToken: getToken(), entryId }),

  emptyTrash: () =>
    ipcInvoke<number>('empty_trash', { sessionToken: getToken() }),

  toggleFavorite: (entryId: string) =>
    ipcInvoke<void>('toggle_favorite', { sessionToken: getToken(), entryId }),

  searchEntries: (query: string) =>
    ipcInvoke<EntryMeta[]>('search_entries', { sessionToken: getToken(), query }),

  exportVault: (format: string) =>
    ipcInvoke<string>('export_vault', { sessionToken: getToken(), format }),

  importVault: (data: string, format: string) =>
    ipcInvoke<number>('import_vault', { sessionToken: getToken(), data, format }),

  exportToFile: (format: string) =>
    ipcInvoke<boolean>('export_vault_to_file', { sessionToken: getToken(), format }),

  importFromFile: (format: string) =>
    ipcInvoke<number>('import_vault_from_file', { sessionToken: getToken(), format }),
}

// ============================================
// 分组
// ============================================
export const groups = {
  list: () => ipcInvoke<Group[]>('list_groups', { sessionToken: getToken() }),

  create: (name: string, icon?: string) =>
    ipcInvoke<string>('create_group', { sessionToken: getToken(), name, icon }),

  update: (groupId: string, name: string, icon?: string) =>
    ipcInvoke<void>('update_group', {
      sessionToken: getToken(),
      groupId,
      name,
      icon,
    }),

  delete: (groupId: string) =>
    ipcInvoke<void>('delete_group', { sessionToken: getToken(), groupId }),
}

// ============================================
// 安全功能
// ============================================
export const security = {
  checkBreach: (password: string) =>
    ipcInvoke<BreachResult>('check_password_breach', {
      sessionToken: getToken(),
      password,
    }),

  generatePassword: (options: PasswordOptions) =>
    ipcInvoke<string>('generate_password', { sessionToken: getToken(), options }),
}

// ============================================
// 窗口控制
// ============================================
export const window = {
  minimize: () => ipcInvoke<void>('minimize_window'),
  maximize: () => ipcInvoke<void>('toggle_maximize'),
  close: () => ipcInvoke<void>('close_window'),
}

// ============================================
// 剪贴板
// ============================================
export const clipboard = {
  copy: (text: string) =>
    ipcInvoke<void>('copy_to_clipboard', { sessionToken: getToken(), text }),
  clear: () => ipcInvoke<void>('clear_clipboard', { sessionToken: getToken() }),
}

// ============================================
// 应用设置
// ============================================
export const settings = {
  get: (key: string) => {
    const args: Record<string, unknown> = { key }
    try {
      args.sessionToken = getToken()
    } catch {
      // 未解锁时仅允许后端白名单内的公开设置项
    }
    return ipcInvoke<string | null>('get_setting', args)
  },
  set: (key: string, value: string) =>
    ipcInvoke<void>('set_setting', { sessionToken: getToken(), key, value }),
}

// ============================================
// WebDAV 同步
// ============================================
export interface SyncConfig {
  url: string
  username: string
  configured: boolean
  lastSyncTime: number | null
  deviceId: string | null
}

export interface SyncPushResult {
  entriesSent: number
  exportedAt: string
}

export interface SyncPullResult {
  entriesMerged: number
  remoteTime: string | null
}

export interface SyncStatus {
  lastRemoteSync: string | null
  lastLocalSync: number | null
}

export const sync = {
  getConfig: () =>
    ipcInvoke<SyncConfig>('get_sync_config', { sessionToken: getToken() }),
  setConfig: (url: string, username: string, password: string) =>
    ipcInvoke<void>('set_sync_config', {
      sessionToken: getToken(),
      input: { url, username, password },
    }),
  testConnection: () =>
    ipcInvoke<void>('test_webdav_connection', { sessionToken: getToken() }),
  push: () => ipcInvoke<SyncPushResult>('sync_push', { sessionToken: getToken() }),
  pull: () => ipcInvoke<SyncPullResult>('sync_pull', { sessionToken: getToken() }),
  getStatus: () =>
    ipcInvoke<SyncStatus>('get_sync_status', { sessionToken: getToken() }),
}
