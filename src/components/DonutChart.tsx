import type { Donut } from '../api/types'
import { chartColor } from '../lib/chartColors'
import { formatNumber } from '../lib/format'
import { arcPath } from '../lib/scales'

interface DonutChartProps {
  donut: Donut | null
  size?: number
}

const START = -Math.PI / 2
const GAP = 0.03

/** Кольцевая диаграмма с легендой — распределение по источникам. */
export function DonutChart({ donut, size = 168 }: DonutChartProps) {
  if (!donut || donut.items.length === 0) {
    return <div className="chart__empty">Нет данных</div>
  }

  const total = donut.items.reduce((sum, item) => sum + item.value, 0) || 1
  const radius = size / 2 - 14
  const center = size / 2

  // Углы секторов считаются от накопленных долей: без изменения состояния
  // между итерациями map, иначе сегменты «съедают» друг друга.
  const segments = donut.items.map((item, index) => {
    const fraction = item.value / total
    const start = START + index * Math.PI * 2 * fraction + GAP / 2
    const end = START + (index + 1) * Math.PI * 2 * fraction - GAP / 2
    return {
      key: item.label,
      d: arcPath(center, center, radius, start, end),
      color: chartColor(index),
      item,
    }
  })

  return (
    <div className="chart">
      <svg width={size} height={size} style={{ margin: '0 auto' }} role="img" aria-label={donut.label}>
        <circle cx={center} cy={center} r={radius} fill="none" stroke="var(--surface-3)" strokeWidth={18} />
        {segments.map((segment) => (
          <path key={segment.key} d={segment.d} fill="none" stroke={segment.color} strokeWidth={18}>
            <title>{`${segment.item.label}: ${formatNumber(segment.item.value)}`}</title>
          </path>
        ))}
        <text
          x={center}
          y={center - 4}
          textAnchor="middle"
          dominantBaseline="middle"
          fill="var(--text)"
          fontSize={size * 0.16}
          fontWeight={620}
          letterSpacing="-0.02em"
        >
          {formatNumber(total)}
        </text>
        <text x={center} y={center + 18} textAnchor="middle" dominantBaseline="middle" fill="var(--muted)" fontSize={11}>
          всего
        </text>
      </svg>

      <ul className="legend" style={{ justifyContent: 'center' }}>
        {segments.map((segment) => (
          <li className="legend__item" key={segment.key}>
            <span className="legend__dot" style={{ '--dot': segment.color } as React.CSSProperties} />
            <span>{segment.item.label}</span>
            <span className="legend__value">{Math.round((segment.item.value / total) * 100)}%</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
