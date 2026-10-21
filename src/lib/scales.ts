/** Шкалы и генераторы путей для SVG-графиков. */

export type Point = [number, number]

export interface Scale {
  (value: number): number
  invert(pixel: number): number
}

export function linearScale(domain: [number, number], range: [number, number]): Scale {
  const [d0, d1] = domain
  const [r0, r1] = range
  const span = d1 - d0 || 1

  const scale = ((value: number) => r0 + ((value - d0) / span) * (r1 - r0)) as Scale
  scale.invert = (pixel: number) => d0 + ((pixel - r0) / (r1 - r0 || 1)) * span
  return scale
}

/**
 * Сглаженная кривая через точки (Catmull-Rom → кубические безье).
 * График из шаблона рисуется именно плавной волной, без углов.
 */
export function smoothPath(points: Point[], tension = 1): string {
  if (points.length === 0) return ''
  if (points.length === 1) return `M ${points[0][0]} ${points[0][1]}`

  let path = `M ${points[0][0]} ${points[0][1]}`

  for (let i = 0; i < points.length - 1; i += 1) {
    const p0 = points[i - 1] ?? points[i]
    const p1 = points[i]
    const p2 = points[i + 1]
    const p3 = points[i + 2] ?? p2

    const cp1x = p1[0] + ((p2[0] - p0[0]) / 6) * tension
    const cp1y = p1[1] + ((p2[1] - p0[1]) / 6) * tension
    const cp2x = p2[0] - ((p3[0] - p1[0]) / 6) * tension
    const cp2y = p2[1] - ((p3[1] - p1[1]) / 6) * tension

    path += ` C ${round(cp1x)} ${round(cp1y)}, ${round(cp2x)} ${round(cp2y)}, ${round(p2[0])} ${round(p2[1])}`
  }

  return path
}

/** Замкнутая область под кривой — с той же сглаженной верхней границей. */
export function areaPath(points: Point[], baseline: number): string {
  if (points.length === 0) return ''
  const line = smoothPath(points)
  const first = points[0]
  const last = points[points.length - 1]
  return `${line} L ${round(last[0])} ${round(baseline)} L ${round(first[0])} ${round(baseline)} Z`
}

/** Прямоугольник со скруглёнными верхними углами — столбцы не должны «стоять на ножке». */
export function roundedTopRect(x: number, y: number, width: number, height: number, radius: number): string {
  const r = Math.min(radius, width / 2, Math.max(0, height))
  const bottom = y + height
  return [
    `M ${round(x)} ${round(bottom)}`,
    `L ${round(x)} ${round(y + r)}`,
    `Q ${round(x)} ${round(y)} ${round(x + r)} ${round(y)}`,
    `L ${round(x + width - r)} ${round(y)}`,
    `Q ${round(x + width)} ${round(y)} ${round(x + width)} ${round(y + r)}`,
    `L ${round(x + width)} ${round(bottom)}`,
    'Z',
  ].join(' ')
}

/** Дуга для кольцевой диаграммы. */
export function arcPath(cx: number, cy: number, radius: number, startAngle: number, endAngle: number): string {
  const sweep = endAngle - startAngle
  if (Math.abs(sweep) < 0.0001) return ''

  const start = polar(cx, cy, radius, startAngle)
  const end = polar(cx, cy, radius, endAngle)
  const largeArc = Math.abs(sweep) > Math.PI ? 1 : 0

  return `M ${round(start.x)} ${round(start.y)} A ${radius} ${radius} 0 ${largeArc} 1 ${round(end.x)} ${round(end.y)}`
}

export function polar(cx: number, cy: number, radius: number, angle: number) {
  return {
    x: cx + radius * Math.cos(angle),
    y: cy + radius * Math.sin(angle),
  }
}

export function round(value: number): number {
  return Math.round(value * 100) / 100
}

/** Красивые круглые деления оси Y (1, 2, 5 × 10^n). */
export function niceTicks(max: number, count = 4): number[] {
  if (max <= 0) return [0]
  const rough = max / count
  const magnitude = 10 ** Math.floor(Math.log10(rough))
  const normalized = rough / magnitude
  const step = (normalized >= 5 ? 10 : normalized >= 2 ? 5 : normalized >= 1 ? 2 : 1) * magnitude

  const ticks: number[] = []
  for (let value = 0; value <= max + step / 2; value += step) {
    ticks.push(value)
  }
  return ticks
}
