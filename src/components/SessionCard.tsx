import type { Session } from '../api/types'

interface SessionCardProps {
  session: Session | null
}

/**
 * Большая карточка: детализация текущей задачи OpenCode —
 * крупный заголовок слева, описание справа, серая полоса-прогресс
 * и абзац с последними шагами агента.
 */
export function SessionCard({ session }: SessionCardProps) {
  if (!session) return null

  const percent = session.total > 0 ? Math.round((session.done / session.total) * 100) : 0

  return (
    <section className="card card--session col-8">
      <header className="session__head">
        <h2 className="session__task">{session.task}</h2>

        <div className="session__meta">
          <p className="session__summary">{session.summary}</p>

          <ul className="session__tags">
            {session.tags.map((tag) => (
              <li key={tag} className="session__tag">
                {tag}
              </li>
            ))}
          </ul>
        </div>
      </header>

      <div className="session__progress">
        <div className="session__bar">
          <span className="session__fill" style={{ width: `${percent}%` }} />
        </div>

        <span className="session__counter">
          {session.done} из {session.total} шагов
        </span>
      </div>

      <ul className="session__steps">
        {session.steps.map((step) => (
          <li key={`${step.at}-${step.text}`} className="session__step">
            <span className="session__time">{step.at}</span>
            <span className="session__text">{step.text}</span>
          </li>
        ))}
      </ul>
    </section>
  )
}
