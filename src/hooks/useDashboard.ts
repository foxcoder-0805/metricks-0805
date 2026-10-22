import { useCallback, useEffect, useState } from 'react'
import { api } from '../api/client'
import type { Range, Series, Summary } from '../api/types'

const POLL_INTERVAL = 15_000

export interface DashboardState {
  summary: Summary | null
  series: Series | null
  error: string | null
  loading: boolean
  updatedAt: Date | null
  refresh: () => void
}

/**
 * Загружает сводку и ряд динамики, обновляет их по таймеру.
 * При ошибке последние успешные данные остаются на экране — дашборд
 * не должен пустеть из-за одного сбоя сети.
 */
export function useDashboard(range: Range, apiBase: string): DashboardState {
  const [summary, setSummary] = useState<Summary | null>(null)
  const [series, setSeries] = useState<Series | null>(null)
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
        const [summaryData, seriesData] = await Promise.all([
          api.summary(controller.signal),
          api.series(range, controller.signal),
        ])

        if (cancelled) return

        setSummary(summaryData)
        setSeries(seriesData)
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

  return { summary, series, error, loading, updatedAt, refresh }
}
