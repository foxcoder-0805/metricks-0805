const STORAGE_KEY = 'metricks.apiBase'
const FALLBACK_BASE = 'http://127.0.0.1:8787'

function normalize(base: string): string {
  return base.trim().replace(/\/+$/, '')
}

/**
 * Базовый URL API: переопределение из UI (localStorage) важнее переменной
 * окружения, чтобы можно было переключиться на другой инстанс без пересборки.
 */
export function getApiBase(): string {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored) return normalize(stored)

  const fromEnv = import.meta.env.VITE_API_BASE
  if (typeof fromEnv === 'string' && fromEnv.length > 0) return normalize(fromEnv)

  return FALLBACK_BASE
}

export function setApiBase(base: string): void {
  const normalized = normalize(base)
  if (normalized.length === 0) {
    localStorage.removeItem(STORAGE_KEY)
    return
  }
  localStorage.setItem(STORAGE_KEY, normalized)
}

export const DEFAULT_API_BASE = FALLBACK_BASE
