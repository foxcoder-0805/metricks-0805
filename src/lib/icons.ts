import {
  ActivityIcon,
  AlertIcon,
  CalendarIcon,
  CheckIcon,
  ChevronDownIcon,
  ClockIcon,
  DatabaseIcon,
  GlobeIcon,
  RefreshIcon,
  ServerIcon,
  SettingsIcon,
  TrendingDownIcon,
  TrendingUpIcon,
  UsersIcon,
  ZapIcon,
} from '../components/Icons'

/** Реестр иконок: имя приходит из API, поэтому нужен безопасный lookup. */
export const ICONS = {
  activity: ActivityIcon,
  alert: AlertIcon,
  clock: ClockIcon,
  users: UsersIcon,
  server: ServerIcon,
  database: DatabaseIcon,
  globe: GlobeIcon,
  zap: ZapIcon,
  refresh: RefreshIcon,
  settings: SettingsIcon,
  calendar: CalendarIcon,
  trendingUp: TrendingUpIcon,
  trendingDown: TrendingDownIcon,
  chevronDown: ChevronDownIcon,
  check: CheckIcon,
} as const

export type IconName = keyof typeof ICONS

/** Неизвестное имя иконки не должно ломать рендер — подставляем заглушку. */
export function resolveIcon(name: string | undefined): IconName {
  return name && name in ICONS ? (name as IconName) : 'activity'
}
