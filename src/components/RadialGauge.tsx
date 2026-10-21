import { useId } from 'react'
import type { Gauge } from '../api/types'
import { formatCompact } from '../lib/format'

interface RadialGaugeProps {
  gauge: Gauge
  size?: number
}

/** Круговой индикатор заполнения — как «циферблат» в макете. */
export function RadialGauge({ gauge, size = 128 }: RadialGaugeProps) {
  const uid = useId().replace(/[^a-zA-Z0-9]/g, '')
  const strokeWidth = 9
  const radius = size / 2 - strokeWidth / 2 - 2
  const circumference = 2 * Math.PI * radius
  const ratio = Math.min(1, Math.max(0, gauge.max > 0 ? gauge.value / gauge.max : 0))

  return (
    <div className="gauge">
      <svg width={size} height={size} role="img" aria-label={`${gauge.label}: ${gauge.value}${gauge.unit ?? ''}`}>
        <defs>
          <linearGradient id={`gauge-${uid}`} x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stopColor="var(--purple-400)" />
            <stop offset="100%" stopColor="var(--purple-700)" />
          </linearGradient>
        </defs>

        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--surface-3)"
          strokeWidth={strokeWidth}
        />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke={`url(#gauge-${uid})`}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeDasharray={`${circumference * ratio} ${circumference}`}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
        />
        <text
          x={size / 2}
          y={size / 2 + 2}
          textAnchor="middle"
          dominantBaseline="middle"
          fill="var(--text)"
          fontSize={size * 0.22}
          fontWeight={620}
          letterSpacing="-0.03em"
        >
          {formatCompact(gauge.value)}
        </text>
        <text
          x={size / 2}
          y={size / 2 + size * 0.19}
          textAnchor="middle"
          dominantBaseline="middle"
          fill="var(--muted)"
          fontSize={size * 0.1}
        >
          {gauge.unit ?? ''}
        </text>
      </svg>

      <div>
        <div className="gauge__label">{gauge.label}</div>
        {gauge.caption ? <div className="gauge__label">{gauge.caption}</div> : null}
      </div>
    </div>
  )
}
