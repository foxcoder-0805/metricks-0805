//! Спектр системного звука для полосы внизу окна.
//!
//! Свой мини-«cava»: `parec` отдаёт монитор вывода PipeWire/PulseAudio,
//! поток разбирается на окна, к ним применяется окно Ханна и БПФ, и
//! фронтенд получает уже готовые уровни полосок.
//!
//! Поверх спектра работает детектор высоты тона: он ищет основной тон
//! методом YIN и переводит его в ноту, чтобы над полосой шла лента нот
//! («In the Hall of the Mountain King» виден как `B3 D4 E4 …`).
//!
//! Захват живёт в отдельном потоке, чтобы не тормозить опрос состояния
//! машины: `parec` пишет в pipe, поток читает его и складывает бары в
//! общий буфер последнего значения.

use std::f32::consts::PI;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, ChildStderr, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use serde::Serialize;

/// Частота дискретизации, размер окна БПФ и шаг чтения.
///
/// Окно широкое (186 мс): полоски отвечают за полутон, а полутон у C4 —
/// всего 15 Гц, поэтому тонкий бин 5.4 Гц важнее короткой задержки.
pub(crate) const SAMPLE_RATE: usize = 44_100;
const WINDOW: usize = 8_192;
/// Сколько сэмплов читаем за раз: между кадрами спектра проходит 23 мс.
pub(crate) const CHUNK: usize = 1_024;
/// Гистерезис: после последнего громкого кадра визуализатор держится
/// ещё немного, чтобы полоса не мигала на паузах между треками.
const HOLD: Duration = Duration::from_secs(2);
/// Ниже этого RMS считаем, что звука нет — как `silence` в cava.
const SILENCE: f32 = 0.004;

/// Сколько полосок рисует фронтенд по умолчанию: по одной на полутон.
const DEFAULT_BARS: usize = 73;
/// Нотная сетка полосок: полутон от C1 (MIDI 24) до C7 (MIDI 96).
const LOW_MIDI: i32 = 24;
const HIGH_MIDI: i32 = 96;
const MIN_DB: f64 = -72.0;
const MAX_DB: f64 = -6.0;

/// Границы голоса и инструментов, которые есть смысл распознавать как ноты.
const PITCH_MIN_HZ: f32 = 80.0;
const PITCH_MAX_HZ: f32 = 1200.0;
/// Ниже этой «уверенности» YIN считаем, что тона нет: аккорды и шум не пишем.
const PITCH_CLARITY: f32 = 0.55;
/// Порог разностной функции YIN: провал ниже него считается периодом тона.
const PITCH_THRESHOLD: f32 = 0.15;
/// Насколько можно промахнуться от чистой ноты, прежде чем рисовать прочерк.
const PITCH_CENTS: f32 = 30.0;
/// Сколько кадров подряд должна подтвердиться новая нота, чтобы её сменить:
/// иначе лента дребезжала бы на вибрато и переходах.
const PITCH_CONFIRM: u32 = 2;
/// Сколько нот хранит лента.
const NOTE_TRAIL: usize = 14;

/// Ноты в 12-равномерном строе: индексы от «до» первой октавы.
const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AudioBars {
    /// Есть звук прямо сейчас (или был мгновение назад).
    pub active: bool,
    /// Уровни полосок от 0 до 1.
    pub bars: Vec<f32>,
    /// Текущая нота, `—` если тон не распознан.
    pub note: String,
    /// Индекс полоски этой ноты: её и подсвечиваем. `None` — тона нет.
    /// Считается здесь, а не на фронтенде: сетка полосок живёт в Rust.
    pub note_bar: Option<usize>,
    /// Лента последних нот: свежая в конце.
    pub notes: Vec<String>,
}

/// Захват звука: поток `parec` и последний посчитанный кадр.
static CAPTURE: OnceLock<Mutex<Capture>> = OnceLock::new();
/// Сколько полосок нужно фронтенду. Одно значение на процесс: окно одно.
static BAR_COUNT: AtomicUsize = AtomicUsize::new(DEFAULT_BARS);

