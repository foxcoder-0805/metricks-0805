import { useState } from 'react'
import { getApiBase, setApiBase } from './api/config'
import { RANGES, type Range } from './api/types'
import { AreaChart } from './components/AreaChart'
import { BarChart } from './components/BarChart'
import { Card } from './components/Card'
import { DataTable } from './components/DataTable'
import { DonutChart } from './components/DonutChart'
import { Header } from './components/Header'
import { KpiCard } from './components/KpiCard'
import { RadialGauge } from './components/RadialGauge'
import { useDashboard } from './hooks/useDashboard'
import { formatClock } from './lib/format'

export default function App() {
  const [range, setRange] = useState<Range>('24h')
  const [apiBase, setApiBaseState] = useState(() => getApiBase())
  const { summary, series, bars, error, loading, updatedAt, refresh } = useDashboard(range, apiBase)

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
              Проверьте адрес <code>{apiBase}</code> или запустите локальный мок:{' '}
              <code>npm run mock</code>
            </p>
            <button type="button" className="btn" onClick={refresh}>
              Повторить
            </button>
          </div>
        ) : (
          <main className="grid">
            {(summary?.kpis ?? []).map((kpi) => (
              <KpiCard key={kpi.id} kpi={kpi} periodLabel={periodLabel} />
            ))}

            <Card
              className="col-8"
              title="Трафик"
              subtitle={`Динамика по источникам за ${periodLabel}`}
            >
              <AreaChart data={series} height={280} unit="req/s" />
            </Card>

            <Card className="col-4" title={summary?.donuts[0]?.label ?? 'Источники трафика'}>
              <DonutChart donut={summary?.donuts[0] ?? null} />
            </Card>

            <Card className="col-8" title="Нагрузка на хранилище" subtitle={`Чтения и записи за ${periodLabel}`}>
              <BarChart data={bars} height={240} unit="ops" />
            </Card>

            <Card className="col-4" title="Ресурсы кластера" subtitle="Текущая загрузка">
              <div className="gauge__grid">
                {(summary?.gauges ?? []).map((gauge) => (
                  <RadialGauge key={gauge.id} gauge={gauge} />
                ))}
              </div>
            </Card>

            <Card
              className="col-12"
              title={summary?.table.id === 'regions' ? 'Регионы' : 'Детализация'}
              subtitle="Задержка p95 и количество ошибок по площадкам"
            >
              <DataTable table={summary?.table ?? null} />
            </Card>
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
