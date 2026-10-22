import type { Kpi } from '../api/types'
import { formatDelta, formatKpi } from '../lib/format'
import { Icon } from './Icon'
import { ArrowDownRightIcon, ArrowUpRightIcon } from './Icons'
import { resolveIcon } from '../lib/icons'
import { Sparkline } from './Sparkline'

interface KpiCardProps {
  kpi: Kpi
  periodLabel: string
}

/**
 * KPI-карточка повторяет карточку из шаблона:
 * крупное значение в левом верхнем углу, круглый бейдж со стрелкой тренда
 * в правом верхнем, подпись и дельта — внизу слева.
 */
export function KpiCard({ kpi, periodLabel }: KpiCardProps) {
  const { value, unit } = formatKpi(kpi)
  const delta = kpi.delta
  const rising = (delta ?? 0) >= 0
  // invertDelta: рост показателя считается ухудшением (ошибки, задержка)
  const good = kpi.invertDelta ? !rising : rising

  return (
    <section className="card card--kpi col-3">
      <div className="kpi__top">
        <div className="kpi__value">
          {value}
          {unit ? <span className="kpi__unit">{unit}</span> : null}
        </div>

        <span
          className={`kpi__badge${good ? ' kpi__badge--good' : ' kpi__badge--bad'}`}
          title={rising ? 'Показатель растёт' : 'Показатель снижается'}
        >
          {rising ? <ArrowUpRightIcon size={18} /> : <ArrowDownRightIcon size={18} />}
        </span>
      </div>

      <div className="kpi__caption">
        <span className="kpi__label">
          <Icon name={resolveIcon(kpi.icon)} size={14} />
          {kpi.label}
        </span>

        <span className="kpi__meta">
          {delta !== undefined ? (
            <span className={`kpi__delta${good ? ' kpi__delta--good' : ' kpi__delta--bad'}`}>
              {rising ? <ArrowUpRightIcon size={12} /> : <ArrowDownRightIcon size={12} />}
              {formatDelta(delta)}
            </span>
          ) : null}
          <span className="kpi__period">за {periodLabel}</span>
        </span>
      </div>

      {kpi.spark && kpi.spark.length > 1 ? (
        <div className="kpi__spark">
          <Sparkline points={kpi.spark} height={34} />
        </div>
      ) : null}
    </section>
  )
}
