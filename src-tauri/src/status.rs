//! Сбор реального состояния машины для карточек дашборда.
//!
//! Всё локально, без сети:
//! - OpenCode читается из своей базы `~/.local/share/opencode/opencode.db`
//!   через CLI `sqlite3` (в режиме чтения);
//! - музыка — через MPRIS на сессионной шине D-Bus (`busctl`);
//! - Steam — по логам клиента и манифестам игр;
//! - система — `/proc` и `df`;
//! - окно в фокусе — `hyprctl activewindow -j`;
//! - git — ветка и изменения в каталоге сессии;
//! - сеть — `nmcli`.
//!
//! Любой источник может отсутствовать: тогда соответствующая карточка
//! получает `available: false` и фронтенд показывает честное «нет данных».

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

/// Сколько последних секунд сессии считаются «сейчас работающей».
const IDLE_AFTER_MS: i64 = 5 * 60 * 1000;

/// Опрос идёт раз в секунду, но `git status` на большом репозитории может
/// считаться сотни миллисекунд — перечитываем его реже.
const GIT_TTL: Duration = Duration::from_secs(5);

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeStatus {
    /// База найдена и в ней есть хотя бы одна сессия.
    pub available: bool,
    /// Заголовок последней сессии — о чём идёт работа.
    pub task: String,
    /// Последний запрос пользователя.
    pub request: String,
    /// Модель, на которой работает сессия.
    pub model: String,
    /// Каталог проекта сессии.
    pub directory: String,
    /// Время последнего обновления сессии (unix, мс).
    pub updated_at: Option<i64>,
    /// Сколько файлов изменено в сессии.
    pub files_changed: Option<i64>,
    /// OpenCode закончил работу и ждёт следующего запроса.
    pub idle: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MusicStatus {
    /// Найден MPRIS-плеер с непустым треком.
    pub available: bool,
    pub track: String,
    pub artist: String,
    /// Позиция и длительность в секундах, если плеер их отдаёт.
    pub position: Option<i64>,
    pub duration: Option<i64>,
    pub playing: bool,
    /// Имя плеера: Firefox, Spotify, mpv…
    pub player: String,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SteamStatus {
    /// Каталог Steam найден.
    pub installed: bool,
    /// Клиент Steam запущен.
    pub running: bool,
    /// Игра, которую качает или обновляет клиент.
    pub game: String,
    /// Процент загрузки по манифесту игры.
    pub percent: Option<f64>,
    /// Скачано и всего байт по манифесту.
    pub downloaded: Option<u64>,
    pub total: Option<u64>,
    /// Скорость в байтах в секунду.
    pub rate: Option<u64>,
    /// Сколько секунд осталось при текущей скорости.
    pub eta: Option<u64>,
    /// Человеческое состояние: «не запущен», «загрузка», «пауза»…
    pub state: String,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    /// Хотя бы память или диск прочитались.
    pub available: bool,
    /// Занятая и общая память в байтах.
    pub memory_used: Option<u64>,
    pub memory_total: Option<u64>,
    /// Занятое и общее место на корневом разделе в байтах.
    pub disk_used: Option<u64>,
    pub disk_total: Option<u64>,
    /// Средняя загрузка за последнюю минуту.
    pub load: Option<f64>,
    /// Время работы системы в секундах.
    pub uptime: Option<u64>,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FocusStatus {
    /// Гипланд ответил и в фокусе есть окно.
    pub available: bool,
    /// Класс окна: `firefox`, `foot`, `steam`…
    pub app: String,
    pub title: String,
    /// Имя рабочего стола.
    pub workspace: String,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    /// Каталог сессии оказался git-репозиторием.
    pub available: bool,
    pub branch: String,
    /// Сколько файлов изменено; `None` — посчитать не удалось.
    pub changed: Option<u64>,
    /// Имя каталога проекта.
    pub directory: String,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    /// Есть активное подключение.
    pub available: bool,
    /// Имя профиля подключения.
    pub connection: String,
    /// Сетевой интерфейс: `wlan0`, `enp3s0`…
    pub device: String,
    /// SSID, если подключение по Wi-Fi.
    pub wifi: String,
    /// Уровень сигнала в процентах, если его отдаёт NetworkManager.
    pub signal: Option<u64>,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusSnapshot {
    pub opencode: OpenCodeStatus,
    pub music: MusicStatus,
    pub steam: SteamStatus,
    pub system: SystemStatus,
    pub focus: FocusStatus,
    pub git: GitStatus,
    pub network: NetworkStatus,
    /// Момент сбора (unix, мс) — по нему фронтенд считает «N мин назад».
    pub collected_at: i64,
}

/// Tauri-команда: срез состояния машины для карточек.
#[tauri::command]
pub fn get_status() -> StatusSnapshot {
    collect()
}

pub fn collect() -> StatusSnapshot {
    let mut guard = CACHE.lock().unwrap_or_else(|poison| poison.into_inner());
    let cache = guard.get_or_insert_with(Cache::default);

    let opencode = opencode_status();
    let git = fresh(&mut cache.git, GIT_TTL, || git_status(&git_workdir(&opencode)));

    StatusSnapshot {
        opencode,
        music: music_status(),
        steam: steam_status(cache),
        system: system_status(),
        focus: focus_status(),
        git,
        network: network_status(),
        collected_at: now_ms(),
    }
}

/// Кэш между опросами: тяжёлые источники плюс предыдущий замер загрузки Steam.
#[derive(Default)]
struct Cache {
    git: Option<(Instant, GitStatus)>,
    progress: Option<Progress>,
    /// Сглаженная скорость загрузки в байтах в секунду.
    rate_ema: Option<f64>,
}

/// Замер прогресса загрузки: по приросту байтов считаем живую скорость,
/// потому что `Current download rate` в логе Steam пишется раз в минуту.
struct Progress {
    app_id: String,
    bytes: u64,
    at: Instant,
}

static CACHE: Mutex<Option<Cache>> = Mutex::new(None);

fn fresh<T: Clone>(
    slot: &mut Option<(Instant, T)>,
    ttl: Duration,
    build: impl FnOnce() -> T,
) -> T {
    if let Some((at, value)) = slot.as_ref() {
        if at.elapsed() < ttl {
            return value.clone();
        }
    }

    let value = build();
    *slot = Some((Instant::now(), value.clone()));
    value
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or_default()
}

/// Запускает внешнюю команду и возвращает stdout. Ошибки не пробрасываются:
/// отсутствие утилиты — это просто «источник недоступен».
fn run(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// Вариант [`run`] без отбрасывания пустого stdout: у `git status` пустой
/// вывод — это «дерево чисто», а не ошибка.
fn run_raw(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ───────────────────────────── OpenCode ─────────────────────────────

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn opencode_db_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".local/share")))?;
    let path = base.join("opencode/opencode.db");
    path.is_file().then_some(path)
}

/// Читает выборку из SQLite через CLI и разбирает JSON-ответ.
fn sqlite_json(db: &Path, query: &str) -> Option<serde_json::Value> {
    let db = db.to_str()?;
    // `.timeout` не даёт команде упасть, если OpenCode в этот момент пишет в базу.
    let raw = run("sqlite3", &["-cmd", ".timeout 3000", "-json", db, query])?;
    serde_json::from_str(&raw).ok()
}

const SESSION_QUERY: &str = "select title, directory, model, agent, time_updated, \
     time_idle, idle_outcome, summary_files from session_v2 \
     order by time_updated desc limit 1";

const REQUEST_QUERY: &str = "select data from session_message \
     where type = 'user' order by seq desc limit 1";

fn opencode_status() -> OpenCodeStatus {
    let Some(db) = opencode_db_path() else {
        return OpenCodeStatus::default();
    };

    let session = sqlite_json(&db, SESSION_QUERY)
        .and_then(|rows| rows.as_array().and_then(|rows| rows.first()).cloned());

    let Some(session) = session else {
        return OpenCodeStatus::default();
    };

    let text = |key: &str| {
        session
            .get(key)
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string()
    };

    let updated_at = session.get("time_updated").and_then(|value| value.as_i64());
    let files_changed = session.get("summary_files").and_then(|value| value.as_i64());
    let idle = session
        .get("idle_outcome")
        .map(|value| !value.is_null())
        .unwrap_or(false)
        && updated_at.is_some_and(|updated| now_ms() - updated > IDLE_AFTER_MS);

    let request = sqlite_json(&db, REQUEST_QUERY)
        .and_then(|rows| rows.as_array().and_then(|rows| rows.first()).cloned())
        .and_then(|row| row.get("data").and_then(|value| value.as_str()).map(str::to_string))
        .and_then(|data| serde_json::from_str::<serde_json::Value>(&data).ok())
        .and_then(|data| data.get("text").and_then(|value| value.as_str()).map(str::to_string))
        .unwrap_or_default();

    OpenCodeStatus {
        available: true,
        task: text("title"),
        request,
        model: model_name(&text("model")),
        directory: text("directory"),
        updated_at,
        files_changed,
        idle,
    }
}

/// В базе модель лежит JSON-строкой вида `{"id":"…","providerID":"…"}`.
fn model_name(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| value.get("id").and_then(|id| id.as_str()).map(str::to_string))
        .unwrap_or_else(|| raw.to_string())
}

// ───────────────────────────── музыка (MPRIS) ─────────────────────────────

const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
const MPRIS_PATH: &str = "/org/mpris/MediaPlayer2";

#[derive(Debug, Default)]
struct Player {
    name: String,
    title: String,
    artist: String,
    duration: Option<i64>,
    position: Option<i64>,
    playing: bool,
}

fn mpris_players() -> Vec<String> {
    let Some(raw) = run("busctl", &["--user", "--json=short", "list"]) else {
        return Vec::new();
    };
    let Ok(services) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };

    services
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("name").and_then(|name| name.as_str()))
                .filter(|name| name.starts_with(MPRIS_PREFIX))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Забирает одно свойство MPRIS и возвращает его `data` из `busctl --json`.
