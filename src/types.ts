/** Карточка живого статуса. */
export interface StatusCard {
  id: string
  /** Крупный заголовок карточки: OpenCode, Музыка, Steam. */
  title: string
  /** Строки подписи внизу слева: одна или две. */
  lines: string[]
}

/** Показатель приложения, за которым можно следить. */
export interface MetricOption {
  id: string
  /** Название показателя в списке выбора. */
  label: string
  /** Статичная подпись, которая попадёт в карточку. */
  value: string
}

/** Приложение, доступное для добавления на доску. */
export interface AppOption {
  id: string
  /** Крупное слово в карточке. */
  title: string
  /** Что можно отслеживать у этого приложения. */
  metrics: MetricOption[]
}