/// Tauri-команда: последний кадр спектра.
#[tauri::command]
pub fn get_audio_bars() -> AudioBars {
    let capture = CAPTURE.get_or_init(|| Mutex::new(Capture::default()));
    let mut guard = capture.lock().unwrap_or_else(|poison| poison.into_inner());

    guard.ensure_running();

    match guard.latest.clone() {
        Some(frame) if frame.fits(BAR_COUNT.load(Ordering::Relaxed)) => frame,
        // Первый кадр приходит через несколько десятков миллисекунд: до него
        // отдаём пустую полосу, а не заставляем фронтенд ждать.
        Some(frame) => {
            let count = BAR_COUNT.load(Ordering::Relaxed);
            AudioBars {
                active: frame.active,
                bars: resample(&frame.bars, count),
                note: frame.note.clone(),
                note_bar: midi_of_note(&frame.note).and_then(|midi| bar_of_midi(midi, count)),
                notes: frame.notes,
            }
        }
        None => AudioBars::default(),
    }
}

impl AudioBars {
    fn fits(&self, count: usize) -> bool {
        self.bars.len() == count
    }

    /// Кадр тишины: полоса пустая, лента нот обнуляется.
    fn silent(count: usize) -> Self {
        Self {
            bars: vec![0.0; count],
            ..Self::default()
        }
    }
}

#[derive(Default)]
struct Capture {
    child: Option<Child>,
    latest: Option<AudioBars>,
}

impl Capture {
    /// Поднимает `parec`, если он ещё не запущен. Поток чтения перезапустит
    /// захват сам, если звуковая подсистема отвалится.
    fn ensure_running(&mut self) {
        let running = self
            .child
            .as_mut()
            .is_some_and(|child| child.try_wait().ok().flatten().is_none());

        if running {
            return;
        }

        let Ok(mut child) = spawn_parec() else {
            return;
        };

        let Some(stdout) = child.stdout.take() else {
            return;
        };

        let stop = Arc::new(AtomicBool::new(false));
        match self.child.take() {
            Some(mut previous) => {
                let _ = previous.kill();
                let _ = previous.wait();
                // Свой флаг: старый читающий поток завершится сам.
                reader_thread(stdout, Arc::new(AtomicBool::new(false)));
            }
            None => reader_thread(stdout, Arc::clone(&stop)),
        }

        self.child = Some(child);
    }
}

fn spawn_parec() -> std::io::Result<Child> {
    Command::new("parec")
        .args([
            "--format=float32le",
            &format!("--rate={SAMPLE_RATE}"),
            "--channels=1",
            "--latency-msec=20",
            "-d",
            "@DEFAULT_MONITOR@",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
}

fn reader_thread(stdout: std::process::ChildStdout, stop: Arc<AtomicBool>) {
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut raw = vec![0u8; 4 * CHUNK];
        let mut samples: Vec<f32> = Vec::with_capacity(WINDOW * 2);
        let mut spectrum = Spectrum::new(WINDOW);
        let mut pitch = PitchDetector::new();
        let mut last_loud: Option<std::time::Instant> = None;

        loop {
            if stop.load(Ordering::Relaxed) {
                return;
            }

            if reader.read_exact(&mut raw).is_err() {
                // Поток закончился: последний кадр гасим, чтобы полоса исчезла.
                let mut guard = lock();
                guard.latest = None;
                return;
            }

            samples.extend(
                raw.chunks_exact(4)
                    .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
            );

            // Шаг в CHUNK, а не в половину окна: широкое окно спектра не должно
            // замедлять обновление полосы.
            if samples.len() < WINDOW {
                continue;
            }

            let rms = (samples[..WINDOW].iter().map(|value| value * value).sum::<f32>()
                / WINDOW as f32)
                .sqrt();

            if rms > SILENCE {
                last_loud = Some(std::time::Instant::now());
            }

            let active = last_loud.is_some_and(|at| at.elapsed() < HOLD);
            let count = BAR_COUNT.load(Ordering::Relaxed);
            let frame = if active {
                pitch.listen(&samples[..WINDOW]);
                AudioBars {
                    active: true,
                    bars: spectrum.bars(&samples[..WINDOW]),
                    note: pitch.note(),
                    note_bar: pitch.note_midi().and_then(|midi| bar_of_midi(midi, count)),
                    notes: pitch.trail(),
                }
            } else {
                pitch.reset();
                AudioBars::silent(count)
            };

            lock().latest = Some(frame);
            samples.drain(..CHUNK);
        }
    });
}

