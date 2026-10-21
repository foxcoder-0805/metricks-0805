import type { Kpi, KpiFormat } from '../api/types'

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

export function formatBytes(bytes: number): { value: string; unit: string } {
  const units = ['Б', 'КБ', 'МБ', 'ГБ', 'ТБ']
  let value = bytes
  let index = 0
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024
    index += 1
  }
  return { value: formatNumber(value, value < 100), unit: units[index] }
}

/** Значение KPI в виде пары «число» + «единица», чтобы единица не дублировалась в разметке. */
export function formatKpi(kpi: Pick<Kpi, 'value' | 'format' | 'unit'>): { value: string; unit: string } {
  const format: KpiFormat = kpi.format ?? 'number'

  switch (format) {
    case 'percent':
      return { value: formatCompact(kpi.value), unit: kpi.unit ?? '%' }
    case 'bytes': {
      const { value, unit } = formatBytes(kpi.value)
      return { value, unit }
    }
    case 'duration': {
      if (kpi.value < 1000) return { value: formatNumber(kpi.value), unit: kpi.unit ?? 'мс' }
      return { value: formatNumber(kpi.value / 1000, true), unit: kpi.unit ?? 'с' }
    }
    default:
      return { value: formatNumber(kpi.value), unit: kpi.unit ?? '' }
  }
}

export function formatDelta(delta: number): string {
  const sign = delta > 0 ? '+' : ''
  return `${sign}${formatNumber(delta, true)}%`
}

export function formatTime(iso: string | Date): string {
  const date = typeof iso === 'string' ? new Date(iso) : iso
  if (Number.isNaN(date.getTime())) return '—'
  return date.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

export function formatClock(date: Date): string {
  return date.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}
