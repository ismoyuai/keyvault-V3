/**
 * 密码生成器 composable
 * 使用 crypto.getRandomValues 生成安全随机密码
 * 从旧项目迁移并转换为 TypeScript
 */
import { ref } from 'vue'
import type { StrengthResult } from '@/types/vault'

interface CharsetConfig {
  uppercase: boolean
  lowercase: boolean
  numbers: boolean
  symbols: boolean
}

const CHARSETS: Record<string, string> = {
  uppercase: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ',
  lowercase: 'abcdefghijklmnopqrstuvwxyz',
  numbers: '0123456789',
  symbols: '!@#$%^&*()_+-=[]{}|;:,.<>?',
}

const AMBIGUOUS = 'Il1O0'

/**
 * 从 [0, limit) 范围内生成均匀分布的随机索引
 * 使用 rejection sampling 消除 modulo bias
 */
function secureRandomIndex(limit: number): number {
  if (limit <= 0 || limit > 256) throw new Error('limit must be 1-256')
  const maxValid = Math.floor(256 / limit) * limit
  const bytes = new Uint8Array(1)
  do {
    crypto.getRandomValues(bytes)
  } while (bytes[0] >= maxValid)
  return bytes[0] % limit
}

export function usePasswordGenerator() {
  const password = ref('')
  const length = ref(20)
  const options = ref<CharsetConfig>({
    uppercase: true,
    lowercase: true,
    numbers: true,
    symbols: true,
  })
  const excludeAmbiguous = ref(false)

  function generate(): string {
    let charset = ''
    const required: string[] = []

    if (options.value.uppercase) {
      let chars = CHARSETS.uppercase
      if (excludeAmbiguous.value) chars = chars.replace(/[IO]/g, '')
      charset += chars
      required.push(chars)
    }
    if (options.value.lowercase) {
      let chars = CHARSETS.lowercase
      if (excludeAmbiguous.value) chars = chars.replace(/[l]/g, '')
      charset += chars
      required.push(chars)
    }
    if (options.value.numbers) {
      let chars = CHARSETS.numbers
      if (excludeAmbiguous.value) chars = chars.replace(/[01]/g, '')
      charset += chars
      required.push(chars)
    }
    if (options.value.symbols) {
      charset += CHARSETS.symbols
      required.push(CHARSETS.symbols)
    }

    if (charset.length === 0) {
      charset = CHARSETS.lowercase
      required.push(CHARSETS.lowercase)
    }

    const len = length.value
    let pwd = ''

    // 确保每种选中的字符类型至少出现一次
    for (let i = 0; i < required.length && i < len; i++) {
      const chars = required[i]
      pwd += chars[secureRandomIndex(chars.length)]
    }

    // 填充剩余长度
    for (let i = pwd.length; i < len; i++) {
      pwd += charset[secureRandomIndex(charset.length)]
    }

    // Fisher-Yates 洗牌
    const arr = pwd.split('')
    for (let i = arr.length - 1; i > 0; i--) {
      const j = secureRandomIndex(i + 1)
      ;[arr[i], arr[j]] = [arr[j], arr[i]]
    }

    password.value = arr.join('')
    return password.value
  }

  function evaluateStrength(pwd?: string): StrengthResult {
    const p = pwd || password.value
    if (!p) return { score: 0, level: 'empty', label: '请输入密码' }

    let score = 0

    // 长度
    if (p.length >= 8) score += 1
    if (p.length >= 12) score += 1
    if (p.length >= 16) score += 1
    if (p.length >= 20) score += 1

    // 字符类型
    if (/[a-z]/.test(p)) score += 1
    if (/[A-Z]/.test(p)) score += 1
    if (/[0-9]/.test(p)) score += 1
    if (/[^a-zA-Z0-9]/.test(p)) score += 1

    // 多样性
    const unique = new Set(p).size
    if (unique >= 8) score += 1
    if (unique >= 12) score += 1

    const levels: Array<{ min: number; level: StrengthResult['level']; label: string }> = [
      { min: 0, level: 'weak', label: '弱' },
      { min: 3, level: 'fair', label: '一般' },
      { min: 5, level: 'good', label: '良好' },
      { min: 8, level: 'strong', label: '强' },
    ]

    const matched = [...levels].reverse().find(l => score >= l.min)!
    return { score: Math.min(score, 10), ...matched }
  }

  return {
    password,
    length,
    options,
    excludeAmbiguous,
    generate,
    evaluateStrength,
  }
}
