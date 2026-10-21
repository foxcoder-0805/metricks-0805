import type { TableData } from '../api/types'
import { formatNumber } from '../lib/format'

interface DataTableProps {
  table: TableData | null
}

export function DataTable({ table }: DataTableProps) {
  if (!table || table.rows.length === 0) {
    return <div className="chart__empty">Нет данных</div>
  }

  return (
    <div className="table-wrap">
      <table className="table">
        <thead>
          <tr>
            {table.columns.map((column) => (
              <th key={column}>{column}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {table.rows.map((row) => (
            <tr key={String(row[0])}>
              {row.map((cell, index) => (
                <td key={index} className={typeof cell === 'number' ? 'num' : undefined}>
                  {typeof cell === 'number' ? formatNumber(cell) : cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
