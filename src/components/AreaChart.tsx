import { useId, useState } from 'react'
import type { Series } from '../api/types'
import { chartColor } from '../lib/chartColors'
import { formatCompact, formatNumber } from '../lib/format'
import { areaPath, linearScale, niceTicks, smoothPath, type Point } from '../lib/scales'
import { useElementSize } from '../lib/useElementSize'
import { Legend } from './Legend'

interface AreaChartProps {
  data: Series | null
  height?: number
  unit?: string
  showLegend?: boolean
}

const PADDING = { top: 14, right: 18, bottom: 26, left: 48 }

/** Сглаженный график с несколькими сериями — как волна в макете. */
export function AreaChart({ data, height = 280, unit, showLegend = true }: AreaChartProps) {
  const { ref, width } = useElementSize<HTMLDivElement>()
  const [hover, setHover] = useState<number | null>(null)
  const uid = useId().replace(/[^a-zA-Z0-9]/g, '')

  const series = data?.series ?? []
  const labels = data?.x ?? []

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

  const maxValue = Math.max(...series.flatMap((item) => item.points), 0)
  const ticks = niceTicks(maxValue, 4)
  const domainMax = Math.max(ticks[ticks.length - 1], maxValue) || 1

  const innerWidth = Math.max(10, width - PADDING.left - PADDING.right)
  const innerHeight = Math.max(10, height - PADDING.top - PADDING.bottom)

  const xScale = linearScale([0, Math.max(1, labels.length - 1)], [PADDING.left, PADDING.left + innerWidth])
  const yScale = linearScale([0, domainMax], [PADDING.top + innerHeight, PADDING.top])

  const pointsOf = (points: number[]): Point[] =>
    points.map((value, index) => [xScale(index), yScale(value)] as Point)

  const labelStep = Math.max(1, Math.ceil(labels.length / 7))
  const hoverIndex = hover === null ? null : Math.min(labels.length - 1, Math.max(0, hover))

  const hoverValues =
    hoverIndex === null
      ? []
      : series.map((item, index) => ({
          label: item.label,
          value: item.points[hoverIndex] ?? 0,
          color: chartColor(index),
        }))

  const tooltipTop =
    hoverIndex === null
      ? 0
      : Math.min(...hoverValues.map((item) => yScale(item.value))) - 12

  return (
    <div className="chart" ref={ref} style={{ height }}>
      <svg
        width={width}
        height={height}
        onMouseLeave={() => setHover(null)}
        onMouseMove={(event) => {
          const rect = event.currentTarget.getBoundingClientRect()
          const x = event.clientX - rect.left
          const index = Math.round(xScale.invert(x))
          setHover(Math.min(labels.length - 1, Math.max(0, index)))
        }}
      >
        <defs>
          {series.map((item, index) => (
            <linearGradient key={item.id} id={`${uid}-${item.id}`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={chartColor(index)} stopOpacity="0.42" />
              <stop offset="100%" stopColor={chartColor(index)} stopOpacity="0" />
            </linearGradient>
          ))}
        </defs>

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
              <text key={`${label}-${index}`} x={xScale(index)} y={height - 8} textAnchor="middle">
                {label}
              </text>
            ) : null,
          )}
        </g>

        {series.map((item, index) => {
          const points = pointsOf(item.points)
          return (
            <g key={item.id}>
              <path d={areaPath(points, PADDING.top + innerHeight)} fill={`url(#${uid}-${item.id})`} />
              <path
                d={smoothPath(points)}
                fill="none"
                stroke={chartColor(index)}
                strokeWidth={2}
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </g>
          )
        })}

        {hoverIndex !== null ? (
          <g>
            <line
              x1={xScale(hoverIndex)}
              x2={xScale(hoverIndex)}
              y1={PADDING.top}
              y2={PADDING.top + innerHeight}
              stroke="var(--purple-400)"
              strokeOpacity={0.45}
              strokeDasharray="4 4"
            />
            {series.map((item, index) => (
              <circle
                key={item.id}
                cx={xScale(hoverIndex)}
                cy={yScale(item.points[hoverIndex] ?? 0)}
                r={4}
                fill="var(--surface)"
                stroke={chartColor(index)}
                strokeWidth={2}
              />
            ))}
          </g>
        ) : null}
      </svg>

      {hoverIndex !== null ? (
        <div
          className="tooltip"
          style={{ left: xScale(hoverIndex), top: Math.max(38, tooltipTop) }}
        >
          <div className="tooltip__title">
            {labels[hoverIndex]}
            {unit ? ` · ${unit}` : ''}
          </div>
          {hoverValues.map((item) => (
            <div className="tooltip__row" key={item.label}>
              <span className="tooltip__dot" style={{ background: item.color }} />
              <span className="tooltip__label">{item.label}</span>
              <span className="tooltip__value">{formatCompact(item.value)}</span>
            </div>
          ))}
        </div>
      ) : null}

      {showLegend ? (
        <Legend
          items={series.map((item, index) => ({
            label: item.label,
            color: chartColor(index),
            value: formatCompact(item.points[item.points.length - 1] ?? 0),
          }))}
        />
      ) : null}
    </div>
  )
}