fn lock() -> std::sync::MutexGuard<'static, Capture> {
    let capture = CAPTURE.get_or_init(|| Mutex::new(Capture::default()));
    capture.lock().unwrap_or_else(|poison| poison.into_inner())
}
/// Разово собирает список устройств вывода — нужен только для диагностики.
pub fn capture_diagnostics() -> String {
    let Ok(output) = Command::new("pactl")
        .args(["list", "short", "sinks"])
        .output()
    else {
        return String::from("pactl недоступен");
    };

    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Диагностика: печатает полоски и ноты реального системного звука.
/// `cargo test -- --ignored --nocapture live_audio_dump`
#[cfg(test)]
fn live_audio_dump() {
    for _ in 0..25 {
        let frame = get_audio_bars();
        println!(
            "active={} bars={:?} note={} notes={:?}",
            frame.active,
            frame
                .bars
                .iter()
                .map(|level| (level * 9.0).round() as u8)
                .collect::<Vec<_>>(),
            frame.note,
            frame.notes,
        );

        thread::sleep(Duration::from_millis(120));
    }
}

/// Читает stderr `parec` — он объясняет, почему нет звука.
#[allow(dead_code)]
fn drain_stderr(stderr: ChildStderr) {
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            eprintln!("parec: {line}");
        }
    });
}

// ───────────────────────────── БПФ ─────────────────────────────

/// Итеративное БПФ с предпосчитанными поворотными множителями и окном Ханна.
struct Spectrum {
    size: usize,
    window: Vec<f32>,
    reversed: Vec<usize>,
    cos: Vec<f32>,
    sin: Vec<f32>,
    real: Vec<f32>,
    imag: Vec<f32>,
}

impl Spectrum {
    fn new(size: usize) -> Self {
        let window = (0..size)
            .map(|index| 0.5 - 0.5 * ((2.0 * PI * index as f32) / size as f32).cos())
            .collect();

        // Бит-реверс порядка индексов — классическая схема Кули — Тьюки.
        let bits = size.trailing_zeros();
        let reversed = (0..size)
            .map(|index| (0..bits).fold(0, |acc, bit| acc | ((index >> bit) & 1) << (bits - 1 - bit)))
            .collect();

        let cos = (0..size / 2)
            .map(|index| (2.0 * PI * index as f32 / size as f32).cos())
            .collect();
        let sin = (0..size / 2)
            .map(|index| (2.0 * PI * index as f32 / size as f32).sin())
            .collect();

        Self {
            size,
            window,
            reversed,
            cos,
            sin,
            real: vec![0.0; size],
            imag: vec![0.0; size],
        }
    }

    /// Полоски спектра на нотной сетке C1…C7: по одному полутону на полоску.
    fn bars(&mut self, samples: &[f32]) -> Vec<f32> {
        let count = BAR_COUNT.load(Ordering::Relaxed);

        for index in 0..self.size {
            self.real[index] = samples[self.reversed[index]] * self.window[self.reversed[index]];
            self.imag[index] = 0.0;
        }

        self.transform();

        let bins = self.size / 2;
        let mut bars = Vec::with_capacity(count);
        let mut peak = 0.0f32;

        for bar in 0..count {
            let from = bar_edge(bar, count, bins);
            let to = bar_edge(bar + 1, count, bins).max(from + 1);

            // Соседние полоски перекрываются: так полоса выглядит плотнее.
            let slice = &self.real[from..to.min(bins)];
            let magnitude = slice.iter().fold(0.0f32, |acc, value| acc.max(*value));

            let db = 20.0 * (magnitude / self.size as f32).max(1e-9).log10();
            let level = ((db as f64 - MIN_DB) / (MAX_DB - MIN_DB)).clamp(0.0, 1.0) as f32;

            peak = peak.max(level);
            bars.push(level);
        }

        // Автоматическая нормализация: тихая музыка даёт низкие бины, но
        // полоса всё равно должна быть выразительной.
        if peak > 0.0 {
            let gain = (0.85 / peak).clamp(1.0, 6.0);
            for level in &mut bars {
                *level = (*level * gain).clamp(0.0, 1.0);
            }
        }

        bars
    }

