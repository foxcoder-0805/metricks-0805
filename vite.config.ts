import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// Tauri открывает окно на devUrl, поэтому порт должен быть фиксированным:
// иначе vite сдвинет его на 1421 и окно останется белым.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
    // rust-исходники меняются часто, а watch за ними только тормозит
    watch: { ignored: ['**/src-tauri/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_'],
})