fn mpris_property(bus_name: &str, interface: &str, property: &str) -> Option<serde_json::Value> {
    let raw = run(
        "busctl",
        &[
            "--user",
            "--json=short",
            "get-property",
            bus_name,
            MPRIS_PATH,
            interface,
            property,
        ],
    )?;
    let value = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    value.get("data").cloned()
}

fn mpris_player(bus_name: &str) -> Player {
    let player_interface = "org.mpris.MediaPlayer2.Player";

    let metadata = mpris_property(bus_name, player_interface, "Metadata").unwrap_or_default();
    let entry = |key: &str| metadata.get(key).and_then(|item| item.get("data")).cloned();

    let title = entry("xesam:title")
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default();
    let artist = entry("xesam:artist")
        .and_then(|value| {
            value
                .as_array()
                .and_then(|items| items.first())
                .and_then(|item| item.as_str())
                .map(str::to_string)
        })
        .unwrap_or_default();
    let duration = entry("mpris:length")
        .and_then(|value| value.as_i64())
        .map(|micros| micros / 1_000_000);
    let position = mpris_property(bus_name, player_interface, "Position")
        .and_then(|value| value.as_i64())
        .map(|micros| micros / 1_000_000);
    let playing = mpris_property(bus_name, player_interface, "PlaybackStatus")
        .and_then(|value| value.as_str().map(|status| status == "Playing"))
        .unwrap_or(false);

    let player = mpris_property(bus_name, "org.mpris.MediaPlayer2", "Identity")
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| fallback_player_name(bus_name));

    Player {
        name: player,
        title,
        artist,
        duration,
        position,
        playing,
    }
}

