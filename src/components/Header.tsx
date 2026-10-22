interface HeaderProps {
  user: {
    name: string
    initials: string
    role?: string
  }
  status: {
    label: string
  }
}

export function Header({ user, status }: HeaderProps) {
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

        <div className="avatar" title={`${user.name}${user.role ? ` · ${user.role}` : ''}`}>
          <span>{user.initials}</span>
          <i className="avatar__dot" />
        </div>
      </div>
    </header>
  )
}
