import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { resolve } from "path";

export default defineConfig({
  plugins: [
    svelte({
      compilerOptions: {
        // 不生成 CSS 文件，全部内联到 JS 中
        css: "injected",
      },
    }),
  ],
  build: {
    lib: {
      entry: resolve(__dirname, "src/main.ts"),
      name: "OninInject",
      formats: ["iife"],
      fileName: () => "onin-inject.js",
    },
    outDir: resolve(__dirname, "../app/src-tauri/templates"),
    emptyOutDir: false, // 不清空目录，保留其他模板文件（如 plugin-window-fab.js）
    minify: "oxc",
    rollupOptions: {
      output: {
        // 确保单文件输出
        inlineDynamicImports: true,
      },
    },
  },
});
