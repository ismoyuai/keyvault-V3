/**
 * 条目类型模板定义
 * 从旧项目迁移并转换为 TypeScript
 */
import type { EntryType, TemplateDef } from '@/types/vault'

export const TEMPLATES: Record<EntryType, TemplateDef> = {
  login: {
    id: 'login',
    name: '网站密码',
    icon: 'Key',
    fields: [
      { key: 'username', label: '用户名', type: 'text' },
      { key: 'password', label: '密码', type: 'password', required: true },
      { key: 'url', label: '网址', type: 'url' },
    ],
  },
  api_key: {
    id: 'api_key',
    name: 'API 密钥',
    icon: 'Code',
    fields: [
      { key: 'username', label: '用户名', type: 'text' },
      { key: 'api_key', label: 'API Key', type: 'password', required: true },
      { key: 'url', label: 'Endpoint', type: 'url' },
    ],
    customFieldPresets: [
      { key: 'rate_limit', label: 'Rate Limit', type: 'text' },
    ],
  },
  note: {
    id: 'note',
    name: '安全笔记',
    icon: 'FileText',
    fields: [
      { key: 'notes', label: '内容', type: 'textarea' },
    ],
  },
  server: {
    id: 'server',
    name: '服务器',
    icon: 'Server',
    fields: [
      { key: 'username', label: '用户名', type: 'text' },
      { key: 'password', label: '密码/密钥', type: 'password', required: true },
    ],
    customFieldPresets: [
      { key: 'host', label: 'Host', type: 'text' },
      { key: 'port', label: 'Port', type: 'text' },
      { key: 'ssh_key', label: 'SSH Key', type: 'textarea' },
    ],
  },
  identity: {
    id: 'identity',
    name: '身份信息',
    icon: 'User',
    fields: [
      { key: 'username', label: '用户名', type: 'text' },
    ],
    customFieldPresets: [
      { key: 'email', label: '邮箱', type: 'email' },
      { key: 'phone', label: '电话', type: 'text' },
      { key: 'address', label: '地址', type: 'textarea' },
    ],
  },
  card: {
    id: 'card',
    name: '银行卡',
    icon: 'CreditCard',
    fields: [
      { key: 'username', label: '持卡人', type: 'text' },
      { key: 'password', label: '卡号', type: 'password', required: true },
    ],
    customFieldPresets: [
      { key: 'expiry_date', label: '有效期', type: 'text' },
      { key: 'cvv', label: 'CVV', type: 'password' },
    ],
  },
  license: {
    id: 'license',
    name: '软件许可证',
    icon: 'Package',
    fields: [
      { key: 'username', label: '注册邮箱', type: 'email' },
      { key: 'password', label: 'License Key', type: 'password', required: true },
    ],
    customFieldPresets: [
      { key: 'expiry_date', label: '过期时间', type: 'date' },
    ],
  },
  crypto: {
    id: 'crypto',
    name: '加密钱包',
    icon: 'Wallet',
    fields: [
      { key: 'username', label: '钱包名称', type: 'text' },
      { key: 'password', label: '密码', type: 'password' },
    ],
    customFieldPresets: [
      { key: 'address', label: '地址', type: 'text' },
      { key: 'private_key', label: '私钥', type: 'password' },
      { key: 'mnemonic', label: '助记词', type: 'textarea' },
    ],
  },
  ssh_key: {
    id: 'ssh_key',
    name: 'SSH 密钥',
    icon: 'Terminal',
    fields: [
      { key: 'username', label: '用户名', type: 'text' },
      { key: 'password', label: 'Passphrase', type: 'password' },
    ],
    customFieldPresets: [
      { key: 'public_key', label: '公钥', type: 'textarea' },
      { key: 'private_key', label: '私钥', type: 'textarea' },
    ],
  },
  custom: {
    id: 'custom',
    name: '自定义',
    icon: 'File',
    fields: [
      { key: 'username', label: '用户名', type: 'text' },
      { key: 'password', label: '密码', type: 'password' },
    ],
  },
}

export const TEMPLATE_LIST = Object.values(TEMPLATES)

export function getTemplate(id: EntryType): TemplateDef {
  return TEMPLATES[id] || TEMPLATES.custom
}

export function getTemplateIcon(id: EntryType): string {
  return getTemplate(id).icon
}
