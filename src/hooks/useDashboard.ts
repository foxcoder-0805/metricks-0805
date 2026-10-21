import { useCallback, useEffect, useState } from 'react'
import { api } from '../api/client'
import type { Bars, Range, Series, Summary } from '../api/types'

const POLL_INTERVAL = 15_000

export interface DashboardState {
  summary: Summary | null
  series: Series | null
  bars: Bars | null
  error: string | null
  loading: boolean
  updatedAt: Date | null
  refresh: () => void
}

/**
 * Загружает все три среза данных и обновляет их по таймеру.
 * При ошибке последние успешные данные остаются на экране — дашборд
 * не должен пустеть из-за одного сбоя сети.
 */
export function useDashboard(range: Range, apiBase: string): DashboardState {
  const [summary, setSummary] = useState<Summary | null>(null)
  const [series, setSeries] = useState<Series | null>(null)
  const [bars, setBars] = useState<Bars | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [updatedAt, setUpdatedAt] = useState<Date | null>(null)
  const [nonce, setNonce] = useState(0)

  const refresh = useCallback(() => setNonce((value) => value + 1), [])

  useEffect(() => {
    let cancelled = false
    const controller = new AbortController()

    const load = async () => {
      try {
        const [summaryData, seriesData, barsData] = await Promise.all([
          api.summary(controller.signal),
          api.series(range, controller.signal),
          api.bars(range, controller.signal),
        ])

        if (cancelled) return

        setSummary(summaryData)
        setSeries(seriesData)
        setBars(barsData)
        setError(null)
        setUpdatedAt(new Date())
      } catch (caught) {
        if (cancelled) return
        if (caught instanceof DOMException && caught.name === 'AbortError') return
        setError(caught instanceof Error ? caught.message : 'Неизвестная ошибка')
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    void load()
    const timer = window.setInterval(() => void load(), POLL_INTERVAL)

    return () => {
      cancelled = true
      controller.abort()
      window.clearInterval(timer)
    }
  }, [range, apiBase, nonce])

  return { summary, series, bars, error, loading, updatedAt, refresh }
}
