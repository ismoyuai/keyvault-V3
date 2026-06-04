import type { EntryType } from '@/types/vault'

/** Material Symbols 名称，与 UNIFIED-SPEC / 原型对齐 */
export const ENTRY_TYPE_ICONS: Record<EntryType, string> = {
  login: 'language',
  api_key: 'code',
  note: 'description',
  server: 'dns',
  identity: 'badge',
  card: 'credit_card',
  license: 'verified',
  crypto: 'account_balance_wallet',
  ssh_key: 'terminal',
  custom: 'tune',
}

/** v1 新建条目可选类型（隐藏 card） */
export const VISIBLE_ENTRY_TYPES: EntryType[] = [
  'login',
  'api_key',
  'note',
  'server',
  'identity',
  'license',
  'crypto',
  'ssh_key',
  'custom',
]
