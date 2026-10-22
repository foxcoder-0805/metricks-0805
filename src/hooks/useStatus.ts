import { useEffect, useState } from 'react'
import { fetchStatus, hasTauri } from '../api/tauri'
import type { StatusSnapshot } from '../types'

/** Как часто перечитываем состояние машины. */
const POLL_INTERVAL = 1000

export interface StatusState {
  snapshot: StatusSnapshot | null
  error: string | null
}

/**
 * Опрашивает Rust-слой с реальным состоянием машины. При ошибке оставляет
 * последний удачный снимок: лучше показать чуть устаревшие данные, чем пустоту.
 */
export function useStatus(): StatusState {
  const [snapshot, setSnapshot] = useState<StatusSnapshot | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!hasTauri()) {
      setError('нет доступа к системе')
      return
    }

    let alive = true
    let inFlight = false

    const poll = async () => {
      // Секундный опрос не должен наслаиваться, если сборка снимка затянулась.
      if (!alive || inFlight) return
      inFlight = true

      try {
        const next = await fetchStatus()
        if (!alive) return
        setSnapshot(next)
        setError(null)
      } catch (cause) {
        if (!alive) return
        setError(cause instanceof Error ? cause.message : String(cause))
      } finally {
        inFlight = false
      }
    }

    void poll()
    const timer = window.setInterval(poll, POLL_INTERVAL)

    return () => {
      alive = false
      window.clearInterval(timer)
    }
  }, [])

  return { snapshot, error }
}
