//! Состояние qBittorrent — без WebUI и без ipc-сокета.
//!
//! Веб-интерфейс клиента выключен, зато он постоянно оставляет на диске всё,
//! что карточке нужно: в `~/.local/share/qBittorrent/BT_backup` лежат файлы
//! резюме (имя торрента, каталог сохранения, пауза) и сами `.torrent`-файлы
//! с полным размером. Сколько байтов уже скачано, видно по занятым блокам
//! файлов (`st_blocks`): размещение sparse, поэтому apparent размер полон
//! с первых кусков, а блоки прибывают вместе с прогрессом — ровно то, что
//! нужно для живых процентов, скорости и остатка.

use std::cmp::Reverse;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::status::{eta_seconds, process_running};

/// Сколько живёт один снимок: папку торрента чистим не каждый кадр —
/// торрент на десятки тысяч файлов обошёлся бы дороже самой карточки.
const TTL: Duration = Duration::from_secs(2);
/// Скорость забываем, если занятые блоки не росли столько секунд.
const STALE: Duration = Duration::from_secs(15);

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QbtStatus {
    /// Клиент найден на машине (бинарь или каталог данных).
    pub installed: bool,
    /// Процесс клиента запущен.
    pub running: bool,
    /// Имя торрента, за которым следит карточка.
    pub torrent: String,
    /// Готовность в процентах: занятые блоки на диске против полного размера.
    pub percent: Option<f64>,
    /// Сколько байтов реально лежит на диске.
    pub downloaded: u64,
    /// Полный размер торрента из метаданных; 0, если `.torrent` не прочитать.
    pub total: u64,
    /// Скорость роста файлов на диске в байтах в секунду.
    pub rate: Option<f64>,
    /// Остаток в секундах, пока скорость известна.
    pub eta: Option<u64>,
    /// Человекочитаемое состояние: «загрузка», «сидирование», «пауза»…
    pub state: String,
}

/// Состояние клиента для карточки.
pub fn qbt_status() -> QbtStatus {
    static CACHE: Mutex<Option<Cache>> = Mutex::new(None);

    let mut guard = CACHE.lock().unwrap_or_else(|poison| poison.into_inner());
    let cache = guard.get_or_insert_with(Cache::default);

    if let Some((at, value)) = cache.status.as_ref() {
        if at.elapsed() < TTL {
            return value.clone();
        }
    }

    let value = fresh_status(&mut cache.meter);
    cache.status = Some((Instant::now(), value.clone()));
    value
}

/// Кэш между опросами: снимок статуса и счётчик скорости.
#[derive(Default)]
struct Cache {
    status: Option<(Instant, QbtStatus)>,
    meter: Meter,
}

fn fresh_status(meter: &mut Meter) -> QbtStatus {
    let installed = binary_installed() || data_dir().is_some();
    let running = process_running(&["qbittorrent", "qbittorrent-nox"]);

    if !installed {
        return QbtStatus {
            running,
            state: "не установлен".into(),
            ..Default::default()
        };
    }

    if !running {
        return QbtStatus {
            installed: true,
            state: "не запущен".into(),
            ..Default::default()
        };
    }

    let Some(resume) = active_resume() else {
        return QbtStatus {
            installed: true,
            running: true,
            state: "торрентов нет".into(),
            ..Default::default()
        };
    };

    let total = fs::read(&resume.meta).ok().and_then(|raw| torrent_size(&raw));

    let downloaded = disk_bytes(&resume.target());
    let rate = meter.update(&resume.name, downloaded, STALE);

    let percent = total
        .filter(|bytes| *bytes > 0)
        .map(|bytes| (downloaded as f64 / bytes as f64 * 100.0).min(100.0));

    let eta = match percent {
        Some(value) if value < 100.0 => {
            let remaining = total.unwrap_or(0).saturating_sub(downloaded);
            eta_seconds(remaining, rate)
        }
        _ => None,
    };

    QbtStatus {
        installed: true,
        running: true,
        state: state_label(resume.paused, rate, percent.is_some_and(|value| value >= 100.0))
            .into(),
        torrent: resume.name,
        percent,
        downloaded,
        total: total.unwrap_or(0),
        rate,
        eta,
    }
}

/// Подпись состояния клиента. Порядок важен: пауза перебивает всё, растущие
/// байты важнее прочерка «готово», а незавершённый торрент без роста —
/// это не остановка, клиент просто стоит без пиров.
fn state_label(paused: bool, rate: Option<f64>, complete: bool) -> &'static str {
    if paused {
        "пауза"
    } else if rate.is_some() {
        "загрузка"
    } else if complete {
        "сидирование"
    } else {
        "без активности"
    }
}

