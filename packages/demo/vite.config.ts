import { defineConfig } from 'vite';

export default defineConfig({
  clearScreen: false,
  server: {
    port: 5174,
    strictPort: true,
  },
  build: {
    target: 'esnext',
    minify: false,
    sourcemap: true,
  },
});
