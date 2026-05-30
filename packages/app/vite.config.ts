import { resolve } from "node:path";
import { readFileSync } from "node:fs";
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;
const packageJson = JSON.parse(
  readFileSync(resolve(__dirname, "package.json"), "utf-8"),
) as { version?: string };

// https://vitejs.dev/config/
export default defineConfig({
  envDir: resolve(__dirname, "../.."),
  plugins: [sveltekit(), tailwindcss()],
  define: {
    "import.meta.env.PACKAGE_VERSION": JSON.stringify(packageJson.version),
  },
  build: {
    sourcemap: true,
  },
  resolve: {
    alias: [
      // TODO: @incremark/svelte@1.0.2 has a packaging bug that preserves raw `.svelte.ts` extensions in compiled ESM output.
      // Once the upstream package is updated to fix this issue (or our PR is merged), this alias workaround can be safely removed.
      {
        find: /^(.*)\.svelte\.ts$/,
        replacement: "$1.svelte.js",
      },
    ],
  },
  optimizeDeps: {
    exclude: ["@incremark/svelte"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
});