/// Бинарь клиента: обычные места установки.
fn binary_installed() -> bool {
    [
        "/usr/bin/qbittorrent",
        "/usr/local/bin/qbittorrent",
        "/opt/qbittorrent/qbittorrent",
    ]
    .iter()
    .any(|path| Path::new(path).exists())
}

/// Каталог данных клиента: в нём живут BT_backup и логи.
fn data_dir() -> Option<PathBuf> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

    DIR.get_or_init(|| {
        let home = std::env::var_os("HOME")?;
        let dir = PathBuf::from(home).join(".local/share/qBittorrent");
        dir.is_dir().then_some(dir)
    })
    .clone()
}

/// Разобранный файл резюме: всё, что карточке нужно из BT_backup.
struct Resume {
    name: String,
    save_path: String,
    paused: bool,
    /// Путь к `.torrent` с метаданными, лежащим рядом.
    meta: PathBuf,
}

impl Resume {
    /// Куда клиент положил файлы торрента.
    fn target(&self) -> PathBuf {
        Path::new(&self.save_path).join(&self.name)
    }
}

/// Выбирает, за каким из торрентов следить: живой важнее поставленного,
/// среди живых — тот, чей файл резюме писался свежее.
fn active_resume() -> Option<Resume> {
    let dir = data_dir()?.join("BT_backup");
    let entries = fs::read_dir(&dir).ok()?;

    let mut candidates: Vec<(bool, Reverse<SystemTime>, Resume)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("fastresume") {
            continue;
        }

        let Ok(raw) = fs::read(&path) else {
            continue;
        };
        let Some(resume) = resume_of(&path, &raw) else {
            continue;
        };

        let mtime = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .unwrap_or(UNIX_EPOCH);

        candidates.push((resume.paused, Reverse(mtime), resume));
    }

    // Сортировка по возрастанию: не на паузе и самое свежее — последнее.
    candidates.sort_by_key(|(paused, mtime, _)| (*paused, *mtime));
    candidates.pop().map(|(_, _, resume)| resume)
}

fn resume_of(path: &Path, raw: &[u8]) -> Option<Resume> {
    let root = parse(raw)?;

    // Имя: `qBt-name` — как его показывают в клиенте; пустое поле значит,
    // что смотрим на имя из метаданных торрента.
    let name = root
        .get(b"qBt-name")
        .and_then(Value::text)
        .filter(|text| !text.is_empty())
        .or_else(|| root.get(b"name").and_then(Value::text))?
        .to_string();

    let save_path = root
        .get(b"qBt-savePath")
        .and_then(Value::text)
        .filter(|text| !text.is_empty())
        .or_else(|| root.get(b"save_path").and_then(Value::text))
        .unwrap_or_default()
        .to_string();

    Some(Resume {
        name,
        save_path,
        paused: root.get(b"paused").and_then(Value::int) == Some(1),
        meta: path.with_extension("torrent"),
    })
}

/// Полный размер торрента из метаданных: один файл (`length`) или дерево
/// (`files` — сумма длин).
fn torrent_size(raw: &[u8]) -> Option<u64> {
    let root = parse(raw)?;
    let info = root.get(b"info")?;

    if let Some(length) = info.get(b"length").and_then(Value::int) {
        return Some(length.max(0) as u64);
    }

    let files = info.get(b"files")?.list()?;
    let sizes: Vec<u64> = files
        .iter()
        .filter_map(|file| file.get(b"length").and_then(Value::int))
        .map(|length| length.max(0) as u64)
        .collect();

    (!sizes.is_empty()).then(|| sizes.iter().sum())
}

/// Сколько байтов пути реально занято на диске.
///
/// `st_blocks * 512` — выделенные блоки. У sparse-файла apparent размер
/// полон с первых кусков, а блоки прибывают по мере записи: прогресс
/// загрузки виден прямо по файлам, без единого запроса к клиенту.
fn disk_bytes(path: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt as _;

    let Ok(meta) = fs::metadata(path) else {
        return 0;
    };

    if !meta.is_dir() {
        return meta.blocks() * 512;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };

    entries
        .flatten()
        .map(|entry| disk_bytes(&entry.path()))
        .sum()
}

