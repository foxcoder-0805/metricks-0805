import { useState } from 'react'
import { getApiBase, setApiBase } from './api/config'
import { Header } from './components/Header'
import { SessionCard } from './components/SessionCard'
import { StatusCard } from './components/StatusCard'
import { useDashboard } from './hooks/useDashboard'
import { formatClock } from './lib/format'

export default function App() {
  const [apiBase, setApiBaseState] = useState(() => getApiBase())
  const [dismissed, setDismissed] = useState<string[]>([])
  const { summary, error, loading, updatedAt, refresh } = useDashboard(apiBase)

  const cards = (summary?.cards ?? []).filter((card) => !dismissed.includes(card.id))

  const handleApiBaseChange = (base: string) => {
    setApiBase(base)
    setApiBaseState(getApiBase())
  }

  const dismissCard = (id: string) => {
    setDismissed((list) => (list.includes(id) ? list : [...list, id]))
  }

  return (
    <div className="app">
      <div className="shell">
        <Header
          user={summary?.user ?? null}
          status={summary?.status ?? null}
          onRefresh={refresh}
          loading={loading}
          connected={!error}
          apiBase={apiBase}
          onApiBaseChange={handleApiBaseChange}
        />

        {error && !summary ? (
          <div className="overlay">
            <div className="overlay__title">Нет связи с источником данных</div>
            <p className="overlay__hint">
              {error}
              <br />
              Проверьте адрес <code>{apiBase}</code> в настройках источника данных.
            </p>
            <button type="button" className="btn" onClick={refresh}>
              Повторить
            </button>
          </div>
        ) : (
          <main className="grid">
            {cards.map((card) => (
              <StatusCard key={card.id} card={card} onDismiss={dismissCard} />
            ))}

            <SessionCard session={summary?.session ?? null} />
          </main>
        )}

        <footer className="statusbar">
          <span className="statusbar__item">
            <span className="chip__dot" />
            {updatedAt ? `Обновлено в ${formatClock(updatedAt)}` : 'Ожидание данных'}
          </span>
          <span className="statusbar__item">API: {apiBase}</span>
          {error ? <span className="statusbar__item">⚠ {error}</span> : null}
          {dismissed.length > 0 ? (
            <button type="button" className="statusbar__link" onClick={() => setDismissed([])}>
              Вернуть карточки
            </button>
          ) : null}
        </footer>
      </div>
    </div>
  )
}
