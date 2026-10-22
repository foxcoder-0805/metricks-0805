import { useRef, useState } from 'react'
import { AddCardDialog } from './components/AddCardDialog'
import { AddCardTile } from './components/AddCardTile'
import { Header } from './components/Header'
import { StatusCard } from './components/StatusCard'
import { dashboardStatus, dashboardUser, initialCards } from './data/dashboard'
import type { StatusCard as StatusCardData } from './types'

export default function App() {
  const [dismissed, setDismissed] = useState<string[]>([])
  const [added, setAdded] = useState<StatusCardData[]>([])
  const [dialogOpen, setDialogOpen] = useState(false)
  const counter = useRef(0)

  const cards = [...initialCards.filter((card) => !dismissed.includes(card.id)), ...added]

  const dismissCard = (id: string) => {
    setAdded((list) => list.filter((card) => card.id !== id))

    // встроенные карточки помним, чтобы «Вернуть карточки» их восстановило
    if (initialCards.some((card) => card.id === id)) {
      setDismissed((list) => (list.includes(id) ? list : [...list, id]))
    }
  }

  const addCard = (card: StatusCardData) => {
    counter.current += 1
    setAdded((list) => [...list, { ...card, id: `custom-${counter.current}` }])
  }

  return (
    <div className="app">
      <div className="shell">
        <Header user={dashboardUser} status={dashboardStatus} />

        <main className="grid">
          {cards.map((card) => (
            <StatusCard key={card.id} card={card} onDismiss={dismissCard} />
          ))}

          <AddCardTile onClick={() => setDialogOpen(true)} />
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

      {dialogOpen ? (
        <AddCardDialog onClose={() => setDialogOpen(false)} onAdd={addCard} />
      ) : null}
    </div>
  )
}
