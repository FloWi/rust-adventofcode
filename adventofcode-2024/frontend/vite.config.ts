// vite.config.ts
import {defineConfig} from 'vite'
import {resolve} from 'path'

export default defineConfig({
  base: '/adventofcode-2024',
  server: {
    port: 3000,
    watch: {
      // Watch the wasm directory for changes
      ignored: ['!**/wasm/**']
    },
    // Ensure proper WASM mime types
    headers: {
      'Cache-Control': 'no-store',
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  optimizeDeps: {
    exclude: ['aoc-2024-wasm']
  },
  build: {
    // Specify output directory
    outDir: 'dist',

    // Generate manifest for asset handling
    manifest: true,

    // Configure asset handling
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
      },
      output: {
        // Ensure assets are placed in predictable locations
        assetFileNames: (assetInfo) => {
          if (assetInfo.name?.endsWith('.wasm')) {
            return 'assets/wasm/[name][extname]'
          }
          return 'assets/[name]-[hash][extname]'
        },
        // Configure chunk naming
        chunkFileNames: 'assets/[name]-[hash].js',
        // Configure entry point naming
        entryFileNames: 'assets/[name]-[hash].js',
      },
    },
  },
})
