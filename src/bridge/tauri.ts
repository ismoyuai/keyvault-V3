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
export const auth = {
  isInitialized: () => invoke<boolean>('is_initialized'),
  setup: (password: string) => invoke<string>('setup', { password }),
  unlock: (password: string) => invoke<string>('unlock', { password }),
  lock: () => invoke<void>('lock'),
  changePassword: (oldPassword: string, newPassword: string) =>
    invoke<void>('change_password', {
      sessionToken: getToken(),
      oldPassword,
      newPassword,
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

  updateEntry: (entryId: string, input: Partial<CreateEntryInput>) =>
    invoke<void>('update_entry', { sessionToken: getToken(), entryId, input }),

  deleteEntry: (entryId: string) =>
    invoke<void>('delete_entry', { sessionToken: getToken(), entryId }),

  toggleFavorite: (entryId: string) =>
    invoke<void>('toggle_favorite', { sessionToken: getToken(), entryId }),

  searchEntries: (query: string) =>
    invoke<EntryMeta[]>('search_entries', { sessionToken: getToken(), query }),
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
// 系统托盘
// ============================================
export const tray = {
  lock: () => invoke<void>('tray_lock'),
  show: () => invoke<void>('tray_show'),
}