/// `org.mpris.MediaPlayer2.firefox.instance_1_51` → `Firefox`.
fn fallback_player_name(bus_name: &str) -> String {
    let tail = bus_name.strip_prefix(MPRIS_PREFIX).unwrap_or(bus_name);
    let head = tail.split('.').next().unwrap_or(tail);
    let mut chars = head.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn music_status() -> MusicStatus {
    let mut players: Vec<Player> = mpris_players()
        .iter()
        .map(|bus_name| mpris_player(bus_name))
        .filter(|player| !player.title.is_empty())
        .collect();

    // Играющий плеер важнее приостановленного, свежий — важнее старого.
    players.sort_by_key(|player| !player.playing);

    let Some(player) = players.into_iter().next() else {
        return MusicStatus::default();
    };

    MusicStatus {
        available: true,
        track: player.title,
        artist: player.artist,
        position: player.position,
        duration: player.duration,
        playing: player.playing,
        player: player.name,
    }
}

// ───────────────────────────── Steam ─────────────────────────────

fn steam_root() -> Option<PathBuf> {
    let home = home_dir()?;
    [
        home.join(".steam/steam"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/data/Steam"),
    ]
    .into_iter()
    .find(|path| path.is_dir())
}

/// Ищет процесс по имени в `/proc`: Steam не пишет pid-файл, зато его процесс
/// всегда виден в списке.
fn process_running(names: &[&str]) -> bool {
    let Ok(entries) = fs::read_dir("/proc") else {
        return false;
    };

    entries.flatten().any(|entry| {
        entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.chars().all(|ch| ch.is_ascii_digit()))
            && fs::read_to_string(entry.path().join("comm"))
                .map(|comm| names.contains(&comm.trim()))
                .unwrap_or(false)
    })
}

/// Читает хвост файла: логи Steam бывают в десятки мегабайт.
fn read_tail(path: &Path, max_bytes: u64) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    file.seek(SeekFrom::Start(len.saturating_sub(max_bytes))).ok()?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).ok()?;
    Some(String::from_utf8_lossy(&buffer).into_owned())
}

