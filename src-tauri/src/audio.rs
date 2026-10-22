//! Спектр системного звука для полосы внизу окна.
//!
//! Свой мини-«cava»: `parec` отдаёт монитор вывода PipeWire/PulseAudio,
//! поток разбирается на окна, к ним применяется окно Ханна и БПФ, и
//! фронтенд получает уже готовые уровни полосок.
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

/// Частота дискретизации и размер окна БПФ. При 44.1 кГц окно даёт
/// примерно 21 Гц на бин — для 32 полосок этого хватает.
const SAMPLE_RATE: usize = 44_100;
const WINDOW: usize = 1024;
/// Гистерезис: после последнего громкого кадра визуализатор держится
/// ещё немного, чтобы полоса не мигала на паузах между треками.
const HOLD: Duration = Duration::from_secs(2);
/// Ниже этого RMS считаем, что звука нет — как `silence` в cava.
const SILENCE: f32 = 0.004;

/// Сколько полосок рисует фронтенд по умолчанию.
const DEFAULT_BARS: usize = 32;
/// Диапазон спектра, который раскладывается по полоскам.
const LOW_HZ: f64 = 40.0;
const HIGH_HZ: f64 = 16_000.0;
const MIN_DB: f64 = -72.0;
const MAX_DB: f64 = -6.0;

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AudioBars {
    /// Есть звук прямо сейчас (или был мгновение назад).
    pub active: bool,
    /// Уровни полосок от 0 до 1.
    pub bars: Vec<f32>,
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
        Some(frame) => AudioBars {
            active: frame.active,
            bars: resample(&frame.bars, BAR_COUNT.load(Ordering::Relaxed)),
        },
        None => AudioBars::default(),
    }
}

impl AudioBars {
    fn fits(&self, count: usize) -> bool {
        self.bars.len() == count
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
        let mut raw = vec![0u8; 4 * WINDOW];
        let mut samples: Vec<f32> = Vec::with_capacity(WINDOW * 2);
        let mut spectrum = Spectrum::new(WINDOW);
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

            // Шаг в половину окна: полоса обновляется чаще, чем раз в окно.
            while samples.len() >= WINDOW {
                let rms = (samples.iter().map(|value| value * value).sum::<f32>()
                    / WINDOW as f32)
                    .sqrt();

                if rms > SILENCE {
                    last_loud = Some(std::time::Instant::now());
                }

                let active = last_loud.is_some_and(|at| at.elapsed() < HOLD);
                let bars = if active {
                    spectrum.bars(&samples[..WINDOW])
                } else {
                    vec![0.0; BAR_COUNT.load(Ordering::Relaxed)]
                };

                lock().latest = Some(AudioBars { active, bars });
                samples.drain(..WINDOW / 2);
            }
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

/// Диагностика: печатает полоски реального системного звука.
/// `cargo test -- --ignored --nocapture live_audio_dump`
#[cfg(test)]
fn live_audio_dump() {
    for _ in 0..25 {
        let frame = get_audio_bars();
        println!(
            "active={} bars={:?}",
            frame.active,
            frame
                .bars
                .iter()
                .map(|level| (level * 9.0).round() as u8)
                .collect::<Vec<_>>()
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

    /// Полоски спектра в диапазоне от `LOW_HZ` до `HIGH_HZ`.
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

/// Частотная граница полоски: логарифмическая шкала, как в cava/эквалайзерах.
fn bar_edge(bar: usize, count: usize, bins: usize) -> usize {
    let ratio = bar as f64 / count as f64;
    let hz = LOW_HZ * (HIGH_HZ / LOW_HZ).powf(ratio);
    let max_hz = SAMPLE_RATE as f64 / 2.0;

    ((hz / max_hz * bins as f64) as usize).clamp(0, bins - 1)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn spectrum_bars(signal: &[f32]) -> Vec<f32> {
        BAR_COUNT.store(32, Ordering::Relaxed);
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

        // 440 Гц при логарифмической шкале 40…16000 попадает в 12-ю полоску
        // из 32 (перекрытие соседних полосок даёт небольшую вилку).
        assert!(
            (11..=13).contains(&loudest),
            "тон 440 Гц попал в полоску {loudest}"
        );
        assert!(bars[loudest] > 0.5, "полоска тона слишком тихая");
    }

    /// Диагностика: печатает полоски реального системного звука.
    #[test]
    #[ignore]
    fn live_audio_dump() {
        super::live_audio_dump();
    }

    #[test]
    fn silence_gives_empty_bars() {
        let bars = spectrum_bars(&[0.0; WINDOW]);

        assert!(bars.iter().all(|level| *level == 0.0));
    }

    #[test]
    fn bar_edges_are_sorted_and_inside_the_spectrum() {
        let bins = WINDOW / 2;
        let edges: Vec<usize> = (0..=32).map(|bar| bar_edge(bar, 32, bins)).collect();

        assert!(edges.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(edges[32] < bins);
    }

    #[test]
    fn resamples_frames_to_new_width() {
        assert_eq!(resample(&[0.0, 1.0], 4), vec![0.0, 0.0, 1.0, 1.0]);
        assert_eq!(resample(&[], 3), vec![0.0, 0.0, 0.0]);
    }
}
