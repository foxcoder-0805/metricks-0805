import type {
  FocusStatus,
  GitStatus,
  LiveSource,
  MusicStatus,
  NetworkStatus,
  OpenCodeSession,
  OpenCodeStatus,
  QbtStatus,
  StatusCard,
  StatusSnapshot,
  SteamStatus,
  SystemStatus,
} from '../types'

/** «N мин назад» по отметке времени. */
export function formatRelative(timestamp: number | null): string {
  if (!timestamp) return 'нет данных'

  const minutes = Math.round(Math.max(0, Date.now() - timestamp) / 60_000)
  if (minutes < 1) return 'только что'
  if (minutes < 60) return `${minutes} мин назад`

  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} ч назад`

  return `${Math.floor(hours / 24)} дн назад`
}

/** Время снимка: ЧЧ:ММ:СС. */
export function formatClock(timestamp: number | null): string {
  if (!timestamp) return '—'

  return new Date(timestamp).toLocaleTimeString('ru-RU', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

/** Секунды в формате М:СС. */
function formatDuration(seconds: number): string {
  const safe = Math.max(0, Math.floor(seconds))
  return `${Math.floor(safe / 60)}:${String(safe % 60).padStart(2, '0')}`
}

function truncate(text: string, limit: number): string {
  return text.length > limit ? `${text.slice(0, limit - 1).trimEnd()}…` : text
}

const GIB = 1024 ** 3

/** Байты в гигабайтах: `31.1 ГБ`. */
function formatGib(bytes: number): string {
  const value = bytes / GIB
  return `${value >= 100 ? Math.round(value) : value.toFixed(1)} ГБ`
}

/** Русская форма слова по числу. */
function plural(count: number, one: string, few: string, many: string): string {
  const mod10 = count % 10
  const mod100 = count % 100

  if (mod10 === 1 && mod100 !== 11) return one
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return few
  return many
}

/** Секунды в «3 дн 4 ч», «5 ч 26 мин» или «12 мин». */
function formatUptime(seconds: number): string {
  const minutes = Math.floor(seconds / 60)
  const days = Math.floor(minutes / 1440)
  const hours = Math.floor((minutes % 1440) / 60)

  if (days > 0) return `${days} дн ${hours} ч`
  if (hours > 0) return `${hours} ч ${minutes % 60} мин`
  return `${minutes} мин`
}

/** Остаток загрузки: «меньше минуты», «≈44 мин», «≈1 ч 12 мин». */
function formatEta(seconds: number): string {
  if (seconds < 60) return 'меньше минуты'

  const minutes = Math.round(seconds / 60)
  if (minutes < 60) return `≈${minutes} мин`

  const hours = Math.floor(minutes / 60)
  const rest = minutes % 60

  return rest > 0 ? `≈${hours} ч ${rest} мин` : `≈${hours} ч`
}

/** Скорость загрузки: «4.4 МБ/с» или «820 КБ/с». */
function formatRate(bytesPerSecond: number): string {
  const megabytes = bytesPerSecond / 1024 ** 2
  if (megabytes >= 1) return `${megabytes.toFixed(1)} МБ/с`

  return `${Math.max(1, Math.round(bytesPerSecond / 1024))} КБ/с`
}

function opencodeTask(status: OpenCodeStatus): string[] {
  const session = status.sessions[0]
  if (!session) return ['сессий нет']

  const details = [formatRelative(session.updatedAt)]
  if (session.model) details.push(session.model)
  if (session.idle) details.push('ждёт запроса')

  return [truncate(session.task || 'сессия без названия', 80), details.join(' · ')]
}

function opencodeRequest(status: OpenCodeStatus): string[] {
  const session = status.sessions[0]
  if (!session) return ['сессий нет']

  const request = session.request || session.task || 'запрос не найден'
  return [truncate(request, 90), formatRelative(session.updatedAt)]
}

/**
 * Строки карточки одной сессии OpenCode. Заголовок самой карточки общий,
 * поэтому первая строка — название работы, дальше — что происходит сейчас,
 * а в конце время и модель. Активность показываем только когда она есть.
 */
export function opencodeSessionLines(
  session: OpenCodeSession,
  many: boolean,
): string[] {
  const lines = [truncate(session.task || 'сессия без названия', 80)]

  if (session.activity) {
    lines.push(session.activity)
  } else if (session.idle) {
    lines.push('ждёт запроса')
  }

  const details = [formatRelative(session.updatedAt)]
  if (session.model) details.push(session.model)
  // Каталог нужен, только когда сессий несколько — иначе это шум.
  if (many && session.directory) {
    details.push(shortPath(session.directory))
  }

  lines.push(details.join(' · '))

  return lines
}

/** Короткий путь: домашка сокращается до `~`, лишние уровни отбрасываются. */
function shortPath(path: string): string {
  const home = path.match(/^\/home\/[^/]+/)
  const trimmed = home ? path.replace(home[0], '~') : path
  const parts = trimmed.split('/')

  return parts.length > 3 ? `…/${parts.slice(-2).join('/')}` : trimmed
}

function musicTrack(status: MusicStatus): string[] {
  if (!status.available) return ['ничего не играет']

  const timing = status.durationMs
    ? `${formatDuration((status.positionMs ?? 0) / 1000)}–${formatDuration(status.durationMs / 1000)}`
    : status.artist || status.player || 'плеер без названия'

  return [truncate(status.track, 90), timing]
}

function musicPlayer(status: MusicStatus): string[] {
  if (!status.available) return ['ничего не играет']

  return [status.player || 'неизвестный плеер', status.playing ? 'играет' : 'на паузе']
}

/** Скорость и остаток загрузки Steam: «осталось ≈44 мин · 4.4 МБ/с». */
function steamDetails(status: SteamStatus): string[] {
  const details: string[] = []
  if (status.eta !== null) details.push(`осталось ${formatEta(status.eta)}`)
  if (status.rate !== null) details.push(formatRate(status.rate))

  return details
}

function steamDownload(status: SteamStatus): string[] {
  if (!status.installed) return ['не установлен']
  if (!status.running) return ['не запущен']
  if (!status.game) return [status.state || 'нет загрузок']

  const progress =
    status.percent === null ? status.game : `${status.game} · ${Math.round(status.percent)}%`
  const headline = status.state === 'пауза' ? `${progress} · пауза` : progress
  const details = steamDetails(status)

  return details.length > 0
    ? [truncate(headline, 80), details.join(' · ')]
    : [truncate(headline, 80)]
}

function steamState(status: SteamStatus): string[] {
  if (!status.installed) return ['не установлен']
  if (!status.running) return ['не запущен']

  const details: string[] = []
  if (status.percent !== null) details.push(`${Math.round(status.percent)}%`)
  details.push(...steamDetails(status))

  return details.length > 0 ? [status.state || 'запущен', details.join(' · ')] : [status.state || 'запущен']
}

/** Имя торрента с процентом: «BeamNG.drive by Igruha · 69%». */
function qbtDownload(status: QbtStatus): string[] {
  if (!status.installed) return ['не установлен']
  if (!status.running) return ['не запущен']
  if (!status.torrent) return [status.state || 'торрентов нет']

  const progress =
    status.percent === null
      ? status.torrent
      : `${status.torrent} · ${Math.round(status.percent)}%`
  const headline = status.state === 'пауза' ? `${progress} · пауза` : progress

  const details: string[] = []
  if (status.total > 0) {
    // Занятых блоков чуть больше полного размера — это округление до блока.
    const landed = Math.min(status.downloaded, status.total)
    details.push(`${formatGib(landed)} из ${formatGib(status.total)}`)
  }
  if (status.rate !== null) details.push(formatRate(status.rate))

  return details.length > 0
    ? [truncate(headline, 80), details.join(' · ')]
    : [truncate(headline, 80)]
}

function qbtState(status: QbtStatus): string[] {
  if (!status.installed) return ['не установлен']
  if (!status.running) return ['не запущен']

  const details: string[] = []
  if (status.percent !== null) details.push(`${Math.round(status.percent)}%`)
  if (status.eta !== null) details.push(`осталось ${formatEta(status.eta)}`)
  if (status.rate !== null) details.push(formatRate(status.rate))

  return details.length > 0 ? [status.state || 'запущен', details.join(' · ')] : [status.state || 'запущен']
}

function systemMemory(status: SystemStatus): string[] {
  if (!status.available || status.memoryUsed === null || !status.memoryTotal) {
    return ['нет данных о памяти']
  }

  const percent = Math.round((status.memoryUsed / status.memoryTotal) * 100)
  return [`${percent}% занято`, `${formatGib(status.memoryUsed)} из ${formatGib(status.memoryTotal)}`]
}

function systemDisk(status: SystemStatus): string[] {
  if (!status.available || status.diskUsed === null || !status.diskTotal) {
    return ['нет данных о диске']
  }

  const percent = Math.round((status.diskUsed / status.diskTotal) * 100)
  return [`${percent}% занято`, `${formatGib(status.diskTotal - status.diskUsed)} свободно`]
}

function systemLoad(status: SystemStatus): string[] {
  if (!status.available || status.load === null) return ['нет данных о нагрузке']

  const lines = [`${status.load.toFixed(2)} за минуту`]
  if (status.uptime !== null) lines.push(`работает ${formatUptime(status.uptime)}`)

  return lines
}

function focusWindow(status: FocusStatus): string[] {
  if (!status.available) return ['нет активного окна']

  const details = [status.app || 'без класса']
  if (status.workspace) details.push(`рабочий стол ${status.workspace}`)

  return [truncate(status.title || status.app, 80), details.join(' · ')]
}

function focusApp(status: FocusStatus): string[] {
  if (!status.available) return ['нет активного окна']

  return [
    status.app || 'без класса',
    status.workspace ? `рабочий стол ${status.workspace}` : 'рабочий стол неизвестен',
  ]
}

/** «изменено 2 файла», «изменён 1 файл» или «дерево чисто». */
function changesLine(changed: number): string {
  if (changed === 0) return 'дерево чисто'

  const word = plural(changed, 'файл', 'файла', 'файлов')
  return changed % 10 === 1 && changed % 100 !== 11
    ? `изменён ${changed} ${word}`
    : `изменено ${changed} ${word}`
}

function gitBranch(status: GitStatus): string[] {
  if (!status.available) return ['не git-репозиторий']

  const details = status.changed === null ? status.directory || 'изменения неизвестны' : changesLine(status.changed)
  return [status.branch || 'без ветки', details]
}

function gitChanges(status: GitStatus): string[] {
  if (!status.available) return ['не git-репозиторий']

  const where = status.directory ? `в ${status.directory}` : 'в проекте сессии'
  if (status.changed === null) return ['изменения не посчитать', where]

  const headline =
    status.changed === 0
      ? 'дерево чисто'
      : `${status.changed} ${plural(status.changed, 'файл', 'файла', 'файлов')}`

  return [headline, where]
}

function networkConnection(status: NetworkStatus): string[] {
  if (!status.available) return ['нет подключения']

  return [status.connection || 'без имени', status.device || 'без интерфейса']
}

function networkWifi(status: NetworkStatus): string[] {
  if (!status.available) return ['нет подключения']
  if (!status.wifi) return ['не Wi-Fi', status.device || status.connection]

  return [status.wifi, status.signal === null ? status.device : `сигнал ${status.signal}%`]
}

/** Строки карточки по живому источнику. */
export function sourceLines(source: LiveSource, snapshot: StatusSnapshot | null): string[] {
  if (!snapshot) return ['нет данных']

  switch (source) {
    case 'opencode.task':
      return opencodeTask(snapshot.opencode)
    case 'opencode.request':
      return opencodeRequest(snapshot.opencode)
    case 'music.track':
      return musicTrack(snapshot.music)
    case 'music.player':
      return musicPlayer(snapshot.music)
    case 'steam.download':
      return steamDownload(snapshot.steam)
    case 'steam.state':
      return steamState(snapshot.steam)
    case 'qbt.download':
      return qbtDownload(snapshot.qbt)
    case 'qbt.state':
      return qbtState(snapshot.qbt)
    case 'system.memory':
      return systemMemory(snapshot.system)
    case 'system.disk':
      return systemDisk(snapshot.system)
    case 'system.load':
      return systemLoad(snapshot.system)
    case 'focus.window':
      return focusWindow(snapshot.focus)
    case 'focus.app':
      return focusApp(snapshot.focus)
    case 'git.branch':
      return gitBranch(snapshot.git)
    case 'git.changes':
      return gitChanges(snapshot.git)
    case 'network.connection':
      return networkConnection(snapshot.network)
    case 'network.wifi':
      return networkWifi(snapshot.network)
    default:
      return ['нет данных']
  }
}

/** Подставляет свежие строки в карточки, привязанные к машине. */
export function renderCard(card: StatusCard, snapshot: StatusSnapshot | null): StatusCard {
  if (!card.source) return card

  return { ...card, lines: sourceLines(card.source, snapshot) }
}
