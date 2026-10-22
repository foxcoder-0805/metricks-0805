/** Контракт данных дашборда. Один и тот же для реального API и для встроенных данных. */

export type HealthState = 'ok' | 'warn' | 'error'

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
  cards: StatusCard[]
  session: Session
}
