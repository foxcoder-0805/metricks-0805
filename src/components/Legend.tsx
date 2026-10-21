export interface LegendItem {
  label: string
  color: string
  value?: string
}

interface LegendProps {
  items: LegendItem[]
}

/** Легенда серий: точка цвета + подпись + последнее значение. */
export function Legend({ items }: LegendProps) {
  if (items.length === 0) return null

  return (
    <ul className="legend">
      {items.map((item) => (
        <li className="legend__item" key={item.label}>
          <span className="legend__dot" style={{ '--dot': item.color } as React.CSSProperties} />
          <span>{item.label}</span>
          {item.value ? <span className="legend__value">{item.value}</span> : null}
        </li>
      ))}
    </ul>
  )
}
