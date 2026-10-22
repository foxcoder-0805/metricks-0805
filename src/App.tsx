import { useRef, useState } from 'react'
import { AddCardDialog } from './components/AddCardDialog'
import { AddCardTile } from './components/AddCardTile'
import { AudioVisualizer } from './components/AudioVisualizer'
import { Header } from './components/Header'
import { StatusCard } from './components/StatusCard'
import { builtinCards, dashboardStatus, dashboardUser } from './data/dashboard'
import { formatClock, renderCard } from './data/live'
import { useAudioBars } from './hooks/useAudioBars'
import { useStatus } from './hooks/useStatus'
import type { StatusCard as StatusCardData } from './types'

export default function App() {
  const { snapshot, error } = useStatus()
  const audio = useAudioBars()
  const [dismissed, setDismissed] = useState<string[]>([])
  const [added, setAdded] = useState<StatusCardData[]>([])
  const [dialogOpen, setDialogOpen] = useState(false)
  const counter = useRef(0)

  const cards = [...builtinCards.filter((card) => !dismissed.includes(card.id)), ...added].map(
    (card) => renderCard(card, snapshot),
  )

  const dismissCard = (id: string) => {
    setAdded((list) => list.filter((card) => card.id !== id))

    // встроенные карточки помним, чтобы «Вернуть карточки» их восстановило
    if (builtinCards.some((card) => card.id === id)) {
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

        <AudioVisualizer frame={audio} />

        <footer className="statusbar">
          <span className="statusbar__item">
            <span className="chip__dot" />
            {error
              ? snapshot
                ? 'связь с машиной потеряна'
                : 'нет данных с машины'
              : 'живые данные с машины'}
          </span>

          <span className="statusbar__item">
            {snapshot ? `обновлено в ${formatClock(snapshot.collectedAt)}` : 'ждём данные'}
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
        <AddCardDialog
          snapshot={snapshot}
          onClose={() => setDialogOpen(false)}
          onAdd={addCard}
        />
      ) : null}
    </div>
  )
}
