# WARNING MADE WITH AI!!!

# Metricks

Desktop status dashboard built with [Tauri v2](https://tauri.app), React 19, TypeScript and Vite.

Dark UI with purple accents and rounded cards. The header carries a pill with the login of the current session. When something plays, a cava-style spectrum bar fills the room left between the cards and the status bar, and above the bars runs a trail of the recognised notes. The board draws one card per open OpenCode session — each shows what that session is doing right now — next to what music is playing and how far a Steam download has gone, plus a tile that adds more cards. Every metric in the catalog is read from the machine (memory, disk, the focused window, git, the network), so there are no placeholder captions. Card captions come from the machine itself, not from a backend.

## Requirements

| Component | Version |
| --- | --- |
| Node.js | 20.19+ or 22.12+ |
| Rust | 1.77.2+ |
| Linux | `webkit2gtk-4.1`, `libayatana-appindicator3`, `librsvg` |
| Runtime tools | `sqlite3`, `busctl`, `df`, `git`; optional `hyprctl` for the focused window, `nmcli` for the network and `parec` for the audio spectrum |

On Arch/Omarchy:

```bash
sudo pacman -S --needed webkit2gtk-4.1 libayatana-appindicator3 librsvg xdo rust sqlite systemd git coreutils networkmanager
```

`sqlite3` reads the OpenCode database and `busctl` talks to MPRIS players; both ship with the packages above. Only the collectors whose tools and files are present produce data: a missing tool leaves the matching card on "нет данных" instead of inventing values.

## Getting started

Install dependencies:

```bash
npm install
```

Run the desktop app:

```bash
npm run tauri dev
```

`tauri dev` starts Vite on `http://127.0.0.1:1420` and opens the Tauri window automatically.

Run frontend only in the browser:

```bash
npm run dev
```

## Data

The dashboard works fully offline: there is no backend and no network calls. The Rust layer collects machine state and the frontend polls it through the `get_status` Tauri command every second. The git repository is re-read at most every 5 seconds, so a one-second poll does not run `git status` on every tick.

| Card | Source |
| --- | --- |
| OpenCode | `~/.local/share/opencode/opencode.db` — every session touched in the last day, newest first: title, last request, model, and what the session is doing right now (a running tool, thinking, or waiting) read from the last assistant message |
| Music | MPRIS over the D-Bus session bus — the first playing player, its track, artist and position |
| Steam | Steam process, `steamapps/appmanifest_<id>.acf` (all libraries from `libraryfolders.vdf`) for the game, downloaded bytes and total size, plus `logs/content_log.txt` for the download rate — percent, speed in MB/s and time left are computed from them |
| System | `/proc/meminfo` (used memory), `/proc/loadavg`, `/proc/uptime` and `df -B1 /` (root disk usage) |
| Window | `hyprctl activewindow -j` — the class, title and workspace of the focused window |
| Git | `git -C <session directory> rev-parse` and `status --porcelain` — the branch and the number of changed files in the OpenCode project |
| Network | `nmcli device` — the active connection; `nmcli device wifi` adds the SSID and signal level for Wi-Fi |
| User | `USER`/`LOGNAME` from the environment, falling back to `whoami`; `HOME` for the home directory |
| Audio spectrum | `parec` on `@DEFAULT_MONITOR@` — the system output is captured into 8192-sample windows (read in 1024-sample steps so the bar keeps its frame rate), windowed with a Hann function and turned into 73 bars by a small radix-2 FFT in `src-tauri/src/audio.rs`: the grid is C1…C7 in semitones, so one bar is one note and the detected note's own bar is lit |
| Notes | the same capture, but over a 2048-sample window: a YIN pitch detector finds the fundamental and names the note (A4 = 440); a trail keeps the last 14 notes |

The Steam state does not come from `StateFlags`: on this machine the flag stays at 1026 without the Running bit even while Steam downloads. The card instead trusts the freshness of the `Current download rate` line in `content_log.txt` (Steam stops writing them when the download stops) and real byte growth, compared in the machine's own timezone — the log writes local time, not UTC.

The spectrum is optional as well: without `parec` or any output the bar simply stays hidden. Notes appear only for a confident, in-tune pitch: chords and noise leave a dash instead of a guess.

Everything degrades honestly: without a session the card says "сессий нет", without a player — "ничего не играет", without Steam — "не установлен", without a Hyprland window — "нет активного окна", outside a repository — "не git-репозиторий", without a connection — "нет подключения".

Other files:

- `src/data/dashboard.ts` — the two built-in cards. OpenCode is not here: the board draws one card per open session.
- `src/data/apps.ts` — the catalog of applications and metrics. Every metric is bound to a live source; nothing is a static caption.
- `src/data/live.ts` — turns a status snapshot into card captions.

Cards added from the catalog live in memory for the current session only.

The collected snapshot can be inspected without the window:

```bash
cargo test -- --ignored --nocapture live_snapshot_dump
```

The audio bar and the pitch detector can be dumped the same way; the second one runs a track through `ffmpeg` and prints the notes it hears:

```bash
cargo test -- --ignored --nocapture live_audio_dump
METRICKS_TRACK=~/Music/track.mp3 cargo test -- --ignored --nocapture track_note_dump
```

## Project structure

```text
src/            Frontend (React + TypeScript)
  api/          Tauri command wrapper
  components/   UI components
  data/         Card catalog and snapshot-to-caption mapping
  hooks/        Status polling
  types.ts      Shared types
src-tauri/      Tauri v2 Rust wrapper
  src/status.rs Machine state collectors
  src/audio.rs  System audio capture and FFT for the spectrum bar
```

In a plain browser (`npm run dev`) there is no Tauri runtime, so the cards show "нет данных" and the status bar reports that machine access is unavailable.

## Build

Frontend bundle:

```bash
npm run build
```

Desktop binary (the Cargo workspace lives in the repository root):

```bash
cargo build
```

The binary is placed in `target/debug/metricks`. This repo ships source only, bundling is disabled via `bundle.active: false` in `src-tauri/tauri.conf.json`.

## Design

Design tokens are defined in `src/theme.css`. Dark background `#0B0A12`, purple accents `#A855F7 / #8B5CF6 / #7C3AED`, border radii `--r-lg`, `--r-md`, `--r-pill`. Colors must come from CSS variables, not hardcoded values.

## Layout

- Top row: the brand and a pill with the login of the current session on the right.
- Cards: one per open OpenCode session, plus music, Steam, and any cards you add. Each one has a large keyword in the top left corner, a round cross in the top right corner and a caption in the bottom left corner. Captions refresh every second.
- The cross in the top right corner hides the card. Hidden cards are remembered until the app is restarted, and the status bar offers a link to bring them back.
- Bottom of the window: a cava-style spectrum bar of the system sound, stretched to the free height between the cards and the status bar. It is hidden while nothing plays and fades in as soon as audio is heard; a trail of recognised notes runs above the bars, and the bar of the note being heard is highlighted.
- Last tile: a `+` tile. Clicking it opens a dialog where you pick an application and what to track for it — the new card is added with the caption of the selected metric. Every metric is live and marked "с машины" in the dialog.
