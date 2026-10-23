import { useRef, useState } from 'react'
import { AddCardDialog } from './components/AddCardDialog'
import { AddCardTile } from './components/AddCardTile'
import { AudioVisualizer } from './components/AudioVisualizer'
import { Header } from './components/Header'
import { StatusCard } from './components/StatusCard'
import { builtinCards, dashboardStatus } from './data/dashboard'
import { formatClock, opencodeSessionLines, renderCard } from './data/live'
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

  // Карточек OpenCode должно быть столько, сколько открыто сессий: каждая
  // отвечает на вопрос «что делает вот эта сессия прямо сейчас».
  const sessions = snapshot?.opencode.sessions ?? []
  const sessionCards: StatusCardData[] = sessions.map((session) => ({
    id: `opencode-${session.id}`,
    title: 'OpenCode',
    lines: opencodeSessionLines(session, sessions.length > 1),
  }))

  const shownCards = [
    ...sessionCards.filter((card) => !dismissed.includes(card.id)),
    ...cards,
  ]

  const dismissCard = (id: string) => {
    setAdded((list) => list.filter((card) => card.id !== id))

    // Карточки помним, чтобы «Вернуть карточки» их восстановило: это касается
    // и встроенных, и карточек сессий OpenCode — их список каждый раз
    // пересобирается из базы, и без этой памяти крестик ничего не закрывал.
    setDismissed((list) => (list.includes(id) ? list : [...list, id]))
  }

  const addCard = (card: StatusCardData) => {
    counter.current += 1
    setAdded((list) => [...list, { ...card, id: `custom-${counter.current}` }])
  }

  return (
    <div className="app">
      <div className="shell">
        <Header login={snapshot?.user.login ?? ''} status={dashboardStatus} />

        <main className="grid">
          {shownCards.map((card) => (
            <StatusCard key={card.id} card={card} onDismiss={dismissCard} />
          ))}

          <AddCardTile onClick={() => setDialogOpen(true)} />
        </main>

        <AudioVisualizer
          frame={audio}
          expected={snapshot?.music.expectedNotes ?? []}
        />

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

          <span className="statusbar__item">Карточек: {shownCards.length}</span>

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
