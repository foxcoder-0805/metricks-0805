# Metricks

Десктопный дашборд метрик. Тёмный фон, фиолетовые акценты, скруглённые
карточки. Графики — собственные SVG-компоненты, без внешних chart-библиотек.

Окно — [Tauri v2](https://tauri.app), интерфейс — React 19 + TypeScript + Vite.
Данные берутся из вашего REST API; рядом лежит mock-сервер с той же схемой,
так что дашборд запускается и работает без боевого бэкенда.

Макет — в `sample.png` (и его исходнике `sample.kra`): композиция и структура
карточек взяты оттуда, цвета и скругления — по требованиям задачи.

## Требования

| Компонент | Версия |
| --- | --- |
| Node.js | 20.19+ или 22.12+ |
| Rust | 1.77.2+ |
| Linux | `webkit2gtk-4.1`, `libayatana-appindicator3`, `librsvg` |

Arch/Omarchy:

```bash
sudo pacman -S --needed webkit2gtk-4.1 xdo appmenu-gtk-module rust
```

## Запуск

```bash
npm install

# в одном терминале — источник данных (можно не запускать, если есть свой API)
npm run mock

# в другом — приложение
npm run tauri dev
```

`tauri dev` сам поднимает Vite на `127.0.0.1:1420` и открывает окно.

Только фронтенд (в браузере, без Rust-обёртки):

```bash
npm run dev
```

## Настройка источника данных

Базовый URL API определяется так:

1. значение, сохранённое в UI (иконка шестерёнки в шапке) — в `localStorage`;
2. переменная окружения `VITE_API_BASE`;
3. дефолт `http://127.0.0.1:8787`.

```bash
VITE_API_BASE=https://metrics.example.com npm run dev
```

Данные обновляются каждые 15 секунд. При ошибке запроса последние успешные
значения остаются на экране, в подвале появляется пометка об ошибке.

## Контракт API

Все эндпоинты отдают JSON. `range` принимает `1h`, `24h`, `7d`, `30d`.

### `GET /api/health`

```json
{ "ok": true }
```

### `GET /api/summary`

```jsonc
{
  "meta": { "generatedAt": "2026-10-22T09:00:00.000Z", "period": "24h" },
  "user": { "name": "Elisey Lomtev", "initials": "EL", "role": "platform", "status": "online" },
  "status": { "label": "В норме", "state": "ok" },
  "kpis": [
    {
      "id": "requests",
      "label": "Запросы",
      "value": 118400,
      "unit": "req/s",
      "delta": 12.4,
      "format": "number",          // number | percent | bytes | duration
      "icon": "activity",          // имя из src/components/Icons.tsx
      "spark": [87.4, 95.5, 117.8],
      "invertDelta": false         // true: рост показателя считается ухудшением
    }
  ],
  "gauges": [{ "id": "cpu", "label": "Загрузка CPU", "value": 41.2, "max": 100, "unit": "%", "caption": "кластер api-01…04" }],
  "donuts": [{ "id": "sources", "label": "Источники трафика", "items": [{ "label": "Web", "value": 38 }] }],
  "table": {
    "id": "regions",
    "columns": ["Регион", "Запросы", "Ошибки", "p95"],
    "rows": [["eu-central-1", 42000, 12, 72]]
  }
}
```

### `GET /api/series?range=24h`

```json
{
  "x": ["01:00", "02:00"],
  "series": [{ "id": "api", "label": "API", "points": [63.6, 76.7] }]
}
```

### `GET /api/bars?range=24h`

```json
{
  "labels": ["01:00", "02:00"],
  "series": [{ "id": "reads", "label": "Чтения", "values": [58.8, 77.7] }]
}
```

Типы описаны в `src/api/types.ts` — это единственный источник истины для
фронтенда и mock-сервера.

## Mock-сервер

```bash
npm run mock              # http://127.0.0.1:8787
node mock-server/server.mjs --port 9000
```

Данные генерируются псевдослучайно, но стабильно в пределах 15-секундного
окна — поэтому значения не «прыгают» при каждом запросе.

## Структура

```
src/            фронтенд
  api/          типы контракта, клиент, конфиг endpoint'а
  components/   карточки и SVG-графики
  hooks/        опрос API
  lib/          форматирование, шкалы, утилиты
mock-server/    node:http сервер с той же схемой, что и API
src-tauri/      Rust-обёртка Tauri
```

## Сборка

```bash
npm run tauri build
```

Сборщик отключён (`bundle.active: false`) — поставляются только исходники и
бинарник, готовые установщики не собираются.

## Дизайн

Токены — в `src/theme.css`: тёмный фон `#0B0A12`, акценты
`#A855F7 / #8B5CF6 / #7C3AED`, скругления `--r-lg` / `--r-md` / `--r-pill`.
Цвета в графиках берутся только из CSS-переменных, хардкод запрещён.
