import { defineConfig } from 'vite';
import { resolve } from 'path';

// Lifecycle 单独打包配置
// 使用方法: vite build --config vite.lifecycle.config.ts
export default defineConfig({
  build: {
    outDir: 'dist',
    emptyOutDir: false, // 不清空，允许与主应用一起构建
    lib: {
      entry: resolve(__dirname, 'src/lifecycle.ts'),
      formats: ['es'], // ES 模块格式
      fileName: () => 'lifecycle.js',
    },
    rollupOptions: {
      // 不要外部化任何依赖，全部打包进去
      external: [],
      output: {
        // 不分割代码，全部内联到一个文件
        inlineDynamicImports: true,
      },
    },
    target: 'es2020',
    minify: 'terser',
  },
});
