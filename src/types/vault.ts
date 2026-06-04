/**
 * KeyVault v3 - 共享 TypeScript 类型定义
 * 对应 Rust 后端的结构体
 */

// ============================================
// 条目类型（对应 Rust entry_type 枚举）
// ============================================
export type EntryType =
  | 'login'
  | 'api_key'
  | 'ssh_key'
  | 'note'
  | 'server'
  | 'identity'
  | 'card'
  | 'license'
  | 'crypto'
  | 'custom'

// ============================================
// 字段类型
// ============================================
export type FieldType = 'text' | 'password' | 'otp' | 'url' | 'file' | 'textarea' | 'email' | 'date'

// ============================================
// 条目元数据（列表显示用，不含敏感数据）
// ============================================
export interface EntryMeta {
  id: string
  entryType: EntryType
  title: string
  subtitle?: string
  tags: string[]
  favorited: boolean
  groupId?: string
  updatedAt: number
  /** 回收站条目：软删除时间戳（秒） */
  deletedAt?: number
}

// ============================================
// 解密字段（按需加载）
// ============================================
export interface DecryptedField {
  fieldKey: string
  fieldType: FieldType
  value: string
  isSensitive: boolean
}

export interface EntrySecrets {
  fields: DecryptedField[]
}

// ============================================
// 创建/更新条目的输入
// ============================================
export interface FieldInput {
  fieldKey: string
  fieldType: FieldType
  value: string
  isSensitive: boolean
}

export interface CreateEntryInput {
  entryType: EntryType
  title: string
  subtitle?: string
  tags: string[]
  groupId?: string
  fields: FieldInput[]
}

// ============================================
// 分组
// ============================================
export interface Group {
  id: string
  name: string
  icon?: string
  color?: string
  sortOrder: number
  createdAt: number
  updatedAt: number
}

// ============================================
// 密码生成选项
// ============================================
export interface PasswordOptions {
  length: number
  uppercase: boolean
  lowercase: boolean
  numbers: boolean
  symbols: boolean
  excludeAmbiguous: boolean
  mode: 'random' | 'diceware'
}

// ============================================
// 泄露检测结果
// ============================================
export interface BreachResult {
  breached: boolean
  count: number
}

// ============================================
// 密码强度评估
// ============================================
export interface StrengthResult {
  score: number
  level: 'empty' | 'weak' | 'fair' | 'good' | 'strong'
  label: string
}

// ============================================
// 模板定义
// ============================================
export interface TemplateFieldDef {
  key: string
  label: string
  type: FieldType
  required?: boolean
}

export interface TemplateDef {
  id: EntryType
  name: string
  icon: string
  fields: TemplateFieldDef[]
  customFieldPresets?: TemplateFieldDef[]
}
