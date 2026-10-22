const numberFormat = new Intl.NumberFormat('ru-RU', { maximumFractionDigits: 0 })
const numberFormatPrecise = new Intl.NumberFormat('ru-RU', {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
})

export function formatNumber(value: number, precise = false): string {
  return precise ? numberFormatPrecise.format(value) : numberFormat.format(value)
}

/**
 * Компактное число для шкал и легенд: крупные значения без дробей,
 * мелкие — с двумя знаками, чтобы «0,56 %» не превратилось в «1 %».
 */
export function formatCompact(value: number): string {
  const absolute = Math.abs(value)
  const digits = absolute >= 10 ? 0 : absolute >= 1 ? 1 : 2
  return new Intl.NumberFormat('ru-RU', {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  }).format(value)
}

export function formatDelta(delta: number): string {
  const sign = delta > 0 ? '+' : ''
  return `${sign}${formatNumber(delta, true)}%`
}

export function formatClock(date: Date): string {
  return date.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}
