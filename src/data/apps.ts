import type { AppOption } from '../types'

/**
 * Каталог приложений и показателей. Данные статичны: карточка,
 * добавленная из каталога, сразу получает подпись из выбранного показателя.
 */
export const appCatalog: AppOption[] = [
  {
    id: 'opencode',
    title: 'OpenCode',
    metrics: [
      { id: 'task', label: 'Текущая задача', value: 'Правит конфиг плагина сборки · 19 мин назад' },
      { id: 'steps', label: 'Прогресс шагов', value: '7 из 9 шагов · остался линтер' },
    ],
  },
  {
    id: 'steam',
    title: 'Steam',
    metrics: [
      { id: 'download', label: 'Загрузка игры', value: 'Cyberpunk 2077 · 42%' },
      { id: 'updates', label: 'Обновления', value: 'Counter-Strike 2 · обновление 1.2 ГБ' },
    ],
  },
  {
    id: 'music',
    title: 'Музыка',
    metrics: [
      { id: 'track', label: 'Текущий трек', value: 'Barbie girl · 1:08–3:05' },
      { id: 'player', label: 'Плеер', value: 'Spotify · альбом Plejura' },
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
      { id: 'branch', label: 'Ветка git', value: 'feature/static-card-builder · 4 коммита' },
    ],
  },
]
