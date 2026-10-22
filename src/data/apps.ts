import type { AppOption } from '../types'

/**
 * Каталог приложений и показателей. У OpenCode, музыки и Steam показатели
 * привязаны к живым источникам машины (`source`), у остальных приложений
 * подпись статична — их на машине пока нечем отследить.
 */
export const appCatalog: AppOption[] = [
  {
    id: 'opencode',
    title: 'OpenCode',
    metrics: [
      { id: 'task', label: 'Текущая задача', source: 'opencode.task' },
      { id: 'request', label: 'Последний запрос', source: 'opencode.request' },
    ],
  },
  {
    id: 'steam',
    title: 'Steam',
    metrics: [
      { id: 'download', label: 'Загрузка игры', source: 'steam.download' },
      { id: 'state', label: 'Состояние клиента', source: 'steam.state' },
    ],
  },
  {
    id: 'music',
    title: 'Музыка',
    metrics: [
      { id: 'track', label: 'Текущий трек', source: 'music.track' },
      { id: 'player', label: 'Плеер', source: 'music.player' },
    ],
  },
  {
    id: 'telegram',
    title: 'Telegram',
    metrics: [
      { id: 'unread', label: 'Непрочитанные', value: '4 непрочитанных в двух чатах' },
      { id: 'online', label: 'Активность', value: 'Был онлайн 19 мин назад' },
    ],
  },
  {
    id: 'discord',
    title: 'Discord',
    metrics: [
      { id: 'voice', label: 'Голосовой канал', value: 'Сидит в канале · 1:24' },
      { id: 'friends', label: 'Друзья онлайн', value: '3 друга в сети' },
    ],
  },
  {
    id: 'browser',
    title: 'Браузер',
    metrics: [
      { id: 'tabs', label: 'Открытые вкладки', value: '12 вкладок · 3 с документацией' },
      { id: 'bookmarks', label: 'Закладки', value: '8 закладок · последняя добавлена сегодня' },
    ],
  },
  {
    id: 'files',
    title: 'Файлы',
    metrics: [
      { id: 'downloads', label: 'Загрузки', value: 'Отчёт по продажам.pdf · 2.4 МБ' },
      { id: 'backup', label: 'Резервная копия', value: 'Синхронизация 12 минут назад' },
    ],
  },
  {
    id: 'terminal',
    title: 'Терминал',
    metrics: [
      { id: 'command', label: 'Активная задача', value: 'npm run build · 2 предупреждения' },
      { id: 'branch', label: 'Ветка git', value: 'feature/live-status · 4 коммита' },
    ],
  },
]
