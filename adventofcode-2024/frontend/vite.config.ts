// vite.config.ts
import {defineConfig} from 'vite'
import {resolve} from 'path'

export default defineConfig({
  server: {
    port: 3000,
    watch: {
      // Watch the pkg directory for changes
      ignored: ['!**/public/pkg/**']
    }
  },
  optimizeDeps: {
    exclude: ['aoc-2024-wasm']
  },
  resolve: {
    alias: {
      'aoc-2024-wasm': resolve(__dirname, 'public/pkg')
    }
  }
})
