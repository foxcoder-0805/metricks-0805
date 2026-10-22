import { PlusIcon } from './Icons'

interface AddCardTileProps {
  onClick: () => void
}

/**
 * Плитка «+» в нижнем ряду: повторяет анатомию статус-карточки,
 * но вместо данных предлагает добавить новую карточку.
 */
export function AddCardTile({ onClick }: AddCardTileProps) {
  return (
    <button type="button" className="card card--add col-4" onClick={onClick}>
      <span className="add__plus" aria-hidden="true">
        <PlusIcon size={30} />
      </span>

      <span className="status__caption add__caption">
        <span className="status__line">Добавить карточку</span>
        <span className="status__line status__line--dim">приложение и что отслеживать</span>
      </span>
    </button>
  )
}
