import type { AudioBars } from '../types'

interface AudioVisualizerProps {
  frame: AudioBars
  /**
   * Ноты, которые должны звучать по эталонному таймлайну: пусто для чужого
   * звука. Список обновляется раз в секунду вместе со статусом.
   */
  expected: string[]
}

/**
 * Полоса спектра во всю ширину окна, как в cava: столбики растут от пола,
 * а над ними идёт лента распознанных нот. Текущая нота подсвечена, прошлые
 * приглушены — по ленте читается мелодия, а не только громкость.
 * Слева, призрачным фиолетовым, идут ожидаемые ноты эталонного трека:
 * рядом с распознанными видно, попал ли звук в ноты с экрана.
 */
export function AudioVisualizer({ frame, expected }: AudioVisualizerProps) {
  const visible = frame.active && frame.bars.length > 0
  const notes = frame.notes

  return (
    <div
      className={`visualizer${visible ? ' visualizer--on' : ''}`}
      aria-hidden="true"
    >
      {notes.length > 0 || expected.length > 0 ? (
        <div className="visualizer__notes">
          {expected.length > 0 ? (
            <span className="visualizer__expected">
              {expected.map((note, index) => (
                // Ожидаемые ноты пересчитываются раз в секунду, индекс — ключ.
                <span key={`${note}-${index}`} className="visualizer__expected-note">
                  {note}
                </span>
              ))}
            </span>
          ) : null}

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
            // Полоска текущей ноты горит ярче: видно, куда именно попал тон.
            className={`visualizer__bar${
              visible && index === frame.noteBar ? ' visualizer__bar--note' : ''
            }`}
            style={{ transform: `scaleY(${Math.max(0.02, level).toFixed(3)})` }}
          />
        ))}
      </div>
    </div>
  )
}
