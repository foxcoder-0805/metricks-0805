import { useState } from 'react'
import { getApiBase, setApiBase } from './api/config'
import { RANGES, type Range } from './api/types'
import { Header } from './components/Header'
import { OverviewCard } from './components/OverviewCard'
import { SourceCard } from './components/SourceCard'
import { useDashboard } from './hooks/useDashboard'
import { formatClock } from './lib/format'

export default function App() {
  const [range, setRange] = useState<Range>('24h')
  const [apiBase, setApiBaseState] = useState(() => getApiBase())
  const { summary, series, error, loading, updatedAt, refresh } = useDashboard(range, apiBase)

  const periodLabel = RANGES.find((item) => item.id === range)?.label ?? '24 часа'

  const handleApiBaseChange = (base: string) => {
    setApiBase(base)
    setApiBaseState(getApiBase())
  }

  return (
    <div className="app">
      <div className="shell">
        <Header
          user={summary?.user ?? null}
          status={summary?.status ?? null}
          range={range}
          onRangeChange={setRange}
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
            {(summary?.sources ?? []).map((source) => (
              <SourceCard key={source.id} source={source} />
            ))}

            <OverviewCard overview={summary?.overview ?? null} series={series} periodLabel={periodLabel} unit="req/s" />
          </main>
        )}

        <footer className="statusbar">
          <span className="statusbar__item">
            <span className="chip__dot" />
            {updatedAt ? `Обновлено в ${formatClock(updatedAt)}` : 'Ожидание данных'}
          </span>
          <span className="statusbar__item">API: {apiBase}</span>
          {error ? <span className="statusbar__item">⚠ {error}</span> : null}
        </footer>
      </div>
    </div>
  )
}