    /// БПФ на месте: сначала бит-реверс, затем «бабочки».
    fn transform(&mut self) {
        let mut length = 2;

        while length <= self.size {
            let half = length / 2;
            let step = self.size / length;

            for start in (0..self.size).step_by(length) {
                for offset in 0..half {
                    let cos = self.cos[offset * step];
                    let sin = self.sin[offset * step];
                    let even = start + offset;
                    let odd = even + half;

                    let odd_real = self.real[odd] * cos - self.imag[odd] * sin;
                    let odd_imag = self.real[odd] * sin + self.imag[odd] * cos;

                    self.real[odd] = self.real[even] - odd_real;
                    self.imag[odd] = self.imag[even] - odd_imag;
                    self.real[even] += odd_real;
                    self.imag[even] += odd_imag;
                }
            }

            length *= 2;
        }

        // Дальше нужны только амплитуды: комплекс хранить незачем.
        for index in 0..self.size {
            self.real[index] = (self.real[index] * self.real[index]
                + self.imag[index] * self.imag[index])
                .sqrt();
        }
    }
}

/// Частотная граница полоски: нотная сетка, по одному полутону на полоску.
///
/// Полоска `i` отвечает ноте с MIDI-номером `LOW_MIDI + i`, а её верхняя
/// граница — следующий полутон. Так столбик спектра и столбик ноты —
/// одно и то же место на полосе.
fn bar_edge(bar: usize, count: usize, bins: usize) -> usize {
    let span = (HIGH_MIDI - LOW_MIDI + 1) as f64;
    let midi = LOW_MIDI as f64 + bar as f64 * span / count as f64;
    let hz = 440.0 * 2f64.powf((midi - 69.0) / 12.0);
    let max_hz = SAMPLE_RATE as f64 / 2.0;

    ((hz / max_hz * bins as f64) as usize).clamp(0, bins - 1)
}

/// Номер полоски для ноты: туда же подсвечивается распознанный тон.
fn bar_of_midi(midi: i32, count: usize) -> Option<usize> {
    let index = (midi - LOW_MIDI) as f64 * count as f64 / (HIGH_MIDI - LOW_MIDI + 1) as f64;
    let index = index.floor() as isize;

    (index >= 0 && index < count as isize).then_some(index as usize)
}

/// Пересобирает кадр под новое число полосок, когда окно изменило ширину.
fn resample(bars: &[f32], count: usize) -> Vec<f32> {
    if bars.is_empty() || count == 0 {
        return vec![0.0; count];
    }

    (0..count)
        .map(|index| {
            let position = index as f64 / count as f64 * bars.len() as f64;
            bars[(position as usize).min(bars.len() - 1)]
        })
        .collect()
}

// ───────────────────────────── ноты ─────────────────────────────

/// Определение высоты тона методом YIN (разностная функция с накоплением).
///
/// На выходе — нота вида `A4`, пока тон уверенный и близкий к чистому;
/// иначе — прочерк. Лента запоминает последние ноты, чтобы над полосой
/// читалась мелодия, а не одна дрожащая буква.
struct PitchDetector {
    /// Кольцевое окно анализа: YIN смотрит на два спектральных окна сразу.
    buffer: Vec<f32>,
    /// Разностная функция: переиспользуется между кадрами, чтобы не мусорить.
    difference: Vec<f32>,
    /// Текущая распознанная нота (MIDI); `None` — тона нет.
    note: Option<i32>,
    /// Нота-кандидат: она станет текущей после `PITCH_CONFIRM` кадров подряд.
    candidate: Option<i32>,
    confirmations: u32,
    /// Лента нот: свежая в конце.
    trail: std::collections::VecDeque<i32>,
}

