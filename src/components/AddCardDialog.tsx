import { useEffect, useRef, useState } from 'react'
import { appCatalog } from '../data/apps'
import type { StatusCard } from '../types'
import { CloseIcon } from './Icons'

interface AddCardDialogProps {
  onClose: () => void
  onAdd: (card: StatusCard) => void
}

/**
 * Окно добавления карточки: сначала выбирается приложение,
 * затем — что именно за ним следить.
 */
export function AddCardDialog({ onClose, onAdd }: AddCardDialogProps) {
  const [appId, setAppId] = useState<string | null>(null)
  const [metricId, setMetricId] = useState<string | null>(null)
  const panelRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose()
    }

    document.addEventListener('keydown', onKeyDown)
    panelRef.current?.focus()

    return () => document.removeEventListener('keydown', onKeyDown)
  }, [onClose])

  const app = appCatalog.find((item) => item.id === appId) ?? null
  const metric = app?.metrics.find((item) => item.id === metricId) ?? null

  const handleAdd = () => {
    if (!app || !metric) return

    onAdd({ id: '', title: app.title, lines: [metric.value] })
    onClose()
  }

  return (
    <div className="modal" onClick={onClose}>
      <div
        className="modal__panel"
        role="dialog"
        aria-modal="true"
        aria-labelledby="add-card-title"
        tabIndex={-1}
        ref={panelRef}
        onClick={(event) => event.stopPropagation()}
      >
        <header className="modal__head">
          <h2 className="modal__title" id="add-card-title">
            Новая карточка
          </h2>

          <button
            type="button"
            className="icon-btn"
            onClick={onClose}
            title="Закрыть"
            aria-label="Закрыть"
          >
            <CloseIcon size={16} />
          </button>
        </header>

        <section className="modal__section">
          <p className="modal__label">Приложение</p>

          <div className="modal__apps">
            {appCatalog.map((item) => (
              <button
                key={item.id}
                type="button"
                className={`chip-btn${item.id === appId ? ' chip-btn--active' : ''}`}
                onClick={() => {
                  setAppId(item.id)
                  setMetricId(null)
                }}
              >
                {item.title}
              </button>
            ))}
          </div>
        </section>

        <section className="modal__section">
          <p className="modal__label">Что отслеживать</p>

          {app ? (
            <div className="modal__metrics">
              {app.metrics.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  className={`metric${item.id === metricId ? ' metric--active' : ''}`}
                  onClick={() => setMetricId(item.id)}
                >
                  <span className="metric__label">{item.label}</span>
                  <span className="metric__value">{item.value}</span>
                </button>
              ))}
            </div>
          ) : (
            <p className="modal__placeholder">Сначала выберите приложение</p>
          )}
        </section>

        <footer className="modal__foot">
          <button type="button" className="btn btn--ghost" onClick={onClose}>
            Отмена
          </button>

          <button type="button" className="btn" disabled={!metric} onClick={handleAdd}>
            Добавить
          </button>
        </footer>
      </div>
    </div>
  )
}
