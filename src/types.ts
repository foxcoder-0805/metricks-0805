/** Живой источник состояния машины, к которому можно привязать карточку. */
export type LiveSource =
  | 'opencode.task'
  | 'opencode.request'
  | 'music.track'
  | 'music.player'
  | 'steam.download'
  | 'steam.state'
  | 'system.memory'
  | 'system.disk'
  | 'system.load'
  | 'focus.window'
  | 'focus.app'
  | 'git.branch'
  | 'git.changes'
  | 'network.connection'
  | 'network.wifi'

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
  /** Живой источник на машине: всё, что можно добавить, реально читается. */
  source: LiveSource
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
  /** Открытые сессии: свежие сверху. */
  sessions: OpenCodeSession[]
}

/** Одна открытая сессия OpenCode. */
export interface OpenCodeSession {
  id: string
  task: string
  request: string
  model: string
  directory: string
  updatedAt: number | null
  filesChanged: number | null
  /** Что OpenCode делает прямо сейчас; пусто, если ждёт запроса. */
  activity: string
  idle: boolean
}

/** Владелец сессии на этой машине. */
export interface UserStatus {
  /** Логин: `foxcoder`. */
  login: string
  home: string
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
  /** Процент загрузки по манифесту игры. */
  percent: number | null
  downloaded: number | null
  total: number | null
  /** Скорость в байтах в секунду. */
  rate: number | null
  /** Сколько секунд осталось при текущей скорости. */
  eta: number | null
  state: string
}

/** Память, диск, нагрузка и время работы машины. */
export interface SystemStatus {
  available: boolean
  memoryUsed: number | null
  memoryTotal: number | null
  diskUsed: number | null
  diskTotal: number | null
  /** Средняя загрузка за последнюю минуту. */
  load: number | null
  /** Время работы в секундах. */
  uptime: number | null
}

/** Окно, которое сейчас в фокусе (Hyprland). */
export interface FocusStatus {
  available: boolean
  app: string
  title: string
  workspace: string
}

/** Git-состояние проекта, в котором идёт сессия OpenCode. */
export interface GitStatus {
  available: boolean
  branch: string
  /** Изменённых файлов; `null` — посчитать не удалось. */
  changed: number | null
  directory: string
}

/** Активное сетевое подключение из NetworkManager. */
export interface NetworkStatus {
  available: boolean
  connection: string
  device: string
  /** SSID для Wi-Fi, иначе пустая строка. */
  wifi: string
  signal: number | null
}

/** Кадр спектра системного звука для полосы внизу окна. */
export interface AudioBars {
  /** Есть звук прямо сейчас (или был мгновение назад). */
  active: boolean
  /** Уровни полосок от 0 до 1. */
  bars: number[]
  /** Текущая нота, `—` если тон не распознан. */
  note: string
  /** Лента последних нот: свежая в конце. */
  notes: string[]
}

/** Срез состояния машины, который отдаёт Rust-слой. */
export interface StatusSnapshot {
  opencode: OpenCodeStatus
  music: MusicStatus
  steam: SteamStatus
  system: SystemStatus
  focus: FocusStatus
  git: GitStatus
  network: NetworkStatus
  user: UserStatus
  collectedAt: number
}
