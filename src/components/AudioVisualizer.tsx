import { useReferenceTimeline } from '../hooks/useReferenceTimeline'
import type { AudioBars, MusicStatus } from '../types'
import { Piano } from './Piano'

interface AudioVisualizerProps {
  frame: AudioBars
  /**
   * Статус плеера: по флагу эталонного трека полоса спектра уступает место
   * фортепиано, а призрачная строка ожидаемых нот остаётся только на время,
   * пока таймлайн ещё не подтянулся.
   */
  music: MusicStatus | null
  /** Момент сбора снимка (unix, мс): от него считается позиция плеера. */
  collectedAt: number | null
}

/**
 * Нижняя полоса окна. Обычно это cava-полоса спектра во всю ширину с лентой
 * распознанных нот над ней; пока играет эталонный трек, полоса заменяется
 * фортепиано, на котором по таймлайну горят ноты экрана, а распознанная
 * живым детектором нота обводится отдельно.
 */
export function AudioVisualizer({ frame, music, collectedAt }: AudioVisualizerProps) {
  const expected = music?.expectedNotes ?? []
  const reference = Boolean(music?.reference && music.positionMs !== null && collectedAt !== null)
  const timeline = useReferenceTimeline(reference)
  const notes = frame.notes

  const showPiano =
    music !== null &&
    collectedAt !== null &&
    timeline !== null &&
    music.reference &&
    music.positionMs !== null

  // Фортепиано живёт по часам плеера и виден, пока трек играет; полосе
  // спектра, как и раньше, нужен слышимый звук.
  const visible = showPiano
    ? Boolean(music?.playing) || frame.active
    : frame.active && frame.bars.length > 0

  return (
    <div
      className={`visualizer${visible ? ' visualizer--on' : ''}`}
      aria-hidden="true"
    >
      {notes.length > 0 || (!showPiano && expected.length > 0) ? (
        <div className="visualizer__notes">
          {!showPiano && expected.length > 0 ? (
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

      {showPiano && music && timeline && collectedAt !== null ? (
        <Piano
          notes={timeline}
          music={music}
          collectedAt={collectedAt}
          heard={frame.note}
        />
      ) : (
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
      )}
    </div>
  )
}
