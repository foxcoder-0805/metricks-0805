import { embedded } from '../data/embedded'
import { getApiBase } from './config'
import type { Range, Series, Summary } from './types'

export class ApiError extends Error {
  readonly status: number

  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

const USE_EMBEDDED = import.meta.env.VITE_USE_REMOTE !== '1'

async function request<T>(path: string, signal?: AbortSignal): Promise<T> {
  if (USE_EMBEDDED) {
    if (path.startsWith('/api/health')) return embedded.health() as T
    if (path.startsWith('/api/summary')) return embedded.summary() as T
    if (path.startsWith('/api/series')) {
      const url = new URL(path, 'http://localhost')
      return embedded.series((url.searchParams.get('range') as Range) || '24h') as T
    }
    throw new ApiError('unknown endpoint', 0)
  }

  const base = getApiBase()
  let response: Response

  try {
    response = await fetch(`${base}${path}`, {
      signal,
      headers: { accept: 'application/json' },
    })
  } catch (error) {
    if (error instanceof DOMException && error.name === 'AbortError') throw error
    throw new ApiError(`Не удалось соединиться с ${base}`, 0)
  }

  if (!response.ok) {
    throw new ApiError(`API ${response.status}: ${response.statusText || 'ошибка запроса'}`, response.status)
  }

  try {
    return (await response.json()) as T
  } catch {
    throw new ApiError('API вернул невалидный JSON', response.status)
  }
}

export const api = {
  health: (signal?: AbortSignal) => request<{ ok: boolean }>('/api/health', signal),
  summary: (signal?: AbortSignal) => request<Summary>('/api/summary', signal),
  series: (range: Range, signal?: AbortSignal) => request<Series>(`/api/series?range=${range}`, signal),
}