/// Состояние обновления из `steamapps/appmanifest_<id>.acf`.
#[derive(Debug)]
struct Manifest {
    app_id: String,
    name: String,
    state_flags: u64,
    downloaded: u64,
    total: u64,
    /// Время правки файла: свежий манифест — актуальная загрузка.
    modified: SystemTime,
}

const STATE_UPDATE_QUEUED: u64 = 8;
const STATE_UPDATE_RUNNING: u64 = 256;
const STATE_UPDATE_PAUSED: u64 = 512;
const STATE_UPDATE_STARTED: u64 = 1024;

/// Значение ключа из формата Valve: `"name"\t\t"Dota 2"`.
fn acf_value(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let rest = text.split(&needle).nth(1)?.trim_start();
    let rest = rest.strip_prefix('"')?;

    rest.split('"').next().map(str::to_string)
}

fn acf_number(text: &str, key: &str) -> Option<u64> {
    acf_value(text, key)?.parse().ok()
}

/// Все `steamapps` клиента: основная библиотека плюс подключённые в
/// `libraryfolders.vdf` (игры часто стоят на другом диске).
fn library_steamapps(root: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![root.join("steamapps")];

    if let Ok(vdf) = fs::read_to_string(root.join("steamapps/libraryfolders.vdf")) {
        for line in vdf.lines() {
            let Some(rest) = line.trim_start().strip_prefix("\"path\"") else {
                continue;
            };
            let Some(rest) = rest.trim_start().strip_prefix('"') else {
                continue;
            };
            let Some(path) = rest.split('"').next() else {
                continue;
            };

            let candidate = Path::new(path).join("steamapps");
            if candidate.is_dir() {
                dirs.push(candidate);
            }
        }
    }

    dirs.sort();
    dirs.dedup();
    dirs
}

