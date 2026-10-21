import { useLayoutEffect, useRef, useState } from 'react'

export interface ElementSize {
  width: number
  height: number
}

/**
 * Реальный размер контейнера в пикселях.
 * Графики рисуются в координатах пикселей, а не в viewBox — иначе подписи
 * и толщина линий «плывут» при растягивании окна.
 */
export function useElementSize<T extends HTMLElement>() {
  const ref = useRef<T | null>(null)
  const [size, setSize] = useState<ElementSize>({ width: 0, height: 0 })

  useLayoutEffect(() => {
    const element = ref.current
    if (!element) return

    const observer = new ResizeObserver((entries) => {
      const rect = entries[0]?.contentRect
      if (!rect) return
      setSize((previous) => {
        const width = Math.round(rect.width)
        const height = Math.round(rect.height)
        if (previous.width === width && previous.height === height) return previous
        return { width, height }
      })
    })

    observer.observe(element)
    return () => observer.disconnect()
  }, [])

  return { ref, ...size }
}
