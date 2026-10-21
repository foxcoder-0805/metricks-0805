import { getApiBase } from './config'
import type { Bars, Range, Series, Summary } from './types'

export class ApiError extends Error {
  readonly status: number

  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

async function request<T>(path: string, signal?: AbortSignal): Promise<T> {
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
  bars: (range: Range, signal?: AbortSignal) => request<Bars>(`/api/bars?range=${range}`, signal),
}
