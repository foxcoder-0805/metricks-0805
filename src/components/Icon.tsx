import { ICONS, type IconName } from '../lib/icons'
import type { IconProps } from './Icons'

/** Иконка по имени из API. */
export function Icon({ name, ...props }: IconProps & { name: IconName }) {
  const Component = ICONS[name]
  return <Component {...props} />
}
