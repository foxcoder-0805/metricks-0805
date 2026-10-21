import { useEffect, useRef, useState } from 'react'
import { RANGES, type HealthState, type Range, type Summary } from '../api/types'
import { CalendarIcon, RefreshIcon, SettingsIcon } from './Icons'

interface HeaderProps {
  user: Summary['user'] | null
  status: Summary['status'] | null
  range: Range
  onRangeChange: (range: Range) => void
  onRefresh: () => void
  loading: boolean
  connected: boolean
  apiBase: string
  onApiBaseChange: (base: string) => void
}

export function Header({
  user,
  status,
  range,
  onRangeChange,
  onRefresh,
  loading,
  connected,
  apiBase,
  onApiBaseChange,
}: HeaderProps) {
  const [panelOpen, setPanelOpen] = useState(false)
  const panelRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!panelOpen) return

    const onPointerDown = (event: MouseEvent) => {
      if (!panelRef.current?.contains(event.target as Node)) setPanelOpen(false)
    }

    document.addEventListener('mousedown', onPointerDown)
    return () => document.removeEventListener('mousedown', onPointerDown)
  }, [panelOpen])

  const chipState: HealthState = connected ? (status?.state ?? 'ok') : 'error'
  const chipLabel = connected ? (status?.label ?? 'В норме') : 'Нет соединения'

  return (
    <header className="topbar">
      <div className="brand">
        <div className="brand__mark">M</div>
        <div>
          <h1 className="brand__title">Metricks</h1>
          <p className="brand__sub">панель метрик в реальном времени</p>
        </div>
      </div>

      <div className="topbar__meta">
        <label className="pill">
          <CalendarIcon size={16} />
          <select value={range} onChange={(event) => onRangeChange(event.target.value as Range)}>
            {RANGES.map((item) => (
              <option key={item.id} value={item.id}>
                {item.label}
              </option>
            ))}
          </select>
        </label>

        <span className={`chip chip--${chipState}`}>
          <span className="chip__dot" />
          {chipLabel}
        </span>

        <button
          type="button"
          className={`icon-btn${loading ? ' icon-btn--busy' : ''}`}
          onClick={onRefresh}
          title="Обновить данные"
          aria-label="Обновить данные"
        >
          <RefreshIcon size={17} />
        </button>

        <div className="endpoint" ref={panelRef}>
          <button
            type="button"
            className="icon-btn"
            onClick={() => setPanelOpen((open) => !open)}
            title="Настройки источника данных"
            aria-label="Настройки источника данных"
            aria-expanded={panelOpen}
          >
            <SettingsIcon size={17} />
          </button>

          {panelOpen ? (
            // key по apiBase: смена адреса пересоздаёт панель и сбрасывает черновик
            <EndpointPanel
              key={apiBase}
              apiBase={apiBase}
              onApply={(base) => {
                onApiBaseChange(base)
                setPanelOpen(false)
              }}
            />
          ) : null}
        </div>

        <div className="avatar" title={user ? `${user.name}${user.role ? ` · ${user.role}` : ''}` : 'Metricks'}>
          <span>{user?.initials ?? 'M'}</span>
          <i className="avatar__dot" />
        </div>
      </div>
    </header>
  )
}

interface EndpointPanelProps {
  apiBase: string
  onApply: (base: string) => void
}

function EndpointPanel({ apiBase, onApply }: EndpointPanelProps) {
  const [draft, setDraft] = useState(apiBase)

  return (
    <div className="endpoint__panel">
      <label className="endpoint__label" htmlFor="endpoint-url">
        Базовый URL API
      </label>
      <div className="endpoint__row">
        <input
          id="endpoint-url"
          className="endpoint__input"
          value={draft}
          spellCheck={false}
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') onApply(draft)
          }}
        />
        <button type="button" className="btn" onClick={() => onApply(draft)}>
          OK
        </button>
      </div>
    </div>
  )
}
