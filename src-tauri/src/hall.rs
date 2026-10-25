//! Нотный эталон «In the Hall of the Mountain King».
//!
//! Видео с фортепианной визуализацией (Kassia) рисует падающие блоки нот:
//! блок, пересекающий строку детекции, — это нота, которая вот-вот прозвучит.
//! Разбор кадров через `ffmpeg` даёт честный таймлайн нот с экрана, который
//! хранится в репозитории как производные данные (`data/`), а само видео
//! в коммит не попадает.
//!
//! Геометрия клавиатуры в кадре откалибрована по щелям между белыми клавишами
//! (шаг 37.86 px, левый край кадра попадает внутрь клавиши A0) и сверена с
//! паттерном чёрных клавиш: карта «x → MIDI» воспроизводит все 88 клавиш A0…C8.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

/// Ширина кадра и полоса разбора: от `BAND_TOP` до линии касания.
const FRAME_WIDTH: usize = 1920;
const BAND_TOP: usize = 440;
const BAND_HEIGHT: usize = 101;
/// Строка детекции внутри полосы (кадровые координаты): y = 520 в кадре.
///
/// На 20 px выше линии касания: вспышка удара сюда не дотягивается, а блок
/// ноты гарантированно проходит строку.
const DETECT_ROW: usize = 80;
/// Частота кадров видео — 30000/1001 fps.
const FPS: f64 = 30_000.0 / 1001.0;
/// Скорость падения блоков: измерена кросс-корреляцией строк y=440 и y=520.
const FALL_SPEED_PX_S: f64 = 600.0;
/// Линия касания на клавиатуре: момент удара = момент прохода блоком строки
/// плюс расстояние до линии, делённое на скорость падения.
const STRIKE_LINE_Y: f64 = 540.0;
/// Яркость выше порога — пиксель принадлежит блоку ноты.
const THRESHOLD: u8 = 90;
/// Сегмент уже самой узкой клавиши — это искра, а не нота.
const MIN_SEGMENT: usize = 14;
/// Сегмент шире трёх клавиш — подчёркивание титульной карточки.
const MAX_SEGMENT: usize = 120;
/// Ширина белой клавиши и сдвиг сетки: клавиша `i` занимает
/// `[KEY_WIDTH·i − KEY_X_OFFSET, KEY_WIDTH·(i+1) − KEY_X_OFFSET]`.
const KEY_WIDTH_PX: f64 = 37.86;
const KEY_X_OFFSET: f64 = 27.7;
/// Нота живёт в кадрах: короче — случайная искра, дольше — настоящий удар.
const MIN_FRAMES: usize = 2;
/// Кадров без блока, которые ещё не закрывают ноту: гасит мерцание кромки.
const GAP_FRAMES: usize = 2;
/// Длительность эталонного трека «In the Hall of the Mountain King».
const TRACK_SECONDS: f64 = 142.71;

/// Нота эталонного таймлайна: начало, длительность и MIDI-номер.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NoteEvent {
    /// Момент касания в секундах от начала видео.
    pub t: f64,
    /// Длительность блока в секундах.
    pub d: f64,
    /// Высота ноты.
    pub m: i32,
}

/// Таймлайн эталонного трека, вшитый в бинарь из `data/`.
const DATA: &str = include_str!("../../data/hall_of_the_mountain_king.json");

/// Эталонные ноты видео.
pub fn hall_notes() -> &'static [NoteEvent] {
    static NOTES: std::sync::OnceLock<Vec<NoteEvent>> = std::sync::OnceLock::new();

    NOTES
        .get_or_init(|| serde_json::from_str(DATA).expect("эталонный таймлайн разбирается"))
        .as_slice()
}

/// Ноты эталонного трека, звучащие в указанный момент.
pub fn notes_active_at(position: f64) -> Vec<i32> {
    let mut active: Vec<i32> = hall_notes()
        .iter()
        .filter(|note| note.t <= position && position < note.t + note.d)
        .map(|note| note.m)
        .collect();

    active.sort_unstable();
    active.dedup();
    active
}

