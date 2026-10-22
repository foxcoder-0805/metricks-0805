import type { StatusCard } from '../types'

/** Состояние доски в шапке. */
export const dashboardStatus = {
  label: 'В норме',
}

/**
 * Встроенные карточки. Все три привязаны к живым источникам машины,
 * поэтому их подписи собираются из снапшота, а не хранятся здесь.
 */
export const builtinCards: StatusCard[] = [
  {
    id: 'opencode',
    title: 'OpenCode',
    lines: ['читаем сессию'],
    source: 'opencode.task',
  },
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
]
