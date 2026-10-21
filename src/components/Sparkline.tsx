import { useId } from 'react'
import { chartColor } from '../lib/chartColors'
import { smoothPath, type Point } from '../lib/scales'

interface SparklineProps {
  points: number[]
  color?: string
  width?: number
  height?: number
}

/** Мини-график в KPI-карточке. Рисуется в viewBox — деталей тут не нужно. */
export function Sparkline({ points, color = chartColor(1), width = 96, height = 34 }: SparklineProps) {
  const gradientId = `spark-${useId().replace(/[^a-zA-Z0-9]/g, '')}`

  if (points.length < 2) return null

  const min = Math.min(...points)
  const max = Math.max(...points)
  const span = max - min || 1

  const scaled: Point[] = points.map((value, index) => {
    const x = (index / (points.length - 1)) * width
    const y = height - 3 - ((value - min) / span) * (height - 6)
    return [x, y]
  })

  const line = smoothPath(scaled)
  const area = `${line} L ${width} ${height} L 0 ${height} Z`

  return (
    <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} aria-hidden="true">
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.45" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      <path d={area} fill={`url(#${gradientId})`} />
      <path d={line} fill="none" stroke={color} strokeWidth={1.8} strokeLinecap="round" />
    </svg>
  )
}
