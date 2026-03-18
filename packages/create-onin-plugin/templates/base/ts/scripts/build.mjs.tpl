import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { build, loadConfigFromFile } from "vite";

const root = process.cwd();
const configFile = [
  "vite.config.ts",
  "vite.config.js",
  "vite.config.mts",
  "vite.config.mjs",
]
  .map((file) => resolve(root, file))
  .find((file) => existsSync(file));

const loaded = configFile
  ? await loadConfigFromFile({ command: "build", mode: process.env.MODE ?? "production" }, configFile)
  : null;

const sharedConfig = loaded?.config ?? {};
const outDir = sharedConfig.build?.outDir ?? "dist";

await build(sharedConfig);

await build({
  ...sharedConfig,
  configFile: false,
  build: {
    ...sharedConfig.build,
    outDir,
    emptyOutDir: false,
    lib: {
      entry: resolve(root, "src/background.__SCRIPT_EXT__"),
      formats: ["es"],
      fileName: () => "lifecycle.js",
    },
    rollupOptions: {
      ...sharedConfig.build?.rollupOptions,
      output: {
        ...sharedConfig.build?.rollupOptions?.output,
        inlineDynamicImports: true,
      },
    },
  },
});
