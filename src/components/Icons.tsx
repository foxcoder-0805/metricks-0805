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

/** Крестик — закрывает карточку. */
export const CloseIcon = (props: IconProps) => (
  <Svg {...props} strokeWidth={2.2}>
    <path d="M6.5 6.5 17.5 17.5" />
    <path d="M17.5 6.5 6.5 17.5" />
  </Svg>
)

/** Плюс — добавить карточку. */
export const PlusIcon = (props: IconProps) => (
  <Svg {...props} strokeWidth={2}>
    <path d="M12 5v14" />
    <path d="M5 12h14" />
  </Svg>
)
