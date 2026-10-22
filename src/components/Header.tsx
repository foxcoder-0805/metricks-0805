interface HeaderProps {
  /** Логин владельца сессии; пусто, пока снимок не пришёл. */
  login: string
  status: {
    label: string
  }
}

export function Header({ login, status }: HeaderProps) {
  const label = login || 'локальный сеанс'

  return (
    <header className="topbar">
      <div className="brand">
        <div className="brand__mark">M</div>
        <div>
          <h1 className="brand__title">Metricks</h1>
          <p className="brand__sub">доска статусов на рабочем столе</p>
        </div>
      </div>

      <div className="topbar__meta">
        <span className="chip chip--ok">
          <span className="chip__dot" />
          {status.label}
        </span>

        <div className="avatar" title={login ? `Сеанс ${login}` : 'Сеанс ещё не определён'}>
          <span className="avatar__login">{label}</span>
          <i className="avatar__dot" />
        </div>
      </div>
    </header>
  )
}
