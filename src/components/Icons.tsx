import type { SVGProps } from 'react'

export interface IconProps extends Omit<SVGProps<SVGSVGElement>, 'children'> {
  size?: number
}

function Svg({ size = 18, children, ...props }: IconProps & { children: React.ReactNode }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.7}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {children}
    </svg>
  )
}

export const ActivityIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="M3 12h4l2.5-7 4 14 2.5-7H21" />
  </Svg>
)

export const AlertIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="M10.3 4.3 2.6 17.5A1.6 1.6 0 0 0 4 20h16a1.6 1.6 0 0 0 1.4-2.5L13.7 4.3a2 2 0 0 0-3.4 0Z" />
    <path d="M12 9v4.5" />
    <path d="M12 16.8h.01" />
  </Svg>
)

export const ClockIcon = (props: IconProps) => (
  <Svg {...props}>
    <circle cx="12" cy="12" r="8.5" />
    <path d="M12 7.5V12l3 2" />
  </Svg>
)

export const UsersIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="M16 20v-1.5a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4V20" />
    <circle cx="9" cy="7" r="3.2" />
    <path d="M22 20v-1.5a4 4 0 0 0-3-3.87" />
    <path d="M16 4.13a4 4 0 0 1 0 7.75" />
  </Svg>
)

export const ServerIcon = (props: IconProps) => (
  <Svg {...props}>
    <rect x="3" y="4" width="18" height="6.5" rx="2" />
    <rect x="3" y="13.5" width="18" height="6.5" rx="2" />
    <path d="M7 7.2h.01M7 16.7h.01" />
  </Svg>
)

export const DatabaseIcon = (props: IconProps) => (
  <Svg {...props}>
    <ellipse cx="12" cy="6" rx="7.5" ry="3" />
    <path d="M4.5 6v12c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3V6" />
    <path d="M4.5 12c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3" />
  </Svg>
)

export const GlobeIcon = (props: IconProps) => (
  <Svg {...props}>
    <circle cx="12" cy="12" r="8.5" />
    <path d="M3.5 12h17" />
    <path d="M12 3.5c2.2 2.4 3.4 5.4 3.4 8.5S14.2 18.1 12 20.5c-2.2-2.4-3.4-5.4-3.4-8.5S9.8 5.9 12 3.5Z" />
  </Svg>
)

export const ZapIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="M13.5 2.5 4.8 13.2a.6.6 0 0 0 .47.98H11l-.5 7.3 8.7-10.7a.6.6 0 0 0-.47-.98H13l.5-7.3Z" />
  </Svg>
)

export const RefreshIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="M20.5 12a8.5 8.5 0 1 1-2.6-6.1" />
    <path d="M20.8 4.5v5.2h-5.2" />
  </Svg>
)

export const SettingsIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="M4 7h9M17 7h3M4 17h3M11 17h9" />
    <circle cx="15" cy="7" r="2.4" />
    <circle cx="9" cy="17" r="2.4" />
  </Svg>
)

export const CalendarIcon = (props: IconProps) => (
  <Svg {...props}>
    <rect x="3.5" y="5" width="17" height="15.5" rx="2.4" />
    <path d="M3.5 9.5h17M8 3v4M16 3v4" />
  </Svg>
)

export const TrendingUpIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="M3.5 16.5 9 11l3.5 3.5L20.5 6.5" />
    <path d="M15.5 6.5h5v5" />
  </Svg>
)

export const TrendingDownIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="M3.5 7.5 9 13l3.5-3.5L20.5 17.5" />
    <path d="M15.5 17.5h5v-5" />
  </Svg>
)

/**
 * Стрелка вверх-вправо — как в шаблоне: диагональ со шляпкой,
 * внутри круглого бейджа трендов.
 */
export const ArrowUpRightIcon = (props: IconProps) => (
  <Svg {...props} strokeWidth={2.1}>
    <path d="M6 18 18 6" />
    <path d="M9.5 6H18v8.5" />
  </Svg>
)

export const ArrowDownRightIcon = (props: IconProps) => (
  <Svg {...props} strokeWidth={2.1}>
    <path d="M6 6 18 18" />
    <path d="M9.5 18H18V9.5" />
  </Svg>
)

export const ChevronDownIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="m6 9.5 6 6 6-6" />
  </Svg>
)

export const CheckIcon = (props: IconProps) => (
  <Svg {...props}>
    <path d="m5 12.5 4.5 4.5L19 7" />
  </Svg>
)