fn manifests(root: &Path) -> Vec<Manifest> {
    let mut items = Vec::new();

    for dir in library_steamapps(root) {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !file_name.starts_with("appmanifest_") || !file_name.ends_with(".acf") {
                continue;
            }

            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let (Some(total), Some(downloaded)) = (
                acf_number(&text, "BytesToDownload"),
                acf_number(&text, "BytesDownloaded"),
            ) else {
                continue;
            };

            let app_id = file_name
                .trim_start_matches("appmanifest_")
                .trim_end_matches(".acf")
                .to_string();

            items.push(Manifest {
                name: acf_value(&text, "name").unwrap_or_else(|| format!("app {app_id}")),
                state_flags: acf_number(&text, "StateFlags").unwrap_or_default(),
                downloaded,
                total,
                modified: entry
                    .metadata()
                    .ok()
                    .and_then(|meta| meta.modified().ok())
                    .unwrap_or(UNIX_EPOCH),
                app_id,
            });
        }
    }

    items
}

/// Игра, у которой скачано меньше, чем нужно. Идущая загрузка важнее
/// приостановленной, свежий манифест — важнее старого.
fn pending_download(items: &[Manifest]) -> Option<&Manifest> {
    items
        .iter()
        .filter(|manifest| manifest.total > 0 && manifest.downloaded < manifest.total)
        .max_by_key(|manifest| {
            (
                manifest.state_flags & STATE_UPDATE_RUNNING != 0,
                manifest.modified,
            )
        })
}

/// Последняя скорость из `content_log.txt`: `Current download rate: 60.386 Mbps`.
/// Steam пишет мегабиты, карточке нужны байты в секунду.
fn last_download_rate(root: &Path) -> Option<f64> {
    let log = read_tail(&root.join("logs/content_log.txt"), 256 * 1024)?;

    log.lines()
        .filter_map(|line| {
            let rest = line.split("Current download rate:").nth(1)?;
            let mbps: f64 = rest.split_whitespace().next()?.parse().ok()?;

            Some(mbps * 1_000_000.0 / 8.0)
        })
        .next_back()
}

fn eta_seconds(remaining: u64, rate: Option<f64>) -> Option<u64> {
    let rate = rate.filter(|value| *value > 0.0)?;

    Some((remaining as f64 / rate).ceil() as u64)
}

/// Скорость по приросту скачанных байтов между опросами. Манифест Steam
/// обновляется чаще, чем строка `Current download rate` в логе, поэтому
/// дельта точнее; рывки сглаживаем, а нулю не даём затереть живую оценку.
fn measured_rate(cache: &mut Cache, app_id: &str, downloaded: u64) -> Option<f64> {
    let mut sample = None;

    if let Some(previous) = cache.progress.take() {
        let elapsed = previous.at.elapsed().as_secs_f64();

        if previous.app_id == app_id {
            if downloaded > previous.bytes && elapsed > 0.2 {
                sample = Some((downloaded - previous.bytes) as f64 / elapsed);
            }
        } else {
            cache.rate_ema = None;
        }
    }

    cache.progress = Some(Progress {
        app_id: app_id.to_string(),
        bytes: downloaded,
        at: Instant::now(),
    });

    if let Some(instant) = sample {
        cache.rate_ema = Some(match cache.rate_ema {
            Some(ema) => ema * 0.5 + instant * 0.5,
            None => instant,
        });
    }

    cache.rate_ema
}

fn steam_status(cache: &mut Cache) -> SteamStatus {
    let running = process_running(&["steam", "steamwebhelper"]);

    let Some(root) = steam_root() else {
        return SteamStatus {
            installed: false,
            running,
            state: if running { "запущен".into() } else { "не установлен".into() },
            ..Default::default()
        };
    };

    if !running {
        return SteamStatus {
            installed: true,
            running: false,
            state: "не запущен".into(),
            ..Default::default()
        };
    }

    let items = manifests(&root);
    let Some(download) = pending_download(&items) else {
        return SteamStatus {
            installed: true,
            running: true,
            state: "нет загрузок".into(),
            ..Default::default()
        };
    };

    let state = if download.state_flags & STATE_UPDATE_RUNNING != 0 {
        "загрузка"
    } else if download.state_flags & STATE_UPDATE_QUEUED != 0 {
        "в очереди"
    } else if download.state_flags & (STATE_UPDATE_PAUSED | STATE_UPDATE_STARTED) != 0 {
        "пауза"
    } else {
        "загрузка"
    };

    let rate = measured_rate(cache, &download.app_id, download.downloaded)
        .or_else(|| last_download_rate(&root));
    let remaining = download.total.saturating_sub(download.downloaded);

    SteamStatus {
        installed: true,
        running: true,
        game: download.name.clone(),
        percent: Some(download.downloaded as f64 / download.total as f64 * 100.0),
        downloaded: Some(download.downloaded),
        total: Some(download.total),
        rate: rate.map(|value| value.round() as u64),
        eta: eta_seconds(remaining, rate),
        state: state.into(),
    }
}

