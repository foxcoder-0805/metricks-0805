/** Живой источник состояния машины, к которому можно привязать карточку. */
export type LiveSource =
  | 'opencode.task'
  | 'opencode.request'
  | 'music.track'
  | 'music.player'
  | 'steam.download'
  | 'steam.state'

/** Карточка статуса на доске. */
export interface StatusCard {
  id: string
  /** Крупный заголовок карточки: OpenCode, Музыка, Steam. */
  title: string
  /**
   * Строки подписи внизу слева. У карточек с `source` это запасной текст
   * до первого снимка, дальше строки собираются из живого снапшота.
   */
  lines: string[]
  /** Живой источник: подпись пересобирается при каждом опросе. */
  source?: LiveSource
}

/** Показатель приложения, за которым можно следить. */
export interface MetricOption {
  id: string
  /** Название показателя в списке выбора. */
  label: string
  /** Живой источник, если показатель можно отследить на машине. */
  source?: LiveSource
  /** Статичная подпись для показателей без источника. */
  value?: string
}

/** Приложение, доступное для добавления на доску. */
export interface AppOption {
  id: string
  /** Крупное слово в карточке. */
  title: string
  /** Что можно отслеживать у этого приложения. */
  metrics: MetricOption[]
}

/** Последняя сессия OpenCode. */
export interface OpenCodeStatus {
  available: boolean
  task: string
  request: string
  model: string
  directory: string
  updatedAt: number | null
  filesChanged: number | null
  idle: boolean
}

/** Текущее воспроизведение MPRIS-плеера. */
export interface MusicStatus {
  available: boolean
  track: string
  artist: string
  position: number | null
  duration: number | null
  playing: boolean
  player: string
}

/** Состояние клиента Steam. */
export interface SteamStatus {
  installed: boolean
  running: boolean
  game: string
  percent: number | null
  state: string
}

/** Срез состояния машины, который отдаёт Rust-слой. */
export interface StatusSnapshot {
  opencode: OpenCodeStatus
  music: MusicStatus
  steam: SteamStatus
  collectedAt: number
}
