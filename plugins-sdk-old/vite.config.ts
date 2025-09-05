import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'UnifiedPluginSDK',
      fileName: (format) => `unified-plugin-sdk.${format}.js`,
      formats: ['es', 'umd']
    },
    rollupOptions: {
      external: ['@tauri-apps/api/core'],
      output: {
        globals: {
          '@tauri-apps/api/core': 'TauriCore'
        }
      }
    }
  },
  test: {
    globals: true,
    environment: 'jsdom'
  }
});