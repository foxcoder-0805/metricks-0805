import { useEffect, useState } from 'react'
import type { HallNote, MusicStatus } from '../types'

/** Клавиатура настоящего пианино: 88 нот от A0 до C8. */
const FIRST_MIDI = 21
const LAST_MIDI = 108
/** Полутон белой клавиши: между B и C, а также между E и F чёрных нет. */
const WHITE_PITCH_CLASSES = new Set([0, 2, 4, 5, 7, 9, 11])
/** Ширина белой клавиши в процентах: их 52 на всей клавиатуре. */
const WHITE_WIDTH = 100 / 52

interface Key {
  midi: number
  left: number
  width: number
}

/** Раскладка клавиши: белые идут своей очередью, чёрные висят на границах. */
function keyboardLayout() {
  const white: Key[] = []
  const black: Key[] = []
  let whites = 0

  for (let midi = FIRST_MIDI; midi <= LAST_MIDI; midi += 1) {
    if (WHITE_PITCH_CLASSES.has(midi % 12)) {
      white.push({ midi, left: whites * WHITE_WIDTH, width: WHITE_WIDTH })
      whites += 1
    } else {
      // Чёрная клавиша сидит посередине между двумя белыми: счётчик белых
      // и есть номер их общей границы.
      black.push({
        midi,
        left: whites * WHITE_WIDTH - WHITE_WIDTH * 0.3,
        width: WHITE_WIDTH * 0.6,
      })
    }
  }

  return { white, black }
}

const LAYOUT = keyboardLayout()

const PITCH_CLASS: Record<string, number> = {
  C: 0,
  'C#': 1,
  D: 2,
  'D#': 3,
  E: 4,
  F: 5,
  'F#': 6,
  G: 7,
  'G#': 8,
  A: 9,
  'A#': 10,
  B: 11,
}

/** Подпись ноты (`A4`) в MIDI-номер: 69 = A4 = 440 Гц. Прочерк — это не нота. */
function midiOfLabel(label: string): number | null {
  const match = /^([A-G]#?)(\d+)$/.exec(label)
  if (!match) return null

  return (Number(match[2]) + 1) * 12 + PITCH_CLASS[match[1]]
}

interface PianoProps {
  /** Таймлайн эталонного трека: когда и какая нота должна звучать. */
  notes: HallNote[]
  music: MusicStatus
  /** Момент сбора снимка (unix, мс): от него идёт позиция между опросами. */
  collectedAt: number
  /** Нота, которую живой детектор слышит сейчас (`A4` или прочерк). */
  heard: string
}

/**
 * Фортепиано вместо полосы спектра: на время эталонного трека подвал
 * показывает клавиатуру — горят ноты из таймлайна в текущей позиции плеера,
 * а то, что реально распознал живой детектор, обводится белым. По двум
 * подсветкам видно, совпало звучание с экраном или нет.
 *
 * Позиция считается локально: снимок даёт опору раз в секунду, между
 * опросами время идёт по часам страницы — тогда клавиши горят плавно,
 * а не рывками раз в секунду.
 */
export function Piano({ notes, music, collectedAt, heard }: PianoProps) {
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    let frame = 0

    const tick = () => {
      setNow(Date.now())
      frame = requestAnimationFrame(tick)
    }

    frame = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(frame)
  }, [])

  const positionMs =
    music.positionMs === null
      ? null
      : // На паузе время не идёт: позиция стоит на том же кадре.
        music.playing
        ? music.positionMs + Math.max(0, now - collectedAt)
        : music.positionMs

  const active = new Set<number>()
  if (positionMs !== null) {
    for (const note of notes) {
      const start = note.t * 1000
      if (start <= positionMs && positionMs < start + note.d * 1000) {
        active.add(note.m)
      }
    }
  }

  const heardMidi = midiOfLabel(heard)
  const keyStyle = (key: Key) => ({ left: `${key.left}%`, width: `${key.width}%` })
  const keyClasses = (key: Key, black: boolean) =>
    [
      black ? 'piano__key piano__key--black' : 'piano__key',
      active.has(key.midi) ? 'piano__key--on' : '',
      heardMidi === key.midi ? 'piano__key--heard' : '',
    ]
      .filter(Boolean)
      .join(' ')

  return (
    <div className="piano" aria-hidden="true">
      {LAYOUT.white.map((key) => (
        <span key={key.midi} className={keyClasses(key, false)} style={keyStyle(key)} />
      ))}
      {/* Чёрные поверх белых: абсолютное позиционирование и z-index держат порядок. */}
      {LAYOUT.black.map((key) => (
        <span key={key.midi} className={keyClasses(key, true)} style={keyStyle(key)} />
      ))}
    </div>
  )
}
