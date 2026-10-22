# Metricks

Desktop status dashboard built with [Tauri v2](https://tauri.app), React 19, TypeScript and Vite.

Dark UI with purple accents and rounded cards. The board shows three status cards — what OpenCode is doing right now, what music is playing and how far a Steam download has gone — plus a tile that adds your own cards on top of them.

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

The dashboard works fully offline: there is no backend and no polling. All data is static and ships with the bundle.

- `src/data/dashboard.ts` — the user shown in the header and the initial status cards.
- `src/data/apps.ts` — the catalog of applications and the metrics that can be tracked for each of them.

Cards added from the catalog live in memory for the current session only.

## Project structure

```text
src/            Frontend (React + TypeScript)
  components/   UI components
  data/         Static dashboard data and the application catalog
  types.ts      Shared types
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
- Second row: a `+` tile in place of the fourth card. Clicking it opens a dialog where you pick an application and what to track for it — the new card is added with the caption of the selected metric.
