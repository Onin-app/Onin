import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  server: {
    port: 5174,
    strictPort: true,
    cors: true, // 允许跨域访问
  },
  resolve: {
    alias: {
      // Resolve SDK from source for development
      sdk: path.resolve(__dirname, '../sdk/src/index.ts'),
    },
  },
  build: {
    target: 'esnext',
    minify: false,
    sourcemap: true,
  },
});
