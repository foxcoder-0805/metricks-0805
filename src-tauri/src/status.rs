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
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

/// Сколько последних секунд сессии считаются «сейчас работающей».
const IDLE_AFTER_MS: i64 = 5 * 60 * 1000;

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
    /// Процент загрузки, если его удалось достать из лога.
    pub percent: Option<f64>,
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
    let opencode = opencode_status();
    let git = git_status(&git_workdir(&opencode));

    StatusSnapshot {
        opencode,
        music: music_status(),
        steam: steam_status(),
        system: system_status(),
        focus: focus_status(),
        git,
        network: network_status(),
        collected_at: now_ms(),
    }
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

#[derive(Debug, PartialEq)]
struct Download {
    app_id: String,
    percent: Option<f64>,
    paused: bool,
}

/// Разбирает строку `content_log.txt`. Формат у Valve менялся, поэтому
/// цепляемся за устойчивые куски: `Downloading app <id>` / `AppID <id>`,
/// процент или пары «скачано/всего».
fn parse_download_line(line: &str) -> Option<Download> {
    let app_id = extract_app_id(line)?;
    let paused = line.contains("Paused") || line.contains("paused");

    Some(Download {
        app_id,
        percent: extract_percent(line).or_else(|| percent_from_bytes(line)),
        paused,
    })
}

fn extract_app_id(line: &str) -> Option<String> {
    for marker in ["Downloading app ", "AppID "] {
        if let Some(rest) = line.split(marker).nth(1) {
            let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
            if !digits.is_empty() {
                return Some(digits);
            }
        }
    }
    None
}

/// Ищет число перед последним знаком `%` в строке: в логе перед ним бывает
/// служебный `[ 42%]`-префикс, а прогресс идёт дальше.
fn extract_percent(line: &str) -> Option<f64> {
    let percent_index = line.rfind('%')?;
    let head = line[..percent_index].trim_end();
    let number: String = head
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    number.parse::<f64>().ok()
}

/// `"... 1234 bytes downloaded, 5678 bytes total"` → процент.
fn percent_from_bytes(line: &str) -> Option<f64> {
    let downloaded = number_before(line, "bytes downloaded")?;
    let total = number_before(line, "bytes total")?;
    (total > 0.0).then(|| (downloaded / total * 100.0).min(100.0))
}

fn number_before(line: &str, marker: &str) -> Option<f64> {
    let head = line.split(marker).next()?.trim_end();
    let number: String = head
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    number.parse::<f64>().ok()
}

/// Имя игры из манифеста `steamapps/appmanifest_<id>.acf`.
fn manifest_name(steam_root: &Path, app_id: &str) -> Option<String> {
    let path = steam_root
        .join("steamapps")
        .join(format!("appmanifest_{app_id}.acf"));
    let text = fs::read_to_string(path).ok()?;

    let mut rest = text.split("\"name\"").nth(1)?;
    rest = rest.trim_start().strip_prefix('"')?;
    rest.split('"').next().map(str::to_string)
}

fn last_download(steam_root: &Path) -> Option<Download> {
    let log = read_tail(&steam_root.join("logs/content_log.txt"), 512 * 1024)?;

    log.lines()
        .filter_map(parse_download_line)
        .next_back()
}

fn steam_status() -> SteamStatus {
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

    let Some(download) = last_download(&root) else {
        return SteamStatus {
            installed: true,
            running: true,
            state: "нет загрузок".into(),
            ..Default::default()
        };
    };

    let game = manifest_name(&root, &download.app_id)
        .unwrap_or_else(|| format!("app {}", download.app_id));

    let state = if download.paused { "пауза" } else { "загрузка" };

    SteamStatus {
        installed: true,
        running: true,
        game,
        percent: download.percent,
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
    fn parses_download_lines() {
        let download =
            parse_download_line("[ 42%] Downloading app 22380 : 42.5% of 1.2 GB").unwrap();
        assert_eq!(download.app_id, "22380");
        assert_eq!(download.percent, Some(42.5));

        let download = parse_download_line("AppID 570 : Downloading, 500 bytes downloaded, 1000 bytes total").unwrap();
        assert_eq!(download.app_id, "570");
        assert_eq!(download.percent, Some(50.0));

        assert!(parse_download_line("no download here").is_none());
    }

    #[test]
    fn parses_manifest_name() {
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"570\"\n\t\"name\"\t\t\"Dota 2\"\n}";
        let mut rest = text.split("\"name\"").nth(1).unwrap();
        rest = rest.trim_start().strip_prefix('"').unwrap();
        assert_eq!(rest.split('"').next().unwrap(), "Dota 2");
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