// ───────────────────────────── система ─────────────────────────────

/// `/proc/meminfo` хранит значения в килобайтах: `MemTotal:  32629828 kB`.
fn meminfo_bytes(text: &str, key: &str) -> Option<u64> {
    let line = text
        .lines()
        .find(|line| line.split(':').next() == Some(key))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;

    Some(kb.saturating_mul(1024))
}

fn system_status() -> SystemStatus {
    let mut status = SystemStatus::default();

    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        let total = meminfo_bytes(&meminfo, "MemTotal");
        // MemAvailable учитывает кеш, который ядро отдаст под нужды приложений.
        let available = meminfo_bytes(&meminfo, "MemAvailable");

        if let (Some(total), Some(available)) = (total, available) {
            status.memory_total = Some(total);
            status.memory_used = Some(total.saturating_sub(available));
        }
    }

    if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
        status.load = loadavg
            .split_whitespace()
            .next()
            .and_then(|value| value.parse().ok());
    }

    if let Ok(uptime) = fs::read_to_string("/proc/uptime") {
        status.uptime = uptime
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<f64>().ok())
            .map(|seconds| seconds as u64);
    }

    // `--output` есть только в GNU coreutils, зато не зависит от локали
    // и не требует разбирать колонки переменной ширины.
    if let Some(df) = run("df", &["-B1", "--output=used,size", "/"]) {
        if let Some(line) = df.lines().nth(1) {
            let mut fields = line.split_whitespace();
            status.disk_used = fields.next().and_then(|value| value.parse().ok());
            status.disk_total = fields.next().and_then(|value| value.parse().ok());
        }
    }

    status.available = status.memory_total.is_some() || status.disk_total.is_some();
    status
}

// ───────────────────────────── окно в фокусе ─────────────────────────────

fn focus_status() -> FocusStatus {
    let Some(raw) = run("hyprctl", &["activewindow", "-j"]) else {
        return FocusStatus::default();
    };
    let Ok(window) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return FocusStatus::default();
    };

    // У части окон класс пустой, у части — заголовок: ориентируемся на оба.
    let text = |value: Option<&serde_json::Value>| {
        value
            .and_then(|item| item.as_str())
            .unwrap_or_default()
            .to_string()
    };

    let app = text(window.get("class"));
    let title = text(window.get("title"));
    let workspace = text(window.get("workspace").and_then(|item| item.get("name")));

    FocusStatus {
        available: !app.is_empty() || !title.is_empty(),
        app,
        title,
        workspace,
    }
}

// ───────────────────────────── git ─────────────────────────────

