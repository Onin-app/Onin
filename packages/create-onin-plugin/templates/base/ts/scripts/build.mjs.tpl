import { existsSync } from "node:fs";
import { rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
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
const backgroundEntrySource = resolve(root, ".onin-plugin-background-entry.__SCRIPT_EXT__");
const pluginEntryUrl = pathToFileURL(resolve(root, "src/plugin.__SCRIPT_EXT__")).href;

await build(sharedConfig);

try {
  await writeFile(
    backgroundEntrySource,
    `import { setupPlugin } from "onin-sdk";
import plugin from ${JSON.stringify(pluginEntryUrl)};

setupPlugin(plugin);
`,
    "utf8",
  );

  await build({
    ...sharedConfig,
    configFile: false,
    build: {
      ...sharedConfig.build,
      outDir,
      emptyOutDir: false,
      lib: {
        entry: backgroundEntrySource,
        formats: ["es"],
        fileName: () => "__BACKGROUND_ENTRY_FILE__",
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
} finally {
  await rm(backgroundEntrySource, { force: true });
}
