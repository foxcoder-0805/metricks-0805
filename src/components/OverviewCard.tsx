import type { Overview as OverviewData, Series } from '../api/types'
import { formatNumber } from '../lib/format'
import { AreaChart } from './AreaChart'

interface OverviewCardProps {
  overview: OverviewData | null
  series: Series | null
  periodLabel: string
  unit?: string
}

/**
 * Большая карточка из шаблона: крупный показатель слева, подпись справа,
 * график в центре и абзац-описание внизу.
 */
export function OverviewCard({ overview, series, periodLabel, unit }: OverviewCardProps) {
  if (!overview) return null

  return (
    <section className="card card--overview col-8">
      <header className="overview__head">
        <div className="overview__figure">
          <span className="overview__value">{formatNumber(overview.value)}</span>
          <span className="overview__unit">{overview.unit}</span>
        </div>

        <div className="overview__text">
          <h2 className="overview__title">{overview.title}</h2>
          <p className="overview__label">
            {overview.label} за {periodLabel}
          </p>
        </div>
      </header>

      <div className="overview__chart">
        <AreaChart data={series} height={300} unit={unit} />
      </div>

      <p className="overview__note">{overview.description}</p>
    </section>
  )
}
