import { useEffect, useRef, useState } from 'react'
import { fetchReferenceTimeline, hasTauri } from '../api/tauri'
import type { HallNote } from '../types'

/**
 * Таймлайн эталонного трека для фортепиано внизу окна.
 *
 * Отдаётся один раз, когда трек опознан: 3287 нот не должны гоняться через
 * IPC на каждый кадр — дальше позиция считается на фронте по отметке
 * времени снимка, а Rust опрашивает плеер раз в секунду и пересинхронизирует.
 * Без таймлайна (или без Tauri в браузере) возвращается `null`, и подвал
 * остаётся обычной полосой спектра.
 */
export function useReferenceTimeline(enabled: boolean): HallNote[] | null {
  const [notes, setNotes] = useState<HallNote[] | null>(null)
  const requested = useRef(false)

  useEffect(() => {
    if (!enabled || requested.current || !hasTauri()) return

    requested.current = true
    let alive = true

    fetchReferenceTimeline()
      .then((timeline) => {
        if (alive) setNotes(timeline)
      })
      .catch(() => {
        // Нет таймлайна — нет фортепиано: подвал честно остаётся кавой.
        requested.current = false
      })

    return () => {
      alive = false
    }
  }, [enabled])

  return enabled ? notes : null
}
