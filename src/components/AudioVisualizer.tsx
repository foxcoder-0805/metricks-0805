import type { AudioBars } from '../types'

interface AudioVisualizerProps {
  frame: AudioBars
}

/**
 * Полоса спектра во всю ширину окна, как в cava: столбики растут от пола,
 * при тишине полоса плавно уезжает вниз и исчезает.
 */
export function AudioVisualizer({ frame }: AudioVisualizerProps) {
  const visible = frame.active && frame.bars.length > 0

  return (
    <div
      className={`visualizer${visible ? ' visualizer--on' : ''}`}
      aria-hidden="true"
    >
      {frame.bars.map((level, index) => (
        <span
          // Полоски обновляются на месте, поэтому индекс — честный ключ.
          key={index}
          className="visualizer__bar"
          style={{ transform: `scaleY(${Math.max(0.02, level).toFixed(3)})` }}
        />
      ))}
    </div>
  )
}
