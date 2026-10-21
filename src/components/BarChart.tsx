import type { Bars } from '../api/types'
import { chartColor } from '../lib/chartColors'
import { formatCompact, formatNumber } from '../lib/format'
import { linearScale, niceTicks, roundedTopRect } from '../lib/scales'
import { useElementSize } from '../lib/useElementSize'
import { Legend } from './Legend'

interface BarChartProps {
  data: Bars | null
  height?: number
  unit?: string
}

const PADDING = { top: 14, right: 18, bottom: 26, left: 48 }

/** Сгруппированные столбцы со скруглёнными вершинами. */
export function BarChart({ data, height = 240, unit }: BarChartProps) {
  const { ref, width } = useElementSize<HTMLDivElement>()

  const series = data?.series ?? []
  const labels = data?.labels ?? []

  if (width === 0) {
    return <div className="chart" ref={ref} style={{ height }} />
  }

  if (series.length === 0 || labels.length === 0) {
    return (
      <div className="chart" ref={ref} style={{ height }}>
        <div className="chart__empty">Нет данных за выбранный период</div>
      </div>
    )
  }

  const maxValue = Math.max(...series.flatMap((item) => item.values), 0)
  const ticks = niceTicks(maxValue, 4)
  const domainMax = Math.max(ticks[ticks.length - 1], maxValue) || 1

  const innerWidth = Math.max(10, width - PADDING.left - PADDING.right)
  const innerHeight = Math.max(10, height - PADDING.top - PADDING.bottom)

  const baseline = PADDING.top + innerHeight
  const yScale = linearScale([0, domainMax], [baseline, PADDING.top])

  const groupWidth = innerWidth / labels.length
  const barWidth = Math.max(3, (groupWidth * 0.62) / series.length)
  const labelStep = Math.max(1, Math.ceil(labels.length / 10))

  return (
    <div className="chart" ref={ref} style={{ height }}>
      <svg width={width} height={height}>
        <g className="chart__grid">
          {ticks.map((tick) => (
            <line key={tick} x1={PADDING.left} x2={PADDING.left + innerWidth} y1={yScale(tick)} y2={yScale(tick)} />
          ))}
        </g>

        <g className="chart__axis">
          {ticks.map((tick) => (
            <text key={tick} x={PADDING.left - 10} y={yScale(tick) + 4} textAnchor="end">
              {formatNumber(tick)}
            </text>
          ))}
          {labels.map((label, index) =>
            index % labelStep === 0 ? (
              <text
                key={`${label}-${index}`}
                x={PADDING.left + groupWidth * index + groupWidth / 2}
                y={height - 8}
                textAnchor="middle"
              >
                {label}
              </text>
            ) : null,
          )}
        </g>

        {series.map((item, seriesIndex) =>
          item.values.map((value, index) => {
            const groupCenter = PADDING.left + groupWidth * index + groupWidth / 2
            const offset = (seriesIndex - (series.length - 1) / 2) * barWidth
            const x = groupCenter + offset - barWidth / 2
            const y = yScale(value)
            const barHeight = Math.max(0, baseline - y)

            return (
              <path
                key={`${item.id}-${index}`}
                d={roundedTopRect(x, y, barWidth, barHeight, 5)}
                fill={chartColor(seriesIndex)}
                fillOpacity={0.9}
              >
                <title>{`${item.label}: ${formatCompact(value)}`}</title>
              </path>
            )
          }),
        )}
      </svg>

      <Legend
        items={series.map((item, index) => ({
          label: item.label,
          color: chartColor(index),
          value: unit ? `${formatCompact(item.values[item.values.length - 1] ?? 0)} ${unit}` : undefined,
        }))}
      />
    </div>
  )
}