/// Скорость по приросту занятых блоков между опросами: как у Steam, только
/// растут здесь файлы на диске, а не счётчик в логе.
#[derive(Default)]
struct Meter {
    last: Option<(String, u64, Instant)>,
    ema: Option<f64>,
    sample_at: Option<Instant>,
}

impl Meter {
    fn update(&mut self, name: &str, bytes: u64, stale: Duration) -> Option<f64> {
        if let Some((previous_name, previous_bytes, at)) = self.last.take() {
            if previous_name != name {
                self.ema = None;
            } else {
                let elapsed = at.elapsed();
                if bytes > previous_bytes && elapsed.as_secs_f64() > 0.2 {
                    let instant = (bytes - previous_bytes) as f64 / elapsed.as_secs_f64();
                    self.ema = Some(match self.ema {
                        Some(ema) => ema * 0.5 + instant * 0.5,
                        None => instant,
                    });
                    self.sample_at = Some(Instant::now());
                } else if self.sample_at.is_some_and(|at| at.elapsed() > stale) {
                    // Байты встали: держать прошлую скорость нельзя, это вранье.
                    self.ema = None;
                }
            }
        }

        self.last = Some((name.to_string(), bytes, Instant::now()));
        self.ema
    }
}

/// Значение bencode: клиент пишет и резюме, и метаданные в этом формате.
#[derive(Debug, PartialEq)]
enum Value {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Dict(Vec<(Vec<u8>, Value)>),
}

impl Value {
    /// Значение по ключу словаря.
    fn get(&self, key: &[u8]) -> Option<&Value> {
        let Value::Dict(entries) = self else {
            return None;
        };

        entries
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    fn int(&self) -> Option<i64> {
        match self {
            Value::Int(value) => Some(*value),
            _ => None,
        }
    }

    /// Строку словаря как UTF-8: битые байты — это не текст, а данные.
    fn text(&self) -> Option<&str> {
        match self {
            Value::Bytes(raw) => std::str::from_utf8(raw).ok(),
            _ => None,
        }
    }

    fn list(&self) -> Option<&[Value]> {
        match self {
            Value::List(items) => Some(items),
            _ => None,
        }
    }
}

/// Разбирает bencode. Ошибка — это просто «файл не читается»: карточка
/// честно покажет «нет данных», а не упадёт на чужом формате.
fn parse(input: &[u8]) -> Option<Value> {
    let mut parser = Parser { input, at: 0 };
    let value = parser.value()?;
    (parser.at == input.len()).then_some(value)
}

struct Parser<'a> {
    input: &'a [u8],
    at: usize,
}

impl<'a> Parser<'a> {
    fn value(&mut self) -> Option<Value> {
        match self.input.get(self.at)? {
            b'd' => {
                self.step(1)?;
                self.dict()
            }
            b'l' => {
                self.step(1)?;
                self.list()
            }
            b'i' => self.int(),
            _ => self.bytes(),
        }
    }

    fn step(&mut self, step: usize) -> Option<()> {
        self.at = self.at.checked_add(step)?;
        (self.at <= self.input.len()).then_some(())
    }

    fn dict(&mut self) -> Option<Value> {
        let mut entries = Vec::new();

        while self.input.get(self.at) != Some(&b'e') {
            let Value::Bytes(key) = self.bytes()? else {
                return None;
            };
            entries.push((key, self.value()?));
        }

        self.step(1)?;
        Some(Value::Dict(entries))
    }

    fn list(&mut self) -> Option<Value> {
        let mut items = Vec::new();

        while self.input.get(self.at) != Some(&b'e') {
            items.push(self.value()?);
        }

        self.step(1)?;
        Some(Value::List(items))
    }

    fn int(&mut self) -> Option<Value> {
        let rest = self.input.get(self.at..)?;
        let end = rest.iter().position(|byte| *byte == b'e')? + self.at;
        let value = std::str::from_utf8(&self.input[self.at + 1..end]).ok()?.parse().ok()?;

        self.at = end + 1;
        Some(Value::Int(value))
    }

