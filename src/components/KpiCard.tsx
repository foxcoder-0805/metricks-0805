import type { Kpi } from '../api/types'
import { formatDelta, formatKpi } from '../lib/format'
import { Icon } from './Icon'
import { TrendingDownIcon, TrendingUpIcon } from './Icons'
import { resolveIcon } from '../lib/icons'
import { Sparkline } from './Sparkline'

interface KpiCardProps {
  kpi: Kpi
  periodLabel: string
}

export function KpiCard({ kpi, periodLabel }: KpiCardProps) {
  const { value, unit } = formatKpi(kpi)
  const delta = kpi.delta
  const rising = (delta ?? 0) >= 0

  return (
    <section className="card col-3">
      <div className="kpi__head">
        <span className="kpi__icon">
          <Icon name={resolveIcon(kpi.icon)} size={17} />
        </span>
        <span className="kpi__label">{kpi.label}</span>
        {delta !== undefined ? (
          <span
            className={`delta ${rising ? 'delta--down' : 'delta--flat'}`}
            title={
              kpi.invertDelta
                ? 'Рост этого показателя считается негативным'
                : 'Изменение к прошлому периоду'
            }
          >
            {rising ? <TrendingUpIcon size={13} /> : <TrendingDownIcon size={13} />}
            {formatDelta(delta)}
          </span>
        ) : null}
      </div>

      <div className="kpi__value">
        {value}
        {unit ? <span className="kpi__unit"> {unit}</span> : null}
      </div>

      <div className="kpi__foot">
        <span className="kpi__unit">за {periodLabel}</span>
        {kpi.spark && kpi.spark.length > 1 ? <Sparkline points={kpi.spark} /> : null}
      </div>
    </section>
  )
}
