# Metricks

Desktop metrics dashboard built with [Tauri v2](https://tauri.app), React 19, TypeScript and Vite.

Dark UI with purple accents and rounded cards. All charts are hand-written SVG components — no third-party charting libraries.

## Requirements

| Component | Version |
| --- | --- |
| Node.js | 20.19+ or 22.12+ |
| Rust | 1.77.2+ |
| Linux | `webkit2gtk-4.1`, `libayatana-appindicator3`, `librsvg` |

On Arch/Omarchy:

```bash
sudo pacman -S --needed webkit2gtk-4.1 libayatana-appindicator3 librsvg xdo rust
```

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

The dashboard works out of the box: sample data is embedded into the bundle (`src/data/embedded.ts`), so no backend is required. It covers three sources — Telegram, OpenCode and Steam.

To read from a real API instead, enable the remote mode at build time:

```bash
VITE_USE_REMOTE=1 npm run build
```

The API base URL is resolved in the following order:

1. Value set via the settings button in the header (saved to `localStorage`)
2. `VITE_API_BASE` environment variable
3. `http://127.0.0.1:8787` (default)

Data is polled every 15 seconds. If a request fails, the last successful data remains on screen and an error state is shown in the status bar.

## API contract

All endpoints return JSON. The `range` parameter accepts `1h`, `24h`, `7d`, `30d`.

### `GET /api/health`

```json
{ "ok": true }
```

### `GET /api/summary`

Returns the user info, the status, the overview block of the large card and the source cards. See `src/api/types.ts` for the full schema.

### `GET /api/series?range=24h`

Returns the time series rendered by the area chart.

## Project structure

```text
src/            Frontend (React + TypeScript)
  api/          API types, client and config
  components/   UI components and SVG charts
  data/         Embedded dashboard data
  hooks/        Data fetching hooks
  lib/          Formatting, scales and utilities
src-tauri/      Tauri v2 Rust wrapper
```

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

Design tokens are defined in `src/theme.css`. Dark background `#0B0A12`, purple accents `#A855F7 / #8B5CF6 / #7C3AED`, border radii `--r-lg`, `--r-md`, `--r-pill`. Chart colors must come from CSS variables, not hardcoded values.