/// Те же ноты, но подписанные (`E3`, `A#4`) — так их рисует фронтенд.
pub fn expected_note_labels(position: f64) -> Vec<String> {
    notes_active_at(position)
        .into_iter()
        .map(crate::audio::note_label)
        .collect()
}

/// Играет ли плеер эталонный трек: файл `great_music.mp3` либо название
/// «In the Hall of the Mountain King», да ещё и та же длительность —
/// другое исполнение той же пьесы длится иначе и ложного сравнения не будет.
pub fn is_reference_track(url: &str, title: &str, duration_secs: Option<f64>) -> bool {
    /// Допуск на разницу в округлении и кодеках.
    const TOLERANCE_S: f64 = 0.5;

    let named = url.ends_with("great_music.mp3")
        || title.trim().eq_ignore_ascii_case("in the hall of the mountain king");
    let length = duration_secs.is_some_and(|secs| (secs - TRACK_SECONDS).abs() <= TOLERANCE_S);

    named && length
}

/// Tauri-команда: весь таймлайн эталонного трека. Фронтенд забирает его один
/// раз, когда трек опознан, и дальше сам считает активные ноты по
/// интерполированной позиции — гонять 3287 нот через IPC на каждый кадр
/// незачем, плеер опрашивается раз в секунду.
#[tauri::command]
pub fn get_reference_timeline() -> Vec<NoteEvent> {
    hall_notes().to_vec()
}

/// Белая клавиша `index` (0 = A0) и её MIDI-номер.
fn white_midi(index: i32) -> Option<i32> {
    const STEP: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];

    (0..=51)
        .contains(&index)
        .then(|| 21 + STEP[index.rem_euclid(7) as usize] + 12 * (index / 7))
}

/// Центры всех 88 клавиш в пикселях кадра, отсортированные по x.
///
/// Чёрная клавиша сидит на границе двух белых и появляется только там, где
/// разница между белыми — целый тон (нет чёрной между B и C, E и F).
fn key_grid() -> Vec<(f64, i32)> {
    let mut keys = Vec::with_capacity(88);

    for index in 0..52 {
        let midi = white_midi(index).expect("белая клавиша в диапазоне A0…C8");
        keys.push((KEY_WIDTH_PX * (index as f64 + 0.5) - KEY_X_OFFSET, midi));

        if let Some(next) = white_midi(index + 1) {
            if next - midi == 2 {
                keys.push((KEY_WIDTH_PX * (index as f64 + 1.0) - KEY_X_OFFSET, midi + 1));
            }
        }
    }

    keys.sort_by(|left, right| left.0.total_cmp(&right.0));
    keys
}

/// MIDI-ноты, чьи клавиши перекрыты сегментом `[x0, x1]`.
///
/// Блоки аккорда слипаются в один сегмент, поэтому ноты ищутся по центрам
/// клавиш внутри сегмента, а не по одному центру сегмента.
fn keys_in(grid: &[(f64, i32)], x0: usize, x1: usize) -> Vec<i32> {
    let (left, right) = (x0 as f64 - 8.0, x1 as f64 + 8.0);

    grid.iter()
        .filter(|(center, _)| (left..=right).contains(center))
        .map(|(_, midi)| *midi)
        .collect()
}

/// Светлые сегменты строки кадра: координаты включительно, по ширине кадра.
fn segments(row: &[u8]) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut open: Option<usize> = None;

    for (x, &value) in row.iter().enumerate() {
        if value > THRESHOLD {
            open.get_or_insert(x);
        } else if let Some(start) = open.take() {
            if ((x - start) >= MIN_SEGMENT) && ((x - start) <= MAX_SEGMENT) {
                found.push((start, x - 1));
            }
        }
    }

    if let Some(start) = open {
        let end = row.len() - 1;
        if (end + 1 - start) >= MIN_SEGMENT && (end + 1 - start) <= MAX_SEGMENT {
            found.push((start, end));
        }
    }

    found
}

/// Незакрытая нота: кадр касания и последний кадр с блоком на строке.
struct Active {
    start: usize,
    last: usize,
}

