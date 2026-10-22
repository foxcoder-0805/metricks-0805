import type { StatusCard as StatusCardData } from '../api/types'
import { CloseIcon } from './Icons'

interface StatusCardProps {
  card: StatusCardData
  /** Крест в правом верхнем углу убирает карточку. */
  onDismiss: (id: string) => void
}

/**
 * Карточка живого статуса: крупное слово в левом верхнем углу,
 * круглый крест в правом верхнем и подпись внизу слева.
 */
export function StatusCard({ card, onDismiss }: StatusCardProps) {
  return (
    <section className="card card--status col-4">
      <div className="status__top">
        <h2 className="status__title">{card.title}</h2>

        <button
          type="button"
          className="status__close"
          onClick={() => onDismiss(card.id)}
          title="Убрать карточку"
          aria-label={`Убрать карточку «${card.title}»`}
        >
          <CloseIcon size={16} />
        </button>
      </div>

      <div className="status__caption">
        {card.lines.map((line) => (
          <p key={line} className="status__line">
            {line}
          </p>
        ))}
      </div>
    </section>
  )
}
