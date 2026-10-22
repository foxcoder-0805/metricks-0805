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

/**
 * Карточка живого статуса: крупное слово в левом верхнем углу,
 * крест-закрытие в правом верхнем и подпись внизу слева.
 */
export interface StatusCard {
  id: string
  /** Крупный заголовок карточки: OpenCode, Музыка, Steam. */
  title: string
  /** Строки подписи внизу слева: одна или две. */
  lines: string[]
}

/** Текущая сессия OpenCode — содержимое большой карточки. */
export interface Session {
  /** Крупный заголовок задачи. */
  task: string
  /** Описание задачи. */
  summary: string
  /** Короткие метаданные: модель, каталог, ветка. */
  tags: string[]
  /** Выполнено шагов. */
  done: number
  /** Всего шагов в задаче. */
  total: number
  /** Последние шаги агента. */
  steps: Array<{ at: string; text: string }>
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
  cards: StatusCard[]
  session: Session
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
