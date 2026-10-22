import { useEffect, useState } from 'react'
import { fetchAudioBars, hasTauri } from '../api/tauri'
import type { AudioBars } from '../types'

/** Как часто забираем новый кадр спектра: примерно 25 кадров в секунду. */
const FRAME_INTERVAL = 40

/** Пустой кадр: звука нет, полоса скрыта. */
const SILENT: AudioBars = { active: false, bars: [] }

/**
 * Следит за спектром системного звука. Захват живёт в Rust-слое,
 * здесь только забираются готовые полоски.
 */
export function useAudioBars(): AudioBars {
  const [frame, setFrame] = useState<AudioBars>(SILENT)

  useEffect(() => {
    if (!hasTauri()) return

    let alive = true
    let inFlight = false

    const poll = async () => {
      // Кадры не должны наслаиваться, если Rust не успевает отдать спектр.
      if (!alive || inFlight) return
      inFlight = true

      try {
        const next = await fetchAudioBars()
        if (alive) setFrame(next)
      } catch {
        // Захват звука — необязательная часть: молча прячем полосу.
        if (alive) setFrame(SILENT)
      } finally {
        inFlight = false
      }
    }

    void poll()
    const timer = window.setInterval(poll, FRAME_INTERVAL)

    return () => {
      alive = false
      window.clearInterval(timer)
    }
  }, [])

  return frame
}
