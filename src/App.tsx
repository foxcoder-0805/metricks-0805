import { useState } from 'react'
import { Header } from './components/Header'
import { StatusCard } from './components/StatusCard'
import { dashboardStatus, dashboardUser, initialCards } from './data/dashboard'

export default function App() {
  const [dismissed, setDismissed] = useState<string[]>([])

  const cards = initialCards.filter((card) => !dismissed.includes(card.id))

  const dismissCard = (id: string) => {
    setDismissed((list) => (list.includes(id) ? list : [...list, id]))
  }

  return (
    <div className="app">
      <div className="shell">
        <Header user={dashboardUser} status={dashboardStatus} />

        <main className="grid">
          {cards.map((card) => (
            <StatusCard key={card.id} card={card} onDismiss={dismissCard} />
          ))}
        </main>

        <footer className="statusbar">
          <span className="statusbar__item">
            <span className="chip__dot" />
            Данные статичны
          </span>

          <span className="statusbar__item">Карточек: {cards.length}</span>

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
