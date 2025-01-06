// vite.config.ts
import {defineConfig} from 'vite'
import {resolve} from 'path'

export default defineConfig({
  server: {
    port: 3000,
    watch: {
      // Watch the wasm directory for changes
      ignored: ['!**/wasm/**']
    }
  },
  optimizeDeps: {
    exclude: ['aoc-2024-wasm']
  },
  build: {
    rollupOptions: {
      output: {
        assetFileNames: (assetInfo) => {
          // Keep WASM files in a separate directory
          if (assetInfo.name?.endsWith('.wasm')) {
            return 'assets/wasm/[name][extname]';
          }
          return 'assets/[name]-[hash][extname]';
        }
      }
    }
  }
})
