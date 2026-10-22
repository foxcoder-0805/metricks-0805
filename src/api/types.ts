/** Контракт данных дашборда. Один и тот же для реального API и для mock-сервера. */

export type KpiFormat = 'number' | 'percent' | 'bytes' | 'duration'

export interface Kpi {
  id: string
  label: string
  value: number
  unit?: string
  /** Изменение в процентах за выбранный период. */
  delta?: number
  /** Формат значения: number | percent | bytes | duration. */
  format?: KpiFormat
  /** Иконка из набора components/Icons.tsx. */
  icon?: string
  /** Ряд для спарклайна. */
  spark?: number[]
  /** true — рост значения считается ухудшением (ошибки, latency). */
  invertDelta?: boolean
}

export interface Gauge {
  id: string
  label: string
  value: number
  max: number
  unit?: string
  caption?: string
}

export interface DonutItem {
  label: string
  value: number
}

export interface Donut {
  id: string
  label: string
  items: DonutItem[]
}

export interface TableData {
  id: string
  columns: string[]
  rows: Array<Array<string | number>>
}

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
  kpis: Kpi[]
  gauges: Gauge[]
  donuts: Donut[]
  table: TableData
}

export interface Series {
  x: string[]
  series: Array<{
    id: string
    label: string
    points: number[]
  }>
}

export interface Bars {
  labels: string[]
  series: Array<{
    id: string
    label: string
    values: number[]
  }>
}

export type Range = '1h' | '24h' | '7d' | '30d'

export const RANGES: Array<{ id: Range; label: string }> = [
  { id: '1h', label: '1 час' },
  { id: '24h', label: '24 часа' },
  { id: '7d', label: '7 дней' },
  { id: '30d', label: '30 дней' },
]
