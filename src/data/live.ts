import type {
  LiveSource,
  MusicStatus,
  OpenCodeStatus,
  StatusCard,
  StatusSnapshot,
  SteamStatus,
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

function opencodeTask(status: OpenCodeStatus): string[] {
  if (!status.available) return ['сессий нет']

  const details = [formatRelative(status.updatedAt)]
  if (status.model) details.push(status.model)
  if (status.idle) details.push('ждёт запроса')

  return [truncate(status.task || 'сессия без названия', 80), details.join(' · ')]
}

function opencodeRequest(status: OpenCodeStatus): string[] {
  if (!status.available) return ['сессий нет']

  const request = status.request || status.task || 'запрос не найден'
  return [truncate(request, 90), formatRelative(status.updatedAt)]
}

function musicTrack(status: MusicStatus): string[] {
  if (!status.available) return ['ничего не играет']

  const timing = status.duration
    ? `${formatDuration(status.position ?? 0)}–${formatDuration(status.duration)}`
    : status.artist || status.player || 'плеер без названия'

  return [truncate(status.track, 90), timing]
}

function musicPlayer(status: MusicStatus): string[] {
  if (!status.available) return ['ничего не играет']

  return [status.player || 'неизвестный плеер', status.playing ? 'играет' : 'на паузе']
}

function steamDownload(status: SteamStatus): string[] {
  if (!status.installed) return ['не установлен']
  if (!status.running) return ['не запущен']
  if (!status.game) return [status.state || 'нет загрузок']

  const line =
    status.percent === null
      ? status.game
      : `${status.game} · ${Math.round(status.percent)}%`

  return [status.state === 'пауза' ? `${line} · пауза` : line]
}

function steamState(status: SteamStatus): string[] {
  if (!status.installed) return ['не установлен']
  if (!status.running) return ['не запущен']

  return [status.state || 'запущен', status.game || 'загрузок нет']
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
    default:
      return ['нет данных']
  }
}

/** Подставляет свежие строки в карточки, привязанные к машине. */
export function renderCard(card: StatusCard, snapshot: StatusSnapshot | null): StatusCard {
  if (!card.source) return card

  return { ...card, lines: sourceLines(card.source, snapshot) }
}
