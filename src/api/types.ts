/** Контракт данных дашборда. Один и тот же для реального API и для встроенных данных. */

export type HealthState = 'ok' | 'warn' | 'error'

/**
 * Карточка источника: крупное название, тренд в круглом бейдже
 * и две строки подписи — та же анатомия, что в шаблоне макета.
 */
export interface SourceCard {
  id: string
  /** Крупный заголовок карточки: Telegram, OpenCode, Steam. */
  title: string
  /** Первая строка подписи: состояние источника. */
  headline: string
  /** Текущее значение показателя. */
  value: number
  /** Единица измерения значения. */
  unit: string
  /** Вторая часть подписи: «19 мин назад». */
  note: string
  /** Изменение в процентах за выбранный период — стрелка в бейдже. */
  delta?: number
  /** Ряд для спарклайна. */
  spark?: number[]
}

/** Верхний блок большой карточки: крупный показатель и описание. */
export interface Overview {
  title: string
  value: number
  unit: string
  label: string
  description: string
}

export interface Summary {
  meta: {
    generatedAt: string
    period: string
  }
  user: {
    name: string
    initials: string
    role?: string
    status: 'online' | 'away' | 'offline'
  }
  status: {
    label: string
    state: HealthState
  }
  overview: Overview
  sources: SourceCard[]
}

export interface Series {
  x: string[]
  series: Array<{
    id: string
    label: string
    points: number[]
  }>
}

export type Range = '1h' | '24h' | '7d' | '30d'

export const RANGES: Array<{ id: Range; label: string }> = [
  { id: '1h', label: '1 час' },
  { id: '24h', label: '24 часа' },
  { id: '7d', label: '7 дней' },
  { id: '30d', label: '30 дней' },
]
