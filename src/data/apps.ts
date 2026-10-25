import type { AppOption } from '../types'

/**
 * Каталог приложений и показателей. У каждого показателя есть живой источник
 * на машине (`source`), поэтому в каталоге нет ничего, что нельзя проверить:
 * если источник недоступен, карточка честно пишет «нет данных».
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
    id: 'music',
    title: 'Музыка',
    metrics: [
      { id: 'track', label: 'Текущий трек', source: 'music.track' },
      { id: 'player', label: 'Плеер', source: 'music.player' },
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
    id: 'qbt',
    title: 'qBittorrent',
    metrics: [
      { id: 'download', label: 'Загрузка торрента', source: 'qbt.download' },
      { id: 'state', label: 'Состояние клиента', source: 'qbt.state' },
    ],
  },
  {
    id: 'system',
    title: 'Система',
    metrics: [
      { id: 'memory', label: 'Память', source: 'system.memory' },
      { id: 'disk', label: 'Диск', source: 'system.disk' },
      { id: 'load', label: 'Нагрузка', source: 'system.load' },
    ],
  },
  {
    id: 'focus',
    title: 'Окно',
    metrics: [
      { id: 'window', label: 'Активное окно', source: 'focus.window' },
      { id: 'app', label: 'Приложение', source: 'focus.app' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    metrics: [
      { id: 'branch', label: 'Ветка проекта', source: 'git.branch' },
      { id: 'changes', label: 'Изменения', source: 'git.changes' },
    ],
  },
  {
    id: 'network',
    title: 'Сеть',
    metrics: [
      { id: 'connection', label: 'Соединение', source: 'network.connection' },
      { id: 'wifi', label: 'Wi-Fi', source: 'network.wifi' },
    ],
  },
]