/// Разбирает видео и возвращает нотный таймлайн, отсортированный по времени.
fn extract(video: &Path) -> std::io::Result<Vec<NoteEvent>> {
    use std::io::Read as _;

    let band = format!("crop={FRAME_WIDTH}:{BAND_HEIGHT}:0:{BAND_TOP}");
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(video)
        .args(["-vf", &band, "-f", "rawvideo", "-pix_fmt", "gray", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut stdout = child.stdout.take().expect("stdout ffmpeg");
    let mut frame = vec![0u8; FRAME_WIDTH * BAND_HEIGHT];
    let grid = key_grid();
    let lead = (STRIKE_LINE_Y - (BAND_TOP + DETECT_ROW) as f64) / FALL_SPEED_PX_S;

    let mut notes = Vec::new();
    let mut active: std::collections::HashMap<i32, Active> = std::collections::HashMap::new();
    let mut index = 0usize;

    while stdout.read_exact(&mut frame).is_ok() {
        let row = &frame[DETECT_ROW * FRAME_WIDTH..(DETECT_ROW + 1) * FRAME_WIDTH];
        let mut sounding = std::collections::HashSet::new();

        for (x0, x1) in segments(row) {
            for midi in keys_in(&grid, x0, x1) {
                sounding.insert(midi);
                active
                    .entry(midi)
                    .and_modify(|note| note.last = index)
                    .or_insert(Active {
                        start: index,
                        last: index,
                    });
            }
        }

        let expired: Vec<i32> = active
            .iter()
            .filter(|(midi, note)| !sounding.contains(*midi) && index - note.last > GAP_FRAMES)
            .map(|(midi, _)| *midi)
            .collect();

        for midi in expired {
            let note = active.remove(&midi).expect("нота ещё открыта");
            let frames = note.last - note.start + 1;
            if frames >= MIN_FRAMES {
                notes.push(NoteEvent {
                    t: note.start as f64 / FPS + lead,
                    d: frames as f64 / FPS,
                    m: midi,
                });
            }
        }

        index += 1;
    }

    let _ = child.kill();
    let _ = child.wait();

    // Ноты, звучащие в последнем кадре, закрываем по последнему кадру.
    for (midi, note) in active {
        let frames = note.last - note.start + 1;
        if frames >= MIN_FRAMES {
            notes.push(NoteEvent {
                t: note.start as f64 / FPS + lead,
                d: frames as f64 / FPS,
                m: midi,
            });
        }
    }

    notes.sort_by(|left, right| left.t.total_cmp(&right.t));

    // Секунды в JSON — с точностью до миллисекунды, чтобы файл был стабильным.
    for note in &mut notes {
        note.t = (note.t * 1000.0).round() / 1000.0;
        note.d = (note.d * 1000.0).round() / 1000.0;
    }

    Ok(notes)
}

/// Корень репозитория: рядом с `src-tauri` лежат данные и видео.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("у src-tauri есть родительский каталог")
        .to_path_buf()
}

/// Где лежит видео: переменная окружения, затем корень репозитория и `~/Videos`.
fn find_video() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("METRICKS_VIDEO") {
        return Some(PathBuf::from(path));
    }

    let home = std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Videos"));
    let directories = [Some(repo_root()), home];

    let mut candidates: Vec<PathBuf> = Vec::new();
    for directory in directories.into_iter().flatten() {
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };

        candidates.extend(
            entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| path.extension().is_some_and(|ext| ext == "mp4")),
        );
    }

    candidates.into_iter().find(|path| path.is_file())
}

/// Файл эталонного таймлайна в репозитории.
fn data_path() -> PathBuf {
    repo_root().join("data/hall_of_the_mountain_king.json")
}

