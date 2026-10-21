# Metricks

Desktop metrics dashboard built with [Tauri v2](https://tauri.app), React 19, TypeScript and Vite.

Dark UI with purple accents, rounded cards. All charts are hand-written SVG components with no third-party charting libraries.

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

Start the mock API (optional if you have your own backend):

```bash
npm run mock
```

Run the desktop app:

```bash
npm run tauri dev
```

`tauri dev` will start Vite on `http://127.0.0.1:1420` and open the Tauri window automatically.

Run frontend only in the browser:

```bash
npm run dev
```

## API configuration

The API base URL is resolved in the following order:

1. Value set via the settings button in the header (saved to `localStorage`)
2. `VITE_API_BASE` environment variable
3. `http://127.0.0.1:8787` (default)

Example:

```bash
VITE_API_BASE=https://metrics.example.com npm run dev
```

Data is polled every 15 seconds. If a request fails, the last successful data remains on screen and an error state is shown in the status bar.

## API contract

All endpoints return JSON. The `range` parameter accepts `1h`, `24h`, `7d`, `30d`.

### `GET /api/health`

```json
{ "ok": true }
```

### `GET /api/summary`

Returns user info, status, KPI cards, gauges, donuts, and table data. See `src/api/types.ts` for the full schema.

### `GET /api/series?range=24h`

Returns time series for area charts.

### `GET /api/bars?range=24h`

Returns labeled values for bar charts.

The mock server in `mock-server/server.mjs` implements the same schema, so the app works out of the box without a real backend.

## Project structure

```text
src/            Frontend (React + TypeScript)
  api/          API types, client and config
  components/   UI components and SVG charts
  hooks/        Data fetching hooks
  lib/          Formatting, scales and utilities
mock-server/    Zero-dependency mock API
src-tauri/      Tauri v2 Rust wrapper
```

## Build

```bash
npm run tauri build
```

This repo ships source only. Bundling is disabled via `bundle.active: false` in `src-tauri/tauri.conf.json`.

## Design

Design tokens are defined in `src/theme.css`. Dark background `#0B0A12`, purple accents `#A855F7 / #8B5CF6 / #7C3AED`, border radii `--r-lg`, `--r-md`, `--r-pill`. Chart colors must come from CSS variables, not hardcoded values.
