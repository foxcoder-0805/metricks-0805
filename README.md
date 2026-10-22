# Metricks

Desktop status dashboard built with [Tauri v2](https://tauri.app), React 19, TypeScript and Vite.

Dark UI with purple accents and rounded cards. The board shows three live status cards — what OpenCode is doing right now, what music is playing and how far a Steam download has gone — plus a large card with the details of the current OpenCode session.

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

The dashboard works out of the box: demo data is embedded into the bundle (`src/data/embedded.ts`), so no backend is required. It contains three status cards (OpenCode, music, Steam) and the current OpenCode session.

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

All endpoints return JSON.

### `GET /api/health`

```json
{ "ok": true }
```

### `GET /api/summary`

Returns the user info, the status, the status cards and the OpenCode session. See `src/api/types.ts` for the full schema.

## Project structure

```text
src/            Frontend (React + TypeScript)
  api/          API types, client and config
  components/   UI components
  data/         Embedded dashboard data
  hooks/        Data fetching hooks
  lib/          Formatting and utilities
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

Design tokens are defined in `src/theme.css`. Dark background `#0B0A12`, purple accents `#A855F7 / #8B5CF6 / #7C3AED`, border radii `--r-lg`, `--r-md`, `--r-pill`. Colors must come from CSS variables, not hardcoded values.

## Layout

- Top row: three status cards. Each one has a large keyword in the top left corner, a round cross in the top right corner and a caption in the bottom left corner.
- The cross in the top right corner hides the card. Hidden cards are remembered until the app is restarted, and the status bar offers a link to bring them back.
- Bottom row: one large card with the details of the current OpenCode session — the task title, its description, a progress bar and the list of the latest steps.
