import type { Summary } from '../api/types'

const iso = new Date().toISOString()

const summaryBase: Summary = {
  meta: { generatedAt: iso, period: '24h' },
  user: { name: 'Elisey Lomtev', initials: 'EL', role: 'platform', status: 'online' },
  status: { label: 'В норме', state: 'ok' },
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
}