impl PitchDetector {
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(PITCH_WINDOW * 2),
            difference: vec![0.0; PITCH_WINDOW / 2],
            note: None,
            candidate: None,
            confirmations: 0,
            trail: std::collections::VecDeque::with_capacity(NOTE_TRAIL),
        }
    }

    /// Текущая нота строкой: `—`, если тон не распознан.
    fn note(&self) -> String {
        self.note.map(note_label).unwrap_or_else(|| String::from("—"))
    }

    /// Текущая нота в MIDI: её номер ложится на полоску спектра.
    fn note_midi(&self) -> Option<i32> {
        self.note
    }

    /// Лента последних нот.
    fn trail(&self) -> Vec<String> {
        self.trail.iter().copied().map(note_label).collect()
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.note = None;
        self.candidate = None;
        self.confirmations = 0;
        self.trail.clear();
    }

    /// Разбирает очередное окно сэмплов, копя окно анализа.
    fn listen(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);

        // Держим ровно окно анализа: старое вытесняется.
        if self.buffer.len() > PITCH_WINDOW {
            let excess = self.buffer.len() - PITCH_WINDOW;
            self.buffer.drain(..excess);
        }

        if self.buffer.len() < PITCH_WINDOW {
            return;
        }

        match detect_pitch(&self.buffer, &mut self.difference).and_then(note_midi) {
            Some(note) => self.settle(note),
            // Тон пропал: держим последнюю ноту до первого уверенного кадра,
            // чтобы лента не мигала между слогами.
            None => {
                self.candidate = None;
                self.confirmations = 0;
            }
        }
    }

    /// Принимает распознанную ноту, отсеивая дребезг.
    fn settle(&mut self, note: i32) {
        if self.note == Some(note) {
            self.candidate = None;
            self.confirmations = 0;
            return;
        }

        if self.candidate == Some(note) {
            self.confirmations += 1;
        } else {
            self.candidate = Some(note);
            self.confirmations = 1;
        }

        if self.confirmations < PITCH_CONFIRM {
            return;
        }

        self.note = Some(note);
        self.candidate = None;
        self.confirmations = 0;

        // Повторы подряд в ленте не дублируем: это одна и та же нота.
        if self.trail.back() != Some(&note) {
            if self.trail.len() == NOTE_TRAIL {
                self.trail.pop_front();
            }
            self.trail.push_back(note);
        }
    }
}

/// Результат детектора: частота и уверенность от 0 до 1.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pitch {
    hz: f32,
    clarity: f32,
}

/// Сколько сэмплов уходит на определение тона. YIN нужен запас в один период
/// самого низкого тона, поэтому окно анализа шире спектрального.
pub(crate) const PITCH_WINDOW: usize = 2_048;

/// Ищет основной тон в окне сэмплов. Возвращает `None`, если тона нет.
pub(crate) fn detect_pitch(samples: &[f32], difference: &mut [f32]) -> Option<Pitch> {
    let max_lag = (SAMPLE_RATE as f32 / PITCH_MIN_HZ).ceil() as usize;
    let min_lag = (SAMPLE_RATE as f32 / PITCH_MAX_HZ).floor() as usize;

    if samples.len() <= max_lag || difference.len() <= max_lag + 1 {
        return None;
    }

    // Разностная функция: на периоде тона она проваливается почти до нуля.
    // Сравниваем только те пары, для которых есть полный период — иначе
    // хвост окна занижает сумму и настоящий период теряется.
    let span = samples.len() - max_lag;
    difference[0] = 0.0;
    for lag in 1..=max_lag {
        let sum = (0..span)
            .map(|index| {
                let delta = samples[index] - samples[index + lag];
                delta * delta
            })
            .sum::<f32>();
        difference[lag] = sum;
    }

    // Накопленная нормировка: без неё «ноль» разностной функции ничего не значит.
    let mut running = 0.0;
    difference[0] = 1.0;
    for lag in 1..=max_lag {
        running += difference[lag];
        difference[lag] = if running > 0.0 {
            difference[lag] * lag as f32 / running
        } else {
            1.0
        };
    }

    // Первый провал ниже порога — кандидат в период; дальше ищем его минимум.
    let mut lag = min_lag.max(2);
    while lag < max_lag {
        if difference[lag] < PITCH_THRESHOLD {
            while lag + 1 < max_lag && difference[lag + 1] < difference[lag] {
                lag += 1;
            }

            let clarity = (1.0 - difference[lag]).clamp(0.0, 1.0);
            if clarity < PITCH_CLARITY {
                return None;
            }

            // Параболическая интерполяция: без неё частота прыгает по бинам лага.
            let hz = interpolate_hz(difference, lag);
            return Some(Pitch { hz, clarity });
        }

        lag += 1;
    }

    None
}

/// Уточняет период по трём точкам разностной функции и переводит в герцы.
fn interpolate_hz(difference: &[f32], lag: usize) -> f32 {
    // На краю диапазона соседняя точка недоступна: интерполировать нечем.
    if lag == 0 || lag + 1 >= difference.len() {
        return SAMPLE_RATE as f32 / lag.max(1) as f32;
    }

    let (left, center, right) = (difference[lag - 1], difference[lag], difference[lag + 1]);

    let denominator = 2.0 * (left - 2.0 * center + right);
    let shift = if denominator.abs() > f32::EPSILON {
        (left - right) / denominator
    } else {
        0.0
    };

    SAMPLE_RATE as f32 / (lag as f32 + shift)
}

