# Metricks — правила проекта

Десктопный дашборд метрик. Tauri v2 (Rust) + React 18 + TypeScript + Vite.
Графики — собственные SVG-компоненты, без chart-библиотек.

## Git

Правила — в `~/.config/opencode/AGENTS.md`, здесь они действуют для этого репозитория:

- ветка `feature/<slug>` → коммиты по шагам → merge в `main`;
- Conventional Commits;
- push в `origin` после каждого коммита;
- `main` не переписывается, force-push запрещён.

Ветка по умолчанию — `main` (совпадает с дефолтом GitHub).

## Структура

```
src/            фронтенд (React + TS)
src/api/        типы, клиент, конфиг endpoint'а
src/components/ UI и SVG-графики
mock-server/    node:http сервер с той же схемой, что и API
src-tauri/      Rust-обёртка Tauri
```

## Контракт API

`BASE_URL` из `VITE_API_BASE` (дефолт `http://127.0.0.1:8787`), переопределяется
в UI и хранится в localStorage. Эндпоинты: `/api/health`, `/api/summary`,
`/api/series?range=24h`, `/api/bars?range=24h`.

## Дизайн

Тёмный фон `#0B0A12`, фиолетовые акценты `#A855F7 / #8B5CF6 / #7C3AED`,
всё остальное — фиолетовое. Скругления обязательны: карточки `--r-lg`,
контролы `--r-md`, пилюли `--r-pill`. Токены — в `src/theme.css`, цвета в
графиках брать из CSS-переменных, а не хардкодить.
