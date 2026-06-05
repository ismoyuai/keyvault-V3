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
  isInitialized: () => invoke<boolean>('is_initialized'),
  getUnlockStatus: () => invoke<UnlockStatus>('get_unlock_status'),
  setup: (password: string) => invoke<string>('setup', { password }),
  unlock: (password: string) => invoke<string>('unlock', { password }),
  lock: () => invoke<void>('lock'),
  changePassword: (oldPassword: string, newPassword: string) =>
    invoke<void>('change_password', {
      sessionToken: getToken(),
      oldPassword,
      newPassword,
    }),
  emergencyWipe: (password: string, confirmation: string) =>
    invoke<void>('emergency_wipe', {
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
    invoke<EntryMeta[]>('list_entries', { sessionToken: getToken() }),

  getEntrySecrets: (entryId: string) =>
    invoke<EntrySecrets>('get_entry_secrets', {
      sessionToken: getToken(),
      entryId,
    }),

  createEntry: (input: CreateEntryInput) =>
    invoke<string>('create_entry', { sessionToken: getToken(), input }),

  updateEntry: (entryId: string, input: CreateEntryInput) =>
    invoke<void>('update_entry', { sessionToken: getToken(), entryId, input }),

  deleteEntry: (entryId: string) =>
    invoke<void>('delete_entry', { sessionToken: getToken(), entryId }),

  listTrashEntries: () =>
    invoke<EntryMeta[]>('list_trash_entries', { sessionToken: getToken() }),

  restoreEntry: (entryId: string) =>
    invoke<void>('restore_entry', { sessionToken: getToken(), entryId }),

  purgeEntry: (entryId: string) =>
    invoke<void>('purge_entry', { sessionToken: getToken(), entryId }),

  emptyTrash: () =>
    invoke<number>('empty_trash', { sessionToken: getToken() }),

  toggleFavorite: (entryId: string) =>
    invoke<void>('toggle_favorite', { sessionToken: getToken(), entryId }),

  searchEntries: (query: string) =>
    invoke<EntryMeta[]>('search_entries', { sessionToken: getToken(), query }),

  exportVault: (format: string) =>
    invoke<string>('export_vault', { sessionToken: getToken(), format }),

  importVault: (data: string, format: string) =>
    invoke<number>('import_vault', { sessionToken: getToken(), data, format }),
}

// ============================================
// 分组
// ============================================
export const groups = {
  list: () => invoke<Group[]>('list_groups', { sessionToken: getToken() }),

  create: (name: string, icon?: string) =>
    invoke<string>('create_group', { sessionToken: getToken(), name, icon }),

  update: (groupId: string, name: string, icon?: string) =>
    invoke<void>('update_group', {
      sessionToken: getToken(),
      groupId,
      name,
      icon,
    }),

  delete: (groupId: string) =>
    invoke<void>('delete_group', { sessionToken: getToken(), groupId }),
}

// ============================================
// 安全功能
// ============================================
export const security = {
  checkBreach: (password: string) =>
    invoke<BreachResult>('check_password_breach', {
      sessionToken: getToken(),
      password,
    }),

  generatePassword: (options: PasswordOptions) =>
    invoke<string>('generate_password', { sessionToken: getToken(), options }),
}

// ============================================
// 窗口控制
// ============================================
export const window = {
  minimize: () => invoke<void>('minimize_window'),
  maximize: () => invoke<void>('toggle_maximize'),
  close: () => invoke<void>('close_window'),
}

// ============================================
// 剪贴板
// ============================================
export const clipboard = {
  copy: (text: string) => invoke<void>('copy_to_clipboard', { text }),
  clear: () => invoke<void>('clear_clipboard'),
}

// ============================================
// 应用设置
// ============================================
export const settings = {
  get: (key: string) => invoke<string | null>('get_setting', { key }),
  set: (key: string, value: string) =>
    invoke<void>('set_setting', { sessionToken: getToken(), key, value }),
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
  getConfig: () => invoke<SyncConfig>('get_sync_config'),
  setConfig: (url: string, username: string, password: string) =>
    invoke<void>('set_sync_config', {
      sessionToken: getToken(),
      input: { url, username, password },
    }),
  testConnection: () =>
    invoke<void>('test_webdav_connection', { sessionToken: getToken() }),
  push: () => invoke<SyncPushResult>('sync_push', { sessionToken: getToken() }),
  pull: () => invoke<SyncPullResult>('sync_pull', { sessionToken: getToken() }),
  getStatus: () =>
    invoke<SyncStatus>('get_sync_status', { sessionToken: getToken() }),
}