/// Git показываем для проекта, в котором идёт сессия OpenCode:
/// это и есть «текущая работа» на машине.
fn git_workdir(opencode: &OpenCodeStatus) -> PathBuf {
    let session_dir = PathBuf::from(&opencode.directory);
    if !opencode.directory.is_empty() && session_dir.is_dir() {
        return session_dir;
    }

    home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

fn git_status(directory: &Path) -> GitStatus {
    let Some(dir) = directory.to_str() else {
        return GitStatus::default();
    };

    let Some(branch) = run("git", &["-C", dir, "rev-parse", "--abbrev-ref", "HEAD"]) else {
        return GitStatus::default();
    };

    // Пустой вывод `--porcelain` — чистое дерево, поэтому здесь нужен run_raw.
    let changed = run_raw("git", &["-C", dir, "status", "--porcelain"])
        .map(|out| out.lines().filter(|line| !line.trim().is_empty()).count() as u64);

    GitStatus {
        available: true,
        branch,
        changed,
        directory: directory
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
    }
}

// ───────────────────────────── сеть ─────────────────────────────

fn network_status() -> NetworkStatus {
    let Some(raw) = run("nmcli", &["-t", "-f", "DEVICE,STATE,CONNECTION", "device"]) else {
        return NetworkStatus::default();
    };

    // Строки `-t` разделены двоеточиями. SSID и имя профиля могут содержать
    // экранированные `\:`, поэтому склеиваем хвост обратно.
    let Some((device, connection)) = raw.lines().find_map(|line| {
        let mut fields = line.splitn(3, ':');
        let device = fields.next()?;
        let state = fields.next()?;
        let connection = fields.next().unwrap_or_default();

        (state == "connected" && device != "lo")
            .then(|| (device.to_string(), connection.replace("\\:", ":")))
    }) else {
        return NetworkStatus::default();
    };

    let mut status = NetworkStatus {
        available: true,
        device,
        connection,
        ..Default::default()
    };

    if status.device.starts_with("wl") {
        let wifi = run("nmcli", &["-t", "-f", "ACTIVE,SSID,SIGNAL", "device", "wifi"]);

        if let Some(lines) = wifi {
            // Активная сеть — та, у которой ACTIVE:yes.
            for line in lines.lines() {
                let mut fields = line.splitn(3, ':');
                if fields.next() != Some("yes") {
                    continue;
                }

                status.wifi = fields.next().unwrap_or_default().replace("\\:", ":");
                status.signal = fields.next().and_then(|value| value.parse().ok());
                break;
            }
        }
    }

    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_app_manifest() {
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"440\"\n\t\"name\"\t\t\"Team Fortress 2\"\n\t\"StateFlags\"\t\t\"1026\"\n\t\"BytesToDownload\"\t\t\"17438430624\"\n\t\"BytesDownloaded\"\t\t\"5610909440\"\n}";

        assert_eq!(acf_value(text, "name").as_deref(), Some("Team Fortress 2"));
        assert_eq!(acf_number(text, "StateFlags"), Some(1026));
        assert_eq!(acf_number(text, "BytesDownloaded"), Some(5610909440));
        assert_eq!(acf_value(text, "missing"), None);
    }

    #[test]
    fn estimates_eta_from_rate() {
        assert_eq!(eta_seconds(4000, Some(1000.0)), Some(4));
        assert_eq!(eta_seconds(4000, Some(0.0)), None);
        assert_eq!(eta_seconds(4000, None), None);
    }

    #[test]
    fn falls_back_to_bus_name() {
        assert_eq!(
            fallback_player_name("org.mpris.MediaPlayer2.firefox.instance_1_51"),
            "Firefox"
        );
    }

    #[test]
    fn reads_meminfo_in_bytes() {
        let text = "MemTotal:       32629828 kB\nMemFree:         1024 kB\nMemAvailable:   25753644 kB\n";

        assert_eq!(meminfo_bytes(text, "MemTotal"), Some(32629828 * 1024));
        assert_eq!(meminfo_bytes(text, "MemFree"), Some(1024 * 1024));
        // ключи-префиксы не должны цепляться за чужие строки
        assert_eq!(meminfo_bytes(text, "MemFree"), Some(1024 * 1024));
        assert_eq!(meminfo_bytes(text, "Mem"), None);
    }

    /// Диагностика: печатает реальный срез машины.
    /// `cargo test -- --ignored --nocapture live_snapshot_dump`
    #[test]
    #[ignore]
    fn live_snapshot_dump() {
        println!(
            "{}",
            serde_json::to_string_pretty(&collect()).expect("снапшот сериализуется")
        );
    }
}