/// Диагностика: перечитывает видео и переписывает эталонный таймлайн.
#[allow(dead_code)]
fn video_note_dump() -> std::io::Result<()> {
    let Some(video) = find_video() else {
        println!("видео не найдено: задай METRICKS_VIDEO=/путь/к/видео.mp4");
        return Ok(());
    };

    println!("разбираю {}", video.display());
    let notes = extract(&video)?;

    let path = data_path();
    std::fs::create_dir_all(path.parent().expect("у файла есть каталог"))?;
    std::fs::write(&path, serde_json::to_string(&notes)?)?;

    println!("{} нот записано в {}", notes.len(), path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{detect_pitch, note_midi, PITCH_WINDOW, SAMPLE_RATE};

    /// Погрешность сверки с YIN: плюс-минус четверть секунды и октава — на
    /// басовых нотах детектор устойчиво ошибается на октаву вниз.
    const MATCH_TOLERANCE_S: f64 = 0.3;
    /// Доля совпадений YIN с эталоном, ниже которой сверка считается провалом.
    const MIN_MATCH_RATE: f64 = 0.6;

    /// Команда отдаёт фронтенду тот же таймлайн, в том же порядке: клавиши
    /// фортепиано считают активные ноты по этому списку.
    #[test]
    fn timeline_command_returns_notes_in_order() {
        let timeline = super::get_reference_timeline();

        assert_eq!(timeline.len(), hall_notes().len());
        assert!(timeline.windows(2).all(|pair| pair[0].t <= pair[1].t));
    }

    #[test]
    fn white_keys_cover_the_whole_piano() {
        assert_eq!(white_midi(0), Some(21));
        assert_eq!(white_midi(1), Some(23));
        assert_eq!(white_midi(2), Some(24));
        assert_eq!(white_midi(51), Some(108));
        assert_eq!(white_midi(52), None);
        assert_eq!(white_midi(-1), None);
    }

    #[test]
    fn black_keys_sit_between_whole_tones() {
        let grid = key_grid();

        assert_eq!(grid.len(), 88, "88 клавиш A0…C8");
        assert_eq!(grid[0].1, 21, "крайняя слева — A0");
        assert_eq!(grid.last().expect("клавиши есть").1, 108, "крайняя справа — C8");

        let midi: std::collections::HashSet<i32> = grid.iter().map(|(_, midi)| *midi).collect();
        assert_eq!(midi.len(), 88, "клавиши не задублированы");
        assert!(midi.contains(&22) && midi.contains(&25), "A#0 и C#1 на месте");
        assert!(!midi.contains(&20) && !midi.contains(&109), "за краями пусто");
    }

    #[test]
    fn segments_reject_sparks_and_titles() {
        // Искра уже самой узкой клавиши.
        assert!(segments(&bright_row(100, 110)).is_empty());
        // Нота: сегмент шириной в белую клавишу.
        assert_eq!(segments(&bright_row(200, 238)), vec![(200, 238)]);
        // Титульная карточка: светлая линия во всю ширину блока.
        assert!(segments(&bright_row(500, 1400)).is_empty());
    }

    #[test]
    fn chord_segment_maps_to_every_covered_key() {
        let grid = key_grid();
        // Слипшиеся блоки аккорда E4…B4 (MIDI 64, 67, 71).
        let e4 = KEY_WIDTH_PX * 25.5 - KEY_X_OFFSET;
        let b4 = KEY_WIDTH_PX * 29.5 - KEY_X_OFFSET;
        let keys = keys_in(&grid, e4 as usize - 4, b4 as usize + 4);

        assert!(keys.contains(&64) && keys.contains(&67) && keys.contains(&71));
    }

    #[test]
    fn ground_truth_track_is_sane() {
        let notes = hall_notes();

        assert!(notes.len() > 2_500, "в таймлайне {} нот", notes.len());
        assert!(
            notes.windows(2).all(|pair| pair[0].t <= pair[1].t),
            "таймлайн отсортирован по времени"
        );
        assert!(notes.iter().all(|note| (21..=108).contains(&note.m)));
        assert!(notes.iter().all(|note| note.d >= 0.06 && note.d <= 5.0));
        assert!(notes.iter().all(|note| (0.0..TRACK_SECONDS).contains(&note.t)));

        // Начало видео — титульные карточки: нот там нет.
        assert!(
            notes.first().expect("таймлайн не пуст").t > 0.5,
            "до 0.5 с нот нет"
        );
        assert!(!notes_active_at(13.0).is_empty(), "в середине трека ноты звучат");
    }

    #[test]
    fn reference_track_is_recognised_by_name_and_length() {
        assert!(is_reference_track(
            "file:///home/foxcoder/Music/great_music.mp3",
            "great_music",
            Some(TRACK_SECONDS)
        ));
        assert!(is_reference_track(
            "https://example/playlist",
            "In the Hall of the Mountain King",
            Some(142.5)
        ));

        // Чужой файл той же пьесы другой длительности — не эталон.
        assert!(!is_reference_track(
            "file:///home/foxcoder/Music/other.mp3",
            "In the Hall of the Mountain King",
            Some(180.0)
        ));
        // Свой файл, но плеер не отдал длительность: сверять не с чем.
        assert!(!is_reference_track(
            "file:///home/foxcoder/Music/great_music.mp3",
            "great_music",
            None
        ));
        // Совпадение только по названию недостаточно.
        assert!(!is_reference_track("", "Bohemian Rhapsody", Some(TRACK_SECONDS)));
    }

    #[test]
    fn expected_notes_follow_the_timeline() {
        // В начале видео — титульные карточки, играть нечего.
        assert!(expected_note_labels(0.2).is_empty());

        // В середине трека эталон звучит, и ноты подписаны по-человечески.
        let labels = expected_note_labels(13.0);
        assert!(!labels.is_empty());
        assert!(
            labels.iter().all(|label| {
                let pitch = label.as_bytes()[0];
                matches!(pitch, b'A'..=b'G') && label.len() <= 3
            }),
            "подписи вида E3/A#4: {labels:?}"
        );
    }

    /// Сверка YIN с эталоном: распознанные ноты трека должны попадать в ноты,
    /// показанные на экране. Требует mp3, запускается вручную:
    /// `METRICKS_TRACK=/путь/к/треку.mp3 cargo test -- --ignored yin_matches_video_notes`
    #[test]
    #[ignore]
    fn yin_matches_video_notes() {
        use std::io::Read as _;

        let Ok(track) = std::env::var("METRICKS_TRACK") else {
            println!("задай METRICKS_TRACK=/путь/к/файлу.mp3");
            return;
        };

        let notes = hall_notes();
        assert!(!notes.is_empty(), "эталонный таймлайн пуст");

        let mut child = Command::new("ffmpeg")
            .args([
                "-v", "error", "-i", &track, "-f", "f32le", "-ac", "1", "-ar", "44100", "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("ffmpeg");

        let mut stdout = child.stdout.take().expect("stdout");
        let mut raw = vec![0u8; crate::audio::CHUNK * 4];
        let mut samples: Vec<f32> = Vec::with_capacity(PITCH_WINDOW * 2);
        let mut difference = vec![0.0; PITCH_WINDOW / 2];
        let mut read = 0usize;
        let mut matched = 0usize;
        let mut pitched = 0usize;

        while stdout.read_exact(&mut raw).is_ok() {
            read += crate::audio::CHUNK;
            samples.extend(
                raw.chunks_exact(4)
                    .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
            );

            if samples.len() > PITCH_WINDOW {
                let excess = samples.len() - PITCH_WINDOW;
                samples.drain(..excess);
            }
            if samples.len() < PITCH_WINDOW {
                continue;
            }

            // Момент окна — его середина: детектор смотрит на последние 2048 сэмплов.
            let position =
                read as f64 / SAMPLE_RATE as f64 - PITCH_WINDOW as f64 / SAMPLE_RATE as f64 / 2.0;

            let Some(midi) = detect_pitch(&samples, &mut difference).and_then(note_midi) else {
                continue;
            };

            pitched += 1;
            let hit = notes.iter().any(|note| {
                (note.t - MATCH_TOLERANCE_S..=note.t + note.d + MATCH_TOLERANCE_S)
                    .contains(&position)
                    && (note.m == midi || note.m == midi + 12 || note.m == midi - 12)
            });

            if hit {
                matched += 1;
            }
        }

        let _ = child.kill();

        let rate = matched as f64 / pitched.max(1) as f64;
        println!(
            "совпало {} из {} распознанных нот ({:.0}%)",
            matched,
            pitched,
            rate * 100.0
        );

        assert!(
            rate >= MIN_MATCH_RATE,
            "YIN совпал с эталоном лишь в {:.0}% случаев",
            rate * 100.0
        );
    }

    /// Подбор настроек YIN по эталонному треку: печатает, сколько кадров
    /// вообще даёт ноту и сколько из них попадает в ноты видео. Алгоритм
    /// тяжёлый, поэтому запускается в релизе:
    /// `METRICKS_TRACK=/путь/к/треку.mp3 cargo test --release --lib -- --ignored pitch_tuning_sweep`
    #[test]
    #[ignore]
    fn pitch_tuning_sweep() {
        use crate::audio::{detect_pitch_with, note_midi_with};
        use std::io::Read as _;

        let Ok(track) = std::env::var("METRICKS_TRACK") else {
            println!("задай METRICKS_TRACK=/путь/к/треку.mp3");
            return;
        };

        let mut child = Command::new("ffmpeg")
            .args([
                "-v", "error", "-i", &track, "-f", "f32le", "-ac", "1", "-ar", "44100", "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("ffmpeg");

        let mut stdout = child.stdout.take().expect("stdout");
        let mut raw = Vec::new();
        stdout.read_to_end(&mut raw).expect("чтение трека");
        let _ = child.kill();
        let samples: Vec<f32> = raw
            .chunks_exact(4)
            .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .collect();
        drop(raw);

        let notes = hall_notes();
        // (окно, порог провала, уверенность, допуск в центах): первая строка —
        // то, что стоит в продакшене, остальные — соседи для сравнения.
        let tunings: &[(usize, f32, f32, f32)] = &[
            (2_048, 0.40, 0.55, 45.0), // текущие настройки
            (2_048, 0.15, 0.55, 30.0), // как было до подбора
            (4_096, 0.40, 0.55, 45.0),
            (4_096, 0.40, 0.55, 60.0),
            (4_096, 0.40, 0.55, 30.0),
            (8_192, 0.40, 0.55, 45.0),
            (4_096, 0.30, 0.55, 45.0),
        ];

        println!(
            "{:>6} {:>7} {:>7} {:>6} {:>7} {:>7}",
            "окно", "порог", "увер.", "центы", "нотили", "совпало"
        );

        for &(window, threshold, clarity, cents) in tunings {
            let mut difference = vec![0.0f32; window / 2];
            let mut pitched = 0usize;
            let mut matched = 0usize;
            let mut frames = 0usize;

            for end in (window..=samples.len()).step_by(crate::audio::CHUNK) {
                frames += 1;
                let slice = &samples[end - window..end];
                let position =
                    end as f64 / SAMPLE_RATE as f64 - window as f64 / SAMPLE_RATE as f64 / 2.0;

                let Some(midi) =
                    detect_pitch_with(slice, &mut difference, threshold, clarity)
                        .and_then(|pitch| note_midi_with(pitch, clarity, cents))
                else {
                    continue;
                };

                pitched += 1;
                if notes.iter().any(|note| {
                    (note.t - MATCH_TOLERANCE_S..=note.t + note.d + MATCH_TOLERANCE_S)
                        .contains(&position)
                        && (note.m == midi || note.m == midi + 12 || note.m == midi - 12)
                }) {
                    matched += 1;
                }
            }

            println!(
                "{window:>6} {threshold:>7.2} {clarity:>7.2} {cents:>6.0} {:>6} ({:>4.0}%) {:>5} ({:>3.0}%)",
                pitched,
                pitched as f64 / frames.max(1) as f64 * 100.0,
                matched,
                matched as f64 / pitched.max(1) as f64 * 100.0,
            );
        }
    }

    /// Переписывает эталонный таймлайн из видео.
    /// `cargo test -- --ignored --nocapture video_note_dump`
    #[test]
    #[ignore]
    fn video_note_dump() {
        super::video_note_dump().expect("разбор видео");
    }

    fn bright_row(from: usize, to: usize) -> Vec<u8> {
        let mut row = vec![0u8; FRAME_WIDTH];
        for value in &mut row[from..=to] {
            *value = 200;
        }
        row
    }
}
