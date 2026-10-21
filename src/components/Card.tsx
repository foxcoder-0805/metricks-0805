import type { ReactNode } from 'react'

interface CardProps {
  title?: string
  subtitle?: string
  /** Класс колонки сетки: col-3 | col-4 | col-7 | col-8 | col-12 */
  className?: string
  actions?: ReactNode
  children: ReactNode
}

/** Скруглённая карточка — базовый контейнер дашборда. */
export function Card({ title, subtitle, className = 'col-12', actions, children }: CardProps) {
  return (
    <section className={`card ${className}`}>
      {title || actions ? (
        <header className="card__head">
          <div>
            <h2 className="card__title">{title}</h2>
            {subtitle ? <p className="card__sub">{subtitle}</p> : null}
          </div>
          {actions}
        </header>
      ) : null}
      <div className="card__body">{children}</div>
    </section>
  )
}
