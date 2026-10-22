import type { AudioBars } from '../types'

interface AudioVisualizerProps {
  frame: AudioBars
}

/**
 * Полоса спектра во всю ширину окна, как в cava: столбики растут от пола,
 * а над ними идёт лента распознанных нот. Текущая нота подсвечена, прошлые
 * приглушены — по ленте читается мелодия, а не только громкость.
 */
export function AudioVisualizer({ frame }: AudioVisualizerProps) {
  const visible = frame.active && frame.bars.length > 0
  const notes = frame.notes

  return (
    <div
      className={`visualizer${visible ? ' visualizer--on' : ''}`}
      aria-hidden="true"
    >
      {notes.length > 0 ? (
        <div className="visualizer__notes">
          {notes.map((note, index) => (
            <span
              // Ноты повторяются редко и идут подряд, поэтому пара «нота + место»
              // даёт устойчивый ключ даже при повторе.
              key={`${note}-${index}`}
              className={`visualizer__note${
                index === notes.length - 1 ? ' visualizer__note--now' : ''
              }`}
            >
              {note}
            </span>
          ))}
        </div>
      ) : null}

      <div className="visualizer__bars">
        {frame.bars.map((level, index) => (
          <span
            // Полоски обновляются на месте, поэтому индекс — честный ключ.
            key={index}
            className="visualizer__bar"
            style={{ transform: `scaleY(${Math.max(0.02, level).toFixed(3)})` }}
          />
        ))}
      </div>
    </div>
  )
}