    fn bytes(&mut self) -> Option<Value> {
        let rest = self.input.get(self.at..)?;
        let colon = rest.iter().position(|byte| *byte == b':')? + self.at;
        let len: usize = std::str::from_utf8(&self.input[self.at..colon]).ok()?.parse().ok()?;

        let start = colon + 1;
        let end = start.checked_add(len)?;
        let raw = self.input.get(start..end)?.to_vec();

        self.at = end;
        Some(Value::Bytes(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Собирает bencode-словарь из готовых пар «ключ → значение».
    fn dict(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let mut raw = b"d".to_vec();
        for (key, value) in entries {
            raw.extend_from_slice(format!("{}:{}", key.len(), key).as_bytes());
            raw.extend_from_slice(value);
        }
        raw.push(b'e');
        raw
    }

    fn int(value: i64) -> Vec<u8> {
        format!("i{value}e").into_bytes()
    }

    fn text(value: &str) -> Vec<u8> {
        format!("{}:{value}", value.len()).into_bytes()
    }

    fn list(items: &[Vec<u8>]) -> Vec<u8> {
        let mut raw = b"l".to_vec();
        for item in items {
            raw.extend_from_slice(item);
        }
        raw.push(b'e');
        raw
    }

    /// Метаданные одного файла: `length` в словаре info.
    fn single_file_torrent(size: u64) -> Vec<u8> {
        dict(&[("info", dict(&[("length", int(size as i64))]))])
    }

    /// Метаданные дерева: `files` — список словарей с длинами.
    fn multi_file_torrent(sizes: &[u64]) -> Vec<u8> {
        let files: Vec<Vec<u8>> = sizes
            .iter()
            .map(|size| dict(&[("length", int(*size as i64))]))
            .collect();

        dict(&[("info", dict(&[("files", list(&files))]))])
    }

    #[test]
    fn reads_a_single_file_torrent_size() {
        assert_eq!(torrent_size(&single_file_torrent(123)), Some(123));
    }

    #[test]
    fn sums_the_files_of_a_tree_torrent() {
        assert_eq!(torrent_size(&multi_file_torrent(&[40, 60])), Some(100));
        // Дерево без единой длины — размер неизвестен, а не нулевой.
        assert_eq!(torrent_size(&multi_file_torrent(&[])), None);
    }

    #[test]
    fn broken_bencode_is_just_missing_data() {
        // Длина строки больше оставшихся байтов.
        assert_eq!(parse(b"4:abc"), None);
        // Словарь обрывается на ключе.
        assert_eq!(parse(b"d1:qBt-name"), None);
        // Число без закрывающего `e`.
        assert_eq!(parse(b"i42"), None);
    }

    #[test]
    fn resume_prefers_the_client_name_and_keeps_the_fallback() {
        let named = dict(&[
            ("qBt-name", text("свежий торрент")),
            ("save_path", text("/home/foxcoder/Downloads")),
            ("paused", int(1)),
        ]);
        let resume = resume_of(Path::new("/tmp/abc.fastresume"), &named).expect("резюме разобрано");

        assert_eq!(resume.name, "свежий торрент");
        assert_eq!(resume.save_path, "/home/foxcoder/Downloads");
        assert!(resume.paused);
        assert_eq!(resume.meta, PathBuf::from("/tmp/abc.torrent"));

        // Пустое qBt-name — обычное дело: имя берём из метаданных торрента.
        let empty = dict(&[
            ("qBt-name", text("")),
            ("name", text("по метаданным")),
            ("save_path", text("/data")),
            ("paused", int(0)),
        ]);
        let resume = resume_of(Path::new("/tmp/abc.fastresume"), &empty).expect("резюме разобрано");

        assert_eq!(resume.name, "по метаданным");
        assert!(!resume.paused);
    }

    #[test]
    fn sparse_file_grows_blocks_as_data_lands() {
        let dir = std::env::temp_dir().join(format!("metricks-qbt-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("временный каталог");
        let file = dir.join("piece.bin");

        // Выделенный размер без записи — дыра: блоков нет.
        fs::File::create(&file)
            .expect("файл создан")
            .set_len(1 << 20)
            .expect("размер задан");
        let holed = disk_bytes(&file);
        assert!(holed < 4096, "дыра не должна занимать блоки: {holed}");

        // Записанные байты сразу попадают в блоки — по ним и виден прогресс.
        fs::write(&file, vec![7u8; 8192]).expect("запись");
        assert!(disk_bytes(&file) >= 8192, "запись занимает блоки");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn state_names_what_the_client_is_doing() {
        assert_eq!(state_label(true, Some(1.0), false), "пауза");
        assert_eq!(state_label(false, Some(1.0), false), "загрузка");
        assert_eq!(state_label(false, None, true), "сидирование");
        assert_eq!(state_label(false, None, false), "без активности");
    }

    /// Живой снимок с этой машины: `cargo test -- --ignored --nocapture qbt_dump`
    #[test]
    #[ignore]
    fn qbt_dump() {
        println!("{:#?}", super::qbt_status());
    }
}
