#!/usr/bin/env node
/**
 * Mock API для локальной разработки.
 *
 * Отдаёт ту же схему, что и боевой API (src/api/types.ts), поэтому фронтенд
 * работает без изменений: npm run mock — и дашборд живой.
 *
 *   node mock-server/server.mjs [--port 8787]
 */

import { createServer } from 'node:http'

const args = process.argv.slice(2)
const portFlag = args.indexOf('--port')
const PORT = Number(process.env.PORT ?? (portFlag !== -1 ? args[portFlag + 1] : 8787))
const HOST = process.env.HOST ?? '127.0.0.1'

/* ─────────────── утилиты ─────────────── */

/** Детерминированный PRNG: одинаковый seed — одинаковая последовательность. */
function mulberry32(seed) {
  let a = seed >>> 0
  return () => {
    a = (a + 0x6d2b79f5) >>> 0
    let t = Math.imul(a ^ (a >>> 15), 1 | a)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

/** Внутри окна в 15 секунд значения не «прыгают», между окнами — меняются. */
function windowSeed() {
  return Math.floor(Date.now() / 15000)
}

const rand = mulberry32(windowSeed())
const jitter = (spread) => (rand() - 0.5) * 2 * spread

function pick(list) {
  return list[Math.floor(rand() * list.length) % list.length]
}

function round(value, digits = 0) {
  const factor = 10 ** digits
  return Math.round(value * factor) / factor
}

/** Плавная волна + шум: выглядит как настоящий график, а не как мусор. */
function wave(length, { base, amplitude, trend = 0, phase = 0, noise = 0 } = {}) {
  const points = []
  for (let i = 0; i < length; i += 1) {
    const t = i / Math.max(1, length - 1)
    const value =
      base +
      Math.sin(t * Math.PI * 2 + phase) * amplitude +
      Math.sin(t * Math.PI * 6 + phase * 1.7) * amplitude * 0.25 +
      trend * t * length +
      jitter(noise)
    points.push(round(Math.max(0, value), 2))
  }
  return points
}

function labelsFor(range, length) {
  const now = new Date()
  switch (range) {
    case '1h': {
      const out = []
      for (let i = length - 1; i >= 0; i -= 1) {
        const d = new Date(now.getTime() - i * 5 * 60_000)
        out.push(`${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`)
      }
      return out
    }
    case '7d':
    case '30d': {
      const out = []
      for (let i = length - 1; i >= 0; i -= 1) {
        const d = new Date(now.getTime() - i * 86_400_000)
        out.push(`${String(d.getDate()).padStart(2, '0')}.${String(d.getMonth() + 1).padStart(2, '0')}`)
      }
      return out
    }
    default: {
      const out = []
      for (let i = length - 1; i >= 0; i -= 1) {
        const d = new Date(now.getTime() - i * 3_600_000)
        out.push(`${String(d.getHours()).padStart(2, '0')}:00`)
      }
      return out
    }
  }
}

const RANGE_POINTS = { '1h': 12, '24h': 24, '7d': 7, '30d': 30 }

/* ─────────────── данные ─────────────── */

function buildSummary() {
  const requests = Math.round(118_000 + jitter(9_000))
  const errors = round(0.4 + rand() * 1.6, 2)
  const latency = round(78 + jitter(26), 1)
  const users = Math.round(3_240 + jitter(420))

  return {
    meta: {
      generatedAt: new Date().toISOString(),
      period: '24h',
    },
    user: {
      name: 'Elisey Lomtev',
      initials: 'EL',
      role: 'platform',
      status: 'online',
    },
    status: {
      label: errors > 1.4 ? 'Есть ошибки' : 'В норме',
      state: errors > 1.4 ? 'warn' : 'ok',
    },
    kpis: [
      {
        id: 'requests',
        label: 'Запросы',
        value: requests,
        unit: 'req/s',
        delta: round(4 + jitter(14), 1),
        format: 'number',
        icon: 'activity',
        spark: wave(24, { base: 90, amplitude: 26, trend: 1.4, noise: 8 }),
      },
      {
        id: 'errors',
        label: 'Доля ошибок',
        value: errors,
        unit: '%',
        delta: round(jitter(22), 1),
        format: 'percent',
        icon: 'alert',
        spark: wave(24, { base: 1.1, amplitude: 0.5, noise: 0.25 }),
        invertDelta: true,
      },
      {
        id: 'latency',
        label: 'Задержка p95',
        value: latency,
        unit: 'мс',
        delta: round(jitter(18), 1),
        format: 'duration',
        icon: 'clock',
        spark: wave(24, { base: 76, amplitude: 18, noise: 6 }),
        invertDelta: true,
      },
      {
        id: 'users',
        label: 'Активные сессии',
        value: users,
        unit: 'чел.',
        delta: round(2 + jitter(9), 1),
        format: 'number',
        icon: 'users',
        spark: wave(24, { base: 120, amplitude: 34, noise: 10 }),
      },
    ],
    gauges: [
      { id: 'cpu', label: 'Загрузка CPU', value: round(41 + jitter(26), 1), max: 100, unit: '%', caption: 'кластер api-01…04' },
      { id: 'memory', label: 'Память', value: round(62 + jitter(18), 1), max: 100, unit: '%', caption: 'node × 8' },
      { id: 'disk', label: 'Диск', value: round(34 + jitter(22), 1), max: 100, unit: '%', caption: 'ssd nvme' },
    ],
    donuts: [
      {
        id: 'sources',
        label: 'Источники трафика',
        items: [
          { label: 'Web', value: Math.round(38 + jitter(8)) },
          { label: 'Mobile', value: Math.round(27 + jitter(6)) },
          { label: 'API', value: Math.round(21 + jitter(5)) },
          { label: 'Прочее', value: Math.round(11 + jitter(4)) },
        ],
      },
    ],
    table: {
      id: 'regions',
      columns: ['Регион', 'Запросы', 'Ошибки', 'p95'],
      rows: [
        ['eu-central-1', Math.round(42_000 + jitter(4_000)), Math.round(rand() * 40), Math.round(72 + jitter(20))],
        ['eu-north-1', Math.round(31_500 + jitter(3_000)), Math.round(rand() * 40), Math.round(88 + jitter(20))],
        ['us-east-1', Math.round(26_800 + jitter(2_500)), Math.round(rand() * 40), Math.round(104 + jitter(24))],
        ['us-west-2', Math.round(19_200 + jitter(2_000)), Math.round(rand() * 40), Math.round(121 + jitter(24))],
        ['ap-south-1', Math.round(12_400 + jitter(1_500)), Math.round(rand() * 40), Math.round(146 + jitter(26))],
        ['sa-east-1', Math.round(7_900 + jitter(1_200)), Math.round(rand() * 40), Math.round(173 + jitter(30))],
      ],
    },
  }
}

function buildSeries(range) {
  const length = RANGE_POINTS[range] ?? 24
  return {
    x: labelsFor(range, length),
    series: [
      {
        id: 'api',
        label: 'API',
        points: wave(length, { base: 62, amplitude: 18, trend: 1.6, phase: 0.2, noise: 5 }),
      },
      {
        id: 'web',
        label: 'Web',
        points: wave(length, { base: 44, amplitude: 15, trend: 0.9, phase: 1.9, noise: 4 }),
      },
      {
        id: 'mobile',
        label: 'Mobile',
        points: wave(length, { base: 27, amplitude: 11, trend: 0.5, phase: 3.1, noise: 3 }),
      },
    ],
  }
}

function buildBars(range) {
  const length = range === '24h' ? 12 : RANGE_POINTS[range] ?? 12
  return {
    labels: labelsFor(range, length),
    series: [
      {
        id: 'reads',
        label: 'Чтения',
        values: wave(length, { base: 58, amplitude: 22, trend: 1.2, noise: 6 }),
      },
      {
        id: 'writes',
        label: 'Записи',
        values: wave(length, { base: 31, amplitude: 12, trend: 0.6, noise: 4 }),
      },
    ],
  }
}

/* ─────────────── сервер ─────────────── */

function send(res, status, body) {
  const payload = JSON.stringify(body)
  res.writeHead(status, {
    'content-type': 'application/json; charset=utf-8',
    'content-length': Buffer.byteLength(payload),
    'access-control-allow-origin': '*',
    'access-control-allow-headers': 'content-type',
    'access-control-allow-methods': 'GET,OPTIONS',
    'cache-control': 'no-store',
  })
  res.end(payload)
}

const server = createServer((req, res) => {
  if (req.method === 'OPTIONS') {
    res.writeHead(204, {
      'access-control-allow-origin': '*',
      'access-control-allow-headers': 'content-type',
      'access-control-allow-methods': 'GET,OPTIONS',
    })
    res.end()
    return
  }

  const url = new URL(req.url ?? '/', `http://${req.headers.host ?? 'localhost'}`)
  const range = url.searchParams.get('range') ?? '24h'

  switch (url.pathname) {
    case '/api/health':
      send(res, 200, { ok: true })
      return
    case '/api/summary':
      send(res, 200, buildSummary())
      return
    case '/api/series':
      send(res, 200, buildSeries(range))
      return
    case '/api/bars':
      send(res, 200, buildBars(range))
      return
    default:
      send(res, 404, { error: 'not found' })
  }
})

server.listen(PORT, HOST, () => {
  console.log(`mock API: http://${HOST}:${PORT}`)
  console.log('  /api/health  /api/summary  /api/series?range=24h  /api/bars?range=24h')
})
