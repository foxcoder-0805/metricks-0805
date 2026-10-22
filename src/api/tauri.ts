import { invoke } from '@tauri-apps/api/core'
import type { StatusSnapshot } from '../types'

/** Снимает срез реального состояния машины из Rust-слоя Tauri. */
export function fetchStatus(): Promise<StatusSnapshot> {
  return invoke<StatusSnapshot>('get_status')
}

/** В обычном браузере Tauri-команд нет: системных данных там не будет. */
export function hasTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}