/// Переводит частоту в MIDI-номер; `None`, если нота далека от чистого строя.
pub(crate) fn note_midi(pitch: Pitch) -> Option<i32> {
    if pitch.clarity < PITCH_CLARITY {
        return None;
    }

    // MIDI-номер: 69 = A4 = 440 Гц.
    let midi = 69.0 + 12.0 * (pitch.hz / 440.0).log2();
    let rounded = midi.round();
    let cents = (midi - rounded).abs() * 100.0;

    if cents > PITCH_CENTS {
        return None;
    }

    let index = rounded as i32;
    (0..=127).contains(&index).then_some(index)
}

/// Человекочитаемое имя ноты: 69 — `A4`.
pub(crate) fn note_label(midi: i32) -> String {
    let name = NOTE_NAMES[(midi.rem_euclid(12)) as usize];
    // Октава в стандарте MIDI: 60 — C4.
    let octave = midi / 12 - 1;

    format!("{name}{octave}")
}

/// Обратное преобразование к [[note_label]]: `A4` → 69. `None`, если имя
/// ноты не разобрать (например, прочерк `—`).
fn midi_of_note(label: &str) -> Option<i32> {
    // Имя — буквы, октава — всё после них (со знаком: `C-1`).
    let split = label.find(|ch: char| !ch.is_ascii_alphabetic())?;
    let name = &label[..split];
    let octave: i32 = label[split..].trim().parse().ok()?;
    let pitch_class = NOTE_NAMES.iter().position(|note| *note == name)? as i32;

    // Имя ноты существует для любой октавы, включая отрицательные.
    let midi = (octave + 1) * 12 + pitch_class;
    (0..=127).contains(&midi).then_some(midi)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spectrum_bars(signal: &[f32]) -> Vec<f32> {
        BAR_COUNT.store(DEFAULT_BARS, Ordering::Relaxed);
        let mut spectrum = Spectrum::new(WINDOW);
        spectrum.bars(signal)
    }

    #[test]
    fn finds_the_frequency_of_a_pure_tone() {
        let tone_hz = 440.0;
        let signal: Vec<f32> = (0..WINDOW)
            .map(|index| (2.0 * PI * tone_hz as f32 * index as f32 / SAMPLE_RATE as f32).sin())
            .collect();

        let bars = spectrum_bars(&signal);
        let loudest = bars
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .map(|(index, _)| index)
            .unwrap();

        // 440 Гц — это A4, MIDI 69; на сетке C1…C7 (73 полоски, по полутону
        // на каждую) это 45-я полоска.
        assert!(
            (44..=46).contains(&loudest),
            "тон 440 Гц попал в полоску {loudest}"
        );
        assert!(bars[loudest] > 0.5, "полоска тона слишком тихая");
    }

    /// Диагностика: печатает полоски и ноты реального системного звука.
    /// `cargo test -- --ignored --nocapture live_audio_dump`
    #[test]
    #[ignore]
    fn live_audio_dump() {
        super::live_audio_dump();
    }

    /// Диагностика: слушает файл (через `ffmpeg`) и печатает распознанные ноты.
    /// `METRICKS_TRACK=/путь/к/треку.mp3 cargo test -- --ignored --nocapture track_note_dump`
    #[test]
    #[ignore]
    fn track_note_dump() {
        use std::io::Read as _;

        let Ok(track) = std::env::var("METRICKS_TRACK") else {
            println!("задай METRICKS_TRACK=/путь/к/файлу.mp3");
            return;
        };

        let mut child = Command::new("ffmpeg")
            .args([
                "-v", "error", "-i", &track, "-t", "25", "-f", "f32le", "-ac", "1", "-ar", "44100",
                "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("ffmpeg");

        let mut stdout = child.stdout.take().expect("stdout");
        let mut raw = vec![0u8; WINDOW * 4];
        let mut pitch = PitchDetector::new();

        while stdout.read_exact(&mut raw).is_ok() {
            let samples: Vec<f32> = raw
                .chunks_exact(4)
                .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                .collect();

            pitch.listen(&samples);
            if pitch.note() != "—" {
                print!("{} ", pitch.note());
            }
        }

        let _ = child.kill();
        println!();
    }

    #[test]
    fn silence_gives_empty_bars() {
        let bars = spectrum_bars(&[0.0; WINDOW]);

        assert!(bars.iter().all(|level| *level == 0.0));
    }

    #[test]
    fn bar_edges_are_sorted_and_inside_the_spectrum() {
        let bins = WINDOW / 2;
        let edges: Vec<usize> = (0..=DEFAULT_BARS)
            .map(|bar| bar_edge(bar, DEFAULT_BARS, bins))
            .collect();

        assert!(edges.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(edges[DEFAULT_BARS] < bins);
    }

    #[test]
    fn one_bar_is_one_semitone() {
        // Границы полосок — частоты нот сетки C1…C7: соседние отличаются полутоном.
        let bins = WINDOW / 2;
        let edge_hz = |bar: usize| bar_edge(bar, DEFAULT_BARS, bins) as f64 * SAMPLE_RATE as f64
            / 2.0
            / bins as f64;

        let ratio = edge_hz(12) / edge_hz(0);
        // Границы округляются до бина, поэтому допуск — не лучше половины бина.
        assert!((ratio - 2.0).abs() < 0.1, "октава = {ratio} вместо 2");

        // 440 Гц — A4 (MIDI 69) — лежит внутри своей полоски.
        let a4 = bar_of_midi(69, DEFAULT_BARS);
        assert_eq!(a4, Some(45));
        assert!(edge_hz(45) <= 440.0 && 440.0 < edge_hz(46));
    }

    #[test]
    fn note_labels_round_trip_through_bar_index() {
        // Подсветка полоски строится из MIDI-номера: имя ноты должно
        // собираться и разбираться обратно без потерь.
        for midi in [0, 24, 60, 69, 96, 127] {
            let label = note_label(midi);
            assert_eq!(midi_of_note(&label), Some(midi), "нота {label}");
        }

        assert_eq!(midi_of_note("—"), None);
        assert_eq!(midi_of_note(""), None);
    }

    #[test]
    fn resamples_frames_to_new_width() {
        assert_eq!(resample(&[0.0, 1.0], 4), vec![0.0, 0.0, 1.0, 1.0]);
        assert_eq!(resample(&[], 3), vec![0.0, 0.0, 0.0]);
    }

    /// Синтетический тон: детектор должен назвать ту же ноту.
    fn tone(hz: f32) -> Vec<f32> {
        (0..PITCH_WINDOW)
            .map(|index| (2.0 * PI * hz * index as f32 / SAMPLE_RATE as f32).sin())
            .collect()
    }

    fn detected(signal: &[f32]) -> Option<String> {
        let mut difference = vec![0.0; PITCH_WINDOW / 2];
        detect_pitch(signal, &mut difference)
            .and_then(note_midi)
            .map(note_label)
    }

    #[test]
    fn names_the_note_of_a_pure_tone() {
        assert_eq!(detected(&tone(440.0)).as_deref(), Some("A4"));
        assert_eq!(detected(&tone(880.0)).as_deref(), Some("A5"));
        assert_eq!(detected(&tone(261.63)).as_deref(), Some("C4"));
        assert_eq!(detected(&tone(146.83)).as_deref(), Some("D3"));
    }

    #[test]
    fn silence_has_no_note() {
        assert!(detected(&[0.0; PITCH_WINDOW]).is_none());
    }

    #[test]
    fn out_of_tune_tone_is_not_named() {
        // Четверть тона выше A4 — это уже не «A» в обиходном смысле.
        let between: Vec<f32> = (0..PITCH_WINDOW)
            .map(|index| (2.0 * PI * 452.9 * index as f32 / SAMPLE_RATE as f32).sin())
            .collect();

        assert!(detected(&between).is_none());
    }

    #[test]
    fn trail_collects_notes_without_repeats() {
        let mut pitch = PitchDetector::new();

        for hz in [440.0, 440.0, 493.88, 493.88, 523.25, 523.25] {
            // Каждая нота подтверждается дважды — так же, как на живом звуке.
            for _ in 0..PITCH_CONFIRM {
                pitch.listen(&tone(hz));
            }
        }

        assert_eq!(pitch.note(), "C5");
        assert_eq!(pitch.trail(), vec!["A4", "B4", "C5"]);
    }

    #[test]
    fn trail_is_cleared_on_reset() {
        let mut pitch = PitchDetector::new();
        for _ in 0..PITCH_CONFIRM {
            pitch.listen(&tone(440.0));
        }

        pitch.reset();

        assert_eq!(pitch.note(), "—");
        assert!(pitch.trail().is_empty());
    }
}
