import type { StatusCard } from '../types'

/** Состояние доски в шапке. */
export const dashboardStatus = {
  label: 'В норме',
}

/**
 * Встроенные карточки. Все привязаны к живым источникам машины,
 * поэтому их подписи собираются из снапшота, а не хранятся здесь.
 * Карточки OpenCode не здесь: их по одной на каждую открытую сессию.
 */
export const builtinCards: StatusCard[] = [
  {
    id: 'music',
    title: 'Музыка',
    lines: ['ищем плеер'],
    source: 'music.track',
  },
  {
    id: 'steam',
    title: 'Steam',
    lines: ['смотрим логи'],
    source: 'steam.download',
  },
  {
    id: 'qbt',
    title: 'qBittorrent',
    lines: ['смотрим BT_backup'],
    source: 'qbt.download',
  },
]
