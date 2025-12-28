import { defineConfig } from 'vite';
import path from 'path';
import dts from 'vite-plugin-dts';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [dts({
    insertTypesEntry: true,
  })],
  build: {
    lib: {
      // Could also be a dictionary or array of multiple entry points
      entry: path.resolve(__dirname, 'src/index.ts'),
      name: 'baize',
      // the proper extensions will be added
      fileName: 'index',
      formats: ['es', 'umd'],
    },
    rollupOptions: {
      // 确保外部化处理那些你不想打包进库的依赖
      external: ['@tauri-apps/api/core'],
      output: {
        // 在 UMD 构建模式下为这些外部化的依赖提供一个全局变量
        exports: 'named',
        globals: {
          '@tauri-apps/api/core': 'TauriApi',
        },
      },
    },
  },
});