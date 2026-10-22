import type { Range, Series, Summary } from '../api/types'

const now = new Date()
const iso = now.toISOString()

function wave(len: number, base: number, amp: number, trend = 0): number[] {
  const res: number[] = []
  for (let i = 0; i < len; i++) {
    const t = i / (len - 1)
    const v = base + Math.sin(t * Math.PI * 2.6 + i * 0.37) * amp + trend * t + (Math.sin(i * 0.73) * 2)
    res.push(Math.max(0, Math.round(v * 10) / 10))
  }
  return res
}

const summaryBase: Summary = {
  meta: { generatedAt: iso, period: '24h' },
  user: { name: 'Elisey Lomtev', initials: 'EL', role: 'platform', status: 'online' },
  status: { label: 'В норме', state: 'ok' },
  overview: {
    title: 'Динамика по источникам',
    value: 124680,
    unit: 'запросов',
    label: 'Всего запросов',
    description:
      'Telegram даёт половину трафика и растёт равномерно, OpenCode прибавляет быстрее остальных, Steam держится на вечерних пиках. Данные обновляются каждые 15 секунд.',
  },
  sources: [
    {
      id: 'telegram',
      title: 'Telegram',
      headline: 'Подписчик был онлайн',
      value: 12480,
      unit: 'подписчиков',
      note: '19 мин назад',
      delta: 6.4,
      spark: wave(24, 95, 14, 12),
    },
    {
      id: 'opencode',
      title: 'OpenCode',
      headline: 'Сессия активна',
      value: 1248,
      unit: 'токенов/мин',
      note: '3 мин назад',
      delta: 12.1,
      spark: wave(24, 46, 13, 9),
    },
    {
      id: 'steam',
      title: 'Steam',
      headline: 'Сессия в игре',
      value: 2940,
      unit: 'игроков онлайн',
      note: '41 мин назад',
      delta: -3.2,
      spark: wave(24, 28, 10, 4),
    },
  ],
  cards: [
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
  ],
  session: {
    task: 'Рефакторинг плагина сборки',
    summary:
      'Выношу резолвер версий в отдельный модуль и покрываю его тестами на матрице версий Node — сейчас правки в конфиге плагина.',
    tags: ['модель glm-4.6', 'каталог ~/prog/metricks', 'ветка feature/live-status-cards'],
    done: 7,
    total: 9,
    steps: [
      { at: '16:02', text: 'Разобрал текущий резолвер версий и его зависимости' },
      { at: '16:07', text: 'Вынес выбор версии в src/lib/resolve-version.ts' },
      { at: '16:11', text: 'Добавил тесты на матрице версий Node' },
      { at: '16:14', text: 'Прогнал сборку, осталось поправить два предупреждения линтера' },
    ],
  },
}

function labelsFor(range: Range, len: number): string[] {
  if (range === '1h') return Array.from({ length: len }, (_, i) => `${String(i).padStart(2, '0')}:00`).slice(0, len)
  if (range === '7d') return ['Пн', 'Вт', 'Ср', 'Чт', 'Пт', 'Сб', 'Вс']
  if (range === '30d') return Array.from({ length: len }, (_, i) => `${i + 1}-го`)
  const base = ['00:00', '01:00', '02:00', '03:00', '04:00', '05:00', '06:00', '07:00', '08:00', '09:00', '10:00', '11:00', '12:00', '13:00', '14:00', '15:00', '16:00', '17:00', '18:00', '19:00', '20:00', '21:00', '22:00', '23:00']
  return base.slice(0, len)
}

function rangeLen(range: Range): number {
  if (range === '1h') return 12
  if (range === '7d') return 7
  if (range === '30d') return 14
  return 24
}

function buildSeries(range: Range): Series {
  const len = rangeLen(range)
  return {
    x: labelsFor(range, len),
    series: [
      { id: 'telegram', label: 'Telegram', points: wave(len, 64, 16, 4) },
      { id: 'opencode', label: 'OpenCode', points: wave(len, 42, 12, 3) },
      { id: 'steam', label: 'Steam', points: wave(len, 26, 9, 2) },
    ],
  }
}

export const embedded = {
  health() {
    return { ok: true }
  },
  summary(): Summary {
    return {
      ...summaryBase,
      meta: { generatedAt: new Date().toISOString(), period: summaryBase.meta.period },
    }
  },
  series(range: Range): Series {
    return buildSeries(range)
  },
}
