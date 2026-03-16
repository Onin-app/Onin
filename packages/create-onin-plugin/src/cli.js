#!/usr/bin/env node

import { stdin as input, stdout as output } from "node:process";
import { createInterface } from "node:readline/promises";
import { fileURLToPath } from "node:url";
import { basename, dirname, join, resolve } from "node:path";
import { copyFile, mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";

const TEMPLATE_NAME = "svelte-view";
const CLI_DIR = dirname(fileURLToPath(import.meta.url));
const TEMPLATE_DIR = resolve(
  CLI_DIR,
  "../templates",
  TEMPLATE_NAME,
);

function parseArgs(argv) {
  const options = {
    targetDir: undefined,
    pluginName: undefined,
    pluginId: undefined,
    withSettings: undefined,
    template: TEMPLATE_NAME,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];

    if (!arg.startsWith("--") && !options.targetDir) {
      options.targetDir = arg;
      continue;
    }

    if (arg === "--template") {
      options.template = argv[i + 1] ?? TEMPLATE_NAME;
      i += 1;
      continue;
    }

    if (arg === "--plugin-name") {
      options.pluginName = argv[i + 1];
      i += 1;
      continue;
    }

    if (arg === "--plugin-id") {
      options.pluginId = argv[i + 1];
      i += 1;
      continue;
    }

    if (arg === "--with-settings") {
      options.withSettings = true;
      continue;
    }

    if (arg === "--no-with-settings") {
      options.withSettings = false;
      continue;
    }
  }

  return options;
}

function toTitleCase(value) {
  return value
    .split(/[-_.\s]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function slugify(value) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9.-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .replace(/-{2,}/g, "-");
}

function isValidPluginId(value) {
  return /^[a-z0-9][a-z0-9.-]*[a-z0-9]$/.test(value) && !value.includes("..");
}

function isValidPackageName(value) {
  return /^[a-z0-9][a-z0-9.-]*[a-z0-9]$/.test(value);
}

async function isDirectoryEmpty(dir) {
  const entries = await readdir(dir);
  return entries.length === 0;
}

async function ensureTargetDirectory(targetDir) {
  try {
    const targetStat = await stat(targetDir);
    if (!targetStat.isDirectory()) {
      throw new Error(`Target path exists and is not a directory: ${targetDir}`);
    }

    if (!(await isDirectoryEmpty(targetDir))) {
      throw new Error(`Target directory is not empty: ${targetDir}`);
    }
  } catch (error) {
    if (error && error.code === "ENOENT") {
      await mkdir(targetDir, { recursive: true });
      return;
    }

    throw error;
  }
}

function renderTemplate(content, context) {
  return content
    .replaceAll("__PLUGIN_NAME__", context.pluginName)
    .replaceAll("__PLUGIN_ID__", context.pluginId)
    .replaceAll("__PACKAGE_NAME__", context.packageName)
    .replaceAll("__PLUGIN_DESCRIPTION__", context.pluginDescription)
    .replaceAll("__SETTINGS_BLOCK__", context.settingsBlock)
    .replaceAll("__SETTINGS_IMPORT__", context.settingsImport)
    .replaceAll("__SETTINGS_NOTE__", context.settingsNote)
    .replaceAll("__KEYWORD__", context.keyword);
}

async function copyTemplateDir(sourceDir, targetDir, context) {
  const entries = await readdir(sourceDir, { withFileTypes: true });

  for (const entry of entries) {
    const sourcePath = join(sourceDir, entry.name);
    const outputName = entry.name.endsWith(".tpl")
      ? entry.name.slice(0, -4)
      : entry.name;
    const targetPath = join(targetDir, outputName);

    if (entry.isDirectory()) {
      await mkdir(targetPath, { recursive: true });
      await copyTemplateDir(sourcePath, targetPath, context);
      continue;
    }

    if (entry.name.endsWith(".tpl")) {
      const content = await readFile(sourcePath, "utf8");
      await writeFile(targetPath, renderTemplate(content, context), "utf8");
      continue;
    }

    await copyFile(sourcePath, targetPath);
  }
}

function buildSettingsBlock(withSettings) {
  if (!withSettings) {
    return "  // Add settings.useSettingsSchema(...) here when your plugin needs configurable options.\n";
  }

  return `  await settings.useSettingsSchema([
    {
      key: "accentColor",
      label: "Accent Color",
      type: "color",
      defaultValue: "#111827",
      description: "Example plugin setting registered during lifecycle onLoad.",
    },
  ]);
`;
}

async function promptForMissingOptions(initialOptions) {
  const rl = createInterface({ input, output });

  try {
    const targetDirInput =
      initialOptions.targetDir ||
      (await rl.question("Project directory name: ")).trim();
    const targetDir = targetDirInput || "my-onin-plugin";
    const packageName = slugify(basename(targetDir));

    const pluginNameInput =
      initialOptions.pluginName ||
      (await rl.question(
        `Plugin name (${toTitleCase(packageName) || "My Onin Plugin"}): `,
      )).trim();
    const pluginName = pluginNameInput || toTitleCase(packageName) || "My Onin Plugin";

    const defaultPluginId = `com.example.${packageName || "my-onin-plugin"}`;
    const pluginIdInput =
      initialOptions.pluginId ||
      (await rl.question(`Plugin ID (${defaultPluginId}): `)).trim();
    const pluginId = pluginIdInput || defaultPluginId;

    let withSettings = initialOptions.withSettings;
    if (withSettings === undefined) {
      const answer = (
        await rl.question("Include settings schema example? (Y/n): ")
      ).trim().toLowerCase();
      withSettings = answer !== "n";
    }

    return {
      targetDir,
      pluginName,
      pluginId,
      withSettings,
    };
  } finally {
    rl.close();
  }
}

function printNextSteps(targetDir) {
  console.log("");
  console.log("Project created.");
  console.log("");
  console.log(`  cd ${targetDir}`);
  console.log("  pnpm install");
  console.log("  pnpm dev");
  console.log("");
  console.log("To build release artifacts:");
  console.log("  pnpm build");
  console.log("  pnpm pack:plugin");
}

async function main() {
  const options = parseArgs(process.argv.slice(2));

  if (options.template !== TEMPLATE_NAME) {
    console.error(`Unsupported template: ${options.template}`);
    process.exitCode = 1;
    return;
  }

  const answers = await promptForMissingOptions(options);
  const targetDir = resolve(process.cwd(), answers.targetDir);
  const packageName = slugify(basename(targetDir));

  if (!isValidPackageName(packageName)) {
    console.error(`Invalid project directory name: ${packageName}`);
    process.exitCode = 1;
    return;
  }

  if (!isValidPluginId(answers.pluginId)) {
    console.error(
      "Invalid plugin ID. Use lowercase letters, numbers, dots, and hyphens only.",
    );
    process.exitCode = 1;
    return;
  }

  await ensureTargetDirectory(targetDir);

  const context = {
    packageName,
    pluginName: answers.pluginName,
    pluginId: answers.pluginId,
    pluginDescription: `${answers.pluginName} plugin for Onin`,
    keyword: packageName.split(".").pop() || packageName,
    settingsImport: answers.withSettings ? ", settings" : "",
    settingsBlock: buildSettingsBlock(answers.withSettings),
    settingsNote: answers.withSettings
      ? "This template includes a sample settings schema registered from lifecycle.ts."
      : "This template omits settings schema. Add it later in src/lifecycle.ts if needed.",
  };

  await copyTemplateDir(TEMPLATE_DIR, targetDir, context);
  printNextSteps(answers.targetDir);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
