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
use std::sync::{Mutex, OnceLock};
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
    /// Открытые сессии: свежие сверху.
    pub sessions: Vec<OpenCodeSession>,
}

/// Одна открытая сессия OpenCode.
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeSession {
    /// Идентификатор сессии — по нему фронтенд различает карточки.
    pub id: String,
    /// Заголовок сессии — о чём идёт работа.
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
    /// Что OpenCode делает прямо сейчас: «думает», «пишет в файл «…»».
    pub activity: String,
    /// Сессия закончила работу и ждёт следующего запроса.
    pub idle: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MusicStatus {
    /// Найден MPRIS-плеер с непустым треком.
    pub available: bool,
    pub track: String,
    pub artist: String,
    /// Позиция и длительность в миллисекундах, если плеер их отдаёт:
    /// MPRIS хранит микросекунды, и целые секунды здесь теряли точность.
    pub position_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub playing: bool,
    /// Имя плеера: Firefox, Spotify, mpv…
    pub player: String,
    /// Адрес трека (`xesam:url`): по нему узнаётся эталонный трек.
    pub url: String,
    /// Ожидаемые ноты эталонного трека на текущей позиции. Для чужого звука
    /// список пуст — там работает только живой детектор.
    pub expected_notes: Vec<String>,
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
pub struct UserStatus {
    /// Логин владельца сессии: `foxcoder`.
    pub login: String,
    /// Домашний каталог — карточке он не нужен, но фронтенду бывает полезен.
    pub home: String,
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
    pub user: UserStatus,
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
        user: user_status(),
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
    /// Когда байты в последний раз реально выросли: по нему понимаем,
    /// что старую скорость пора забыть.
    sample_at: Option<Instant>,
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

/// Смещение местной зоны от UTC в миллисекундах.
///
/// Логи Steam пишут время без зоны, но по местным часам машины. Без поправки
/// строка «свежая» выглядела бы на три часа старше — и загрузка навсегда
/// оставалась бы паузой.
fn local_offset_ms() -> i64 {
    static OFFSET: OnceLock<i64> = OnceLock::new();

    *OFFSET.get_or_init(|| {
        let Some(raw) = run("date", &["+%z"]) else {
            return 0;
        };

        let raw = raw.trim();
        let sign = match raw.as_bytes().first() {
            Some(b'-') => -1,
            _ => 1,
        };
        let digits = raw.trim_start_matches(['+', '-']);

        // Формат `+HHMM`: часы и минуты смещения.
        let (hours, minutes) = match digits.len() {
            4 => (
                digits[..2].parse::<i64>().unwrap_or(0),
                digits[2..].parse::<i64>().unwrap_or(0),
            ),
            _ => (0, 0),
        };

        sign * (hours * 3_600 + minutes * 60) * 1_000
    })
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

/// Логин владельца сессии. Обычно он есть в окружении, а `whoami` нужен
/// только как запасной путь: приложение может быть запущено из окружения
/// без `USER` (например, из автозапуска).
fn user_status() -> UserStatus {
    let login = std::env::var("USER")
        .ok()
        .filter(|name| !name.is_empty())
        .or_else(|| std::env::var("LOGNAME").ok().filter(|name| !name.is_empty()))
        .or_else(|| run("whoami", &[]).map(|name| name.trim().to_string()).filter(|n| !n.is_empty()))
        .unwrap_or_default();

    UserStatus {
        login,
        home: home_dir()
            .and_then(|home| home.to_str().map(str::to_string))
            .unwrap_or_default(),
    }
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

const SESSION_QUERY: &str = "select id, title, directory, model, time_updated, \
     time_idle, idle_outcome, summary_files from session_v2 \
     order by time_updated desc limit 10";

/// Сколько сессий считаются открытыми: те, что трогали за последние сутки.
/// Старые сессии в базе остаются навсегда, но карточек им не полагается.
const SESSION_OPEN_MS: i64 = 24 * 60 * 60 * 1000;

fn opencode_status() -> OpenCodeStatus {
    let Some(db) = opencode_db_path() else {
        return OpenCodeStatus::default();
    };

    let Some(rows) = sqlite_json(&db, SESSION_QUERY)
        .and_then(|value| value.as_array().cloned())
    else {
        return OpenCodeStatus::default();
    };

    let now = now_ms();
    let sessions: Vec<OpenCodeSession> = rows
        .iter()
        .map(|row| session_status(&db, row, now))
        // Открытые сессии: те, что трогали за последние сутки. Остальные
        // остаются в базе навсегда, но карточек им не полагается.
        .filter(|session| {
            session
                .updated_at
                .is_some_and(|updated| now - updated < SESSION_OPEN_MS)
        })
        .collect();

    OpenCodeStatus {
        available: !sessions.is_empty(),
        sessions,
    }
}

/// Собирает карточку одной сессии: заголовок, запрос, активность.
fn session_status(db: &Path, row: &serde_json::Value, now: i64) -> OpenCodeSession {
    let text = |key: &str| {
        row.get(key)
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string()
    };

    let id = text("id");
    let updated_at = row.get("time_updated").and_then(|value| value.as_i64());
    let files_changed = row.get("summary_files").and_then(|value| value.as_i64());
    // Сессию считаем idle, только если она давно не обновлялась: свежая
    // отметка означает, что OpenCode прямо сейчас работает.
    let idle = row
        .get("idle_outcome")
        .map(|value| !value.is_null())
        .unwrap_or(false)
        && updated_at.is_some_and(|updated| now - updated > IDLE_AFTER_MS);

    let request = last_message(db, &id, "user")
        .and_then(|data| data.get("text").and_then(|value| value.as_str()).map(str::to_string))
        .unwrap_or_default();

    let activity = if idle {
        String::new()
    } else {
        last_message(db, &id, "assistant")
            .map(|data| describe_activity(&data, now))
            .unwrap_or_default()
    };

    OpenCodeSession {
        id,
        task: text("title"),
        request,
        model: model_name(&text("model")),
        directory: text("directory"),
        updated_at,
        files_changed,
        activity,
        idle,
    }
}

/// Последнее сообщение сессии нужного типа: `data` уже разобранный JSON.
fn last_message(db: &Path, session_id: &str, kind: &str) -> Option<serde_json::Value> {
    let query = format!(
        "select data from session_message where session_id = '{session_id}' \
         and type = '{kind}' order by seq desc limit 1"
    );

    sqlite_json(db, &query)
        .and_then(|rows| rows.as_array().and_then(|rows| rows.first()).cloned())
        .and_then(|row| row.get("data").and_then(|value| value.as_str()).map(str::to_string))
        .and_then(|data| serde_json::from_str(&data).ok())
}

/// Что сессия делает прямо сейчас — по частям последнего ответа ассистента.
///
/// Активность живёт, пока сообщение свежее: если ассистент ответил давно,
/// честнее показать пустую строку, чем подвисшее «думает».
fn describe_activity(data: &serde_json::Value, now: i64) -> String {
    let created = data
        .get("time")
        .and_then(|time| time.get("created"))
        .and_then(|value| value.as_i64());

    // Сообщение старше `IDLE_AFTER_MS` — работа давно закончилась.
    if created.is_some_and(|at| now - at > IDLE_AFTER_MS) {
        return String::new();
    }

    let Some(parts) = data.get("content").and_then(|value| value.as_array()) else {
        return String::new();
    };

    // Идём с конца: важен последний шаг, а не начало ответа.
    for part in parts.iter().rev() {
        match part.get("type").and_then(|value| value.as_str()) {
            Some("tool") => {
                let running = part
                    .get("state")
                    .and_then(|state| state.get("status"))
                    .and_then(|value| value.as_str())
                    == Some("running");

                if running {
                    return tool_activity(part);
                }
            }
            Some("reasoning") => return String::from("думает"),
            Some("text") => return String::from("пишет ответ"),
            _ => {}
        }
    }

    String::new()
}

/// Инструмент под рукой: что именно OpenCode делает прямо сейчас.
fn tool_activity(part: &serde_json::Value) -> String {
    let name = part.get("name").and_then(|value| value.as_str()).unwrap_or("");
    let input = part.get("state").and_then(|state| state.get("input"));

    let field = |key: &str| {
        input
            .and_then(|input| input.get(key))
            .and_then(|value| value.as_str())
            .unwrap_or("")
    };

    match name {
        "edit" | "write" => match file_title(&field("path")) {
            Some(title) => format!("пишет в файл «{title}»"),
            None => String::from("правит файлы"),
        },
        "read" => match file_title(&field("path")) {
            Some(title) => format!("читает «{title}»"),
            None => String::from("читает файлы"),
        },
        "shell" => {
            // Показываем саму команду: по ней видно, чем занят агент.
            let command = field("command");
            let command = command.split(['\n', '|', ';']).next().unwrap_or("").trim();

            if command.is_empty() {
                String::from("запускает команду")
            } else {
                format!("запускает «{}»", truncate(command, 60))
            }
        }
        "grep" => format!("ищет «{}»", truncate(&field("pattern"), 40)),
        "glob" => format!("ищет файлы «{}»", truncate(&field("pattern"), 40)),
        "webfetch" => format!("открывает {}", truncate(&field("url"), 50)),
        "question" => String::from("спрашивает"),
        "task" => String::from("запускает подзадачу"),
        "" => String::from("работает"),
        other => format!("использует {other}"),
    }
}

/// Имя файла из пути: полный путь в карточку не влезает.
fn file_title(path: &str) -> Option<String> {
    let path = path.trim();

    if path.is_empty() {
        return None;
    }

    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .or_else(|| Some(path.to_string()))
}

fn truncate(text: &str, limit: usize) -> String {
    let text = text.trim();

    if text.chars().count() <= limit {
        return text.to_string();
    }

    let cut: String = text.chars().take(limit.saturating_sub(1)).collect();
    format!("{cut}…")
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
    url: String,
    duration_ms: Option<i64>,
    position_ms: Option<i64>,
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
    let url = entry("xesam:url")
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default();
    let duration_ms = entry("mpris:length").and_then(|value| value.as_i64()).map(micros_to_ms);
    let position_ms = mpris_property(bus_name, player_interface, "Position")
        .and_then(|value| value.as_i64())
        .map(micros_to_ms);
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
        url,
        duration_ms,
        position_ms,
        playing,
    }
}

/// Микросекунды MPRIS → миллисекунды. Время позиции нам нужно с точностью
/// до миллисекунды: по нему синхронизируются ожидаемые ноты эталонного трека.
fn micros_to_ms(micros: i64) -> i64 {
    micros / 1_000
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

    // Ожидаемые ноты есть только у эталонного трека: для чужого звука на
    // экране работает один живой детектор, совпадать не с чем.
    let duration_secs = player.duration_ms.map(|ms| ms as f64 / 1000.0);
    let expected_notes = if crate::hall::is_reference_track(
        &player.url,
        &player.title,
        duration_secs,
    ) {
        crate::hall::expected_note_labels(player.position_ms.unwrap_or(0) as f64 / 1000.0)
    } else {
        Vec::new()
    };

    MusicStatus {
        available: true,
        track: player.title,
        artist: player.artist,
        position_ms: player.position_ms,
        duration_ms: player.duration_ms,
        playing: player.playing,
        player: player.name,
        url: player.url,
        expected_notes,
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

/// Строка скорости моложе этого — загрузка идёт (Steam шлёт их раз в минуту).
const DOWNLOAD_FRESH: Duration = Duration::from_secs(120);
/// Байты, не растующие дольше этого, не считаем живой скоростью.
const DOWNLOAD_STALE: Duration = Duration::from_secs(20);

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

/// Последняя скорость из `content_log.txt`: `Current download rate: 60.386 Mbps`,
/// вместе с её возрастом. Steam шлёт эти строки примерно раз в минуту и только
/// пока действительно качает: после остановки они прекращаются. Свежая строка —
/// самый честный признак идущей загрузки, ведь `StateFlags` бывает 1026 и без
/// флага Running (проверено на живом клиенте).
struct DownloadRate {
    bytes_per_second: f64,
    age: Duration,
}

fn last_download_rate(root: &Path) -> Option<DownloadRate> {
    let log = read_tail(&root.join("logs/content_log.txt"), 256 * 1024)?;
    let now = now_ms();

    log.lines()
        .filter_map(|line| {
            let rest = line.split("Current download rate:").nth(1)?;
            let mbps: f64 = rest.split_whitespace().next()?.parse().ok()?;

            // Строка начинается с `[2026-10-23 00:59:24]` — возраст берём от него.
            let age = log_timestamp_ms(line)
                .map(|at| Duration::from_millis((now - at).max(0) as u64))
                .unwrap_or_default();

            Some(DownloadRate {
                bytes_per_second: mbps * 1_000_000.0 / 8.0,
                age,
            })
        })
        .next_back()
}

/// Отметка времени из начала строки лога: `[YYYY-MM-DD HH:MM:SS]`.
/// Время в логе местное, поэтому сразу приводим его к UTC — так его можно
/// сравнивать с `now_ms()`.
fn log_timestamp_ms(line: &str) -> Option<i64> {
    if !line.starts_with('[') {
        return None;
    }

    let stamp = line.get(1..20)?;
    let (date, time) = stamp.split_once(' ')?;
    let mut parts = date.split('-').map(str::parse::<i32>);
    let (year, month, day) = (
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
    );
    let mut clock = time.split(':').map(str::parse::<i64>);
    let (hour, minute, second) = (
        clock.next()?.ok()?,
        clock.next()?.ok()?,
        clock.next()?.ok()?,
    );

    let days = days_since_epoch(year, month, day);
    Some((days * 86_400 + hour * 3_600 + minute * 60 + second) * 1_000 - local_offset_ms())
}

/// Дней от 1970-01-01 до даты — нужен только для сравнения строк лога с «сейчас».
fn days_since_epoch(year: i32, month: i32, day: i32) -> i64 {
    // Перевод даты в юлианский день — стандартная формула без внешних крейтов.
    let year = year - i32::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_shift = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_shift + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era as i64 * 146_097 + day_of_era as i64 - 719_468
}

fn eta_seconds(remaining: u64, rate: Option<f64>) -> Option<u64> {
    let rate = rate.filter(|value| *value > 0.0)?;

    Some((remaining as f64 / rate).ceil() as u64)
}

/// Скорость по приросту скачанных байтов между опросами. Манифест Steam
/// обновляется реже, чем идёт загрузка, поэтому дельта не всегда есть; зато
/// когда байты растут, это прямое доказательство, что Steam качает.
fn measured_rate(cache: &mut Cache, app_id: &str, downloaded: u64, stale: Duration) -> Option<f64> {
    let previous = cache.progress.take();

    if let Some(previous) = previous {
        if previous.app_id != app_id {
            cache.rate_ema = None;
        } else {
            let elapsed = previous.at.elapsed();
            if downloaded > previous.bytes && elapsed.as_secs_f64() > 0.2 {
                let instant = (downloaded - previous.bytes) as f64 / elapsed.as_secs_f64();
                cache.rate_ema = Some(match cache.rate_ema {
                    Some(ema) => ema * 0.5 + instant * 0.5,
                    None => instant,
                });
                cache.sample_at = Some(Instant::now());
            } else if cache
                .sample_at
                .is_some_and(|at| at.elapsed() > stale)
            {
                // Байты встали: держать прошлую скорость нельзя, это вранье.
                cache.rate_ema = None;
            }
        }
    }

    cache.progress = Some(Progress {
        app_id: app_id.to_string(),
        bytes: downloaded,
        at: Instant::now(),
    });

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

    let report = last_download_rate(&root);
    // Свежая строка скорости (минута-две) — значит Steam действительно качает;
    // прирост байтов это подтверждает, когда манифест успел обновиться.
    let live = report
        .as_ref()
        .filter(|report| report.age < DOWNLOAD_FRESH)
        .map(|report| report.bytes_per_second);
    let rate = measured_rate(cache, &download.app_id, download.downloaded, DOWNLOAD_STALE).or(live);

    // Флаг Running Steam на этой машине не ставит даже во время загрузки,
    // поэтому состояние решает не он, а живая скорость.
    let state = if rate.is_some() {
        "загрузка"
    } else if download.state_flags & STATE_UPDATE_RUNNING != 0
        && download.state_flags & STATE_UPDATE_PAUSED == 0
    {
        "загрузка"
    } else if download.state_flags & STATE_UPDATE_QUEUED != 0 {
        "в очереди"
    } else {
        "пауза"
    };

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
/// это и есть «текущая работа» на машине. Сессии идут свежие сверху.
fn git_workdir(opencode: &OpenCodeStatus) -> PathBuf {
    if let Some(session) = opencode.sessions.first() {
        let session_dir = PathBuf::from(&session.directory);
        if !session.directory.is_empty() && session_dir.is_dir() {
            return session_dir;
        }
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
    fn parses_log_timestamp() {
        // Лог пишет местное время: в UTC строка обязана сдвинуться ровно
        // на смещение зоны машины.
        let stamp = log_timestamp_ms("[2026-10-23 00:59:24] Current download rate: 63.096 Mbps")
            .expect("отметка времени разбирается");

        assert_eq!(stamp, 1_792_717_164_000 - local_offset_ms());
        // Без отметки времени строку сравнивать с «сейчас» нельзя.
        assert_eq!(log_timestamp_ms("stats: (CDN, 33) 18 MB"), None);
        assert_eq!(log_timestamp_ms("[не дата] мусор"), None);
    }

    #[test]
    fn parses_timezone_offset() {
        // На этой машине зона задана, значит смещение — не ноль и кратно минутам.
        let offset = local_offset_ms();
        assert_eq!(offset % 60_000, 0, "смещение зоны не кратно минуте");
        assert!(offset.abs() <= 14 * 3_600_000, "смещение зоны вне разумных границ");
    }

    /// Свежая строка скорости означает идущую загрузку, старая — паузу.
    #[test]
    fn download_state_follows_rate_freshness() {
        let root = std::env::temp_dir().join("metricks-steam-test");
        let logs = root.join("logs");
        fs::create_dir_all(&logs).expect("каталог логов");
        let log = logs.join("content_log.txt");

        let local = |offset_seconds: i64| {
            let at = now_ms() + local_offset_ms() - offset_seconds * 1_000;
            let seconds = at.div_euclid(1_000);
            let days = seconds.div_euclid(86_400);
            let clock = seconds.rem_euclid(86_400);
            // Обратный перевод к календарной дате: дни от эпохи.
            let (year, month, day) = civil_from_days(days);
            format!(
                "[{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}] Current download rate: 60.0 Mbps",
                clock / 3_600,
                clock % 3_600 / 60,
                clock % 60,
            )
        };

        // Строка минуту назад: загрузка идёт.
        fs::write(&log, format!("{}\n", local(60))).expect("свежий лог");
        let fresh = last_download_rate(&root).expect("строка найдена");
        assert!(fresh.age < DOWNLOAD_FRESH, "свежая строка: {:?}", fresh.age);

        // Та же строка, но час назад: загрузка уже не идёт.
        fs::write(&log, format!("{}\n", local(3_600))).expect("старый лог");
        let stale = last_download_rate(&root).expect("строка найдена");
        assert!(stale.age > DOWNLOAD_FRESH, "старая строка: {:?}", stale.age);

        let _ = fs::remove_dir_all(&root);
    }

    /// Дни от эпохи обратно в календарную дату — нужен тесту выше.
    fn civil_from_days(days: i64) -> (i64, i64, i64) {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let day_of_era = z.rem_euclid(146_097);
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_shift = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_shift + 2) / 5 + 1;
        let month = month_shift + if month_shift < 10 { 3 } else { -9 };

        (year + i64::from(month <= 2), month, day)
    }

    /// Сообщение ассистента в том же виде, как его пишет OpenCode.
    fn assistant_message(parts: &str, created: i64) -> serde_json::Value {
        serde_json::from_str(&format!(
            "{{\"time\":{{\"created\":{created}}},\"content\":[{parts}]}}"
        ))
        .expect("разбор сообщения")
    }

    #[test]
    fn reads_running_tool_as_activity() {
        let now = 1_000_000;
        let edit = assistant_message(
            r#"{"type":"reasoning"},{"type":"tool","name":"edit","state":{"status":"running","input":{"path":"/home/fox/prog/metricks/src-tauri/src/status.rs"}}}"#,
            now,
        );

        assert_eq!(describe_activity(&edit, now), "пишет в файл «status.rs»");
    }

    #[test]
    fn ignores_finished_tools() {
        let now = 1_000_000;
        // Инструмент уже отработал, значит смотрим на предшествующее размышление.
        let finished = assistant_message(
            r#"{"type":"reasoning"},{"type":"tool","name":"shell","state":{"status":"completed","input":{"command":"ls"}}}"#,
            now,
        );

        assert_eq!(describe_activity(&finished, now), "думает");
    }

    #[test]
    fn names_the_shell_command() {
        let now = 1_000_000;
        let shell = assistant_message(
            r#"{"type":"tool","name":"shell","state":{"status":"running","input":{"command":"git status\ncargo test"}}}"#,
            now,
        );

        assert_eq!(describe_activity(&shell, now), "запускает «git status»");
    }

    #[test]
    fn stale_message_has_no_activity() {
        let now = 1_000_000;
        let old = assistant_message(r#"{"type":"reasoning"}"#, now - IDLE_AFTER_MS - 1);

        assert_eq!(describe_activity(&old, now), "");
    }

    #[test]
    fn plain_answer_reads_as_writing() {
        let now = 1_000_000;
        let answer = assistant_message(r#"{"type":"text","text":"готово"}"#, now);

        assert_eq!(describe_activity(&answer, now), "пишет ответ");
    }

    #[test]
    fn long_paths_keep_only_the_file_name() {
        assert_eq!(
            file_title("/home/fox/prog/metricks/src/index.css").as_deref(),
            Some("index.css")
        );
        assert_eq!(file_title("  ").as_deref(), None);
        assert_eq!(truncate("абвгде", 4), "абв…");
        assert_eq!(truncate("абв", 4), "абв");
    }

    #[test]
    fn falls_back_to_bus_name() {
        assert_eq!(
            fallback_player_name("org.mpris.MediaPlayer2.firefox.instance_1_51"),
            "Firefox"
        );
    }

    #[test]
    fn mpris_time_is_kept_in_milliseconds() {
        // Позиция берётся в миллисекундах: секундная точность дрожала бы
        // и ожидаемые ноты в конце трека не совпали бы с плеером.
        assert_eq!(micros_to_ms(12_713_000), 12_713);
        assert_eq!(micros_to_ms(0), 0);
        assert_eq!(micros_to_ms(142_709_000), 142_709);
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
