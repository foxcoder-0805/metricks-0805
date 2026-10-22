import type { StatusCard } from '../types'

/** Пользователь в шапке. */
export const dashboardUser = {
  name: 'Elisey Lomtev',
  initials: 'EL',
  role: 'platform',
  status: 'online' as const,
}

/** Состояние доски в шапке. */
export const dashboardStatus = {
  label: 'В норме',
}

/** Стартовый набор карточек. */
export const initialCards: StatusCard[] = [
  {
    id: 'opencode',
    title: 'OpenCode',
    lines: ['Правит конфиг плагина сборки', '19 мин назад'],
  },
  {
    id: 'music',
    title: 'Музыка',
    lines: ['Barbie girl · 1:08–3:05'],
  },
  {
    id: 'steam',
    title: 'Steam',
    lines: ['Cyberpunk 2077 · 42%'],
  },
]
