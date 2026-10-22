import type { SourceCard as SourceCardData } from '../api/types'
import { formatDelta, formatNumber } from '../lib/format'
import { ArrowDownRightIcon, ArrowUpRightIcon } from './Icons'
import { Sparkline } from './Sparkline'

interface SourceCardProps {
  source: SourceCardData
}

/**
 * Карточка источника повторяет карточку из шаблона:
 * крупное название в левом верхнем углу, круглый бейдж со стрелкой тренда
 * в правом верхнем, подпись — внизу слева.
 */
export function SourceCard({ source }: SourceCardProps) {
  const delta = source.delta ?? 0
  const rising = delta >= 0

  return (
    <section className="card card--source col-4">
      <div className="source__top">
        <h2 className="source__title">{source.title}</h2>

        <span
          className={`source__badge${rising ? ' source__badge--good' : ' source__badge--bad'}`}
          title={rising ? 'Показатель растёт' : 'Показатель снижается'}
        >
          {rising ? <ArrowUpRightIcon size={18} /> : <ArrowDownRightIcon size={18} />}
        </span>
      </div>

      {source.spark && source.spark.length > 1 ? (
        <div className="source__spark">
          <Sparkline points={source.spark} height={44} />
        </div>
      ) : null}

      <div className="source__caption">
        <p className="source__headline">{source.headline}</p>

        <p className="source__meta">
          <span className="source__value">{formatNumber(source.value)}</span>
          <span className="source__unit">{source.unit}</span>

          <span className={`source__delta${rising ? ' source__delta--good' : ' source__delta--bad'}`}>
            {rising ? <ArrowUpRightIcon size={12} /> : <ArrowDownRightIcon size={12} />}
            {formatDelta(delta)}
          </span>

          {source.note ? <span className="source__note">{source.note}</span> : null}
        </p>
      </div>
    </section>
  )
}
