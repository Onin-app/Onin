#!/usr/bin/env node

import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

import { scaffoldPlugin } from "./scaffold.js";
import type { CliOptions } from "./types.js";

const TEMPLATE_NAME = "svelte-view";
const SUPPORTED_TEMPLATES = [TEMPLATE_NAME] as const;
const CLI_DIR = dirname(fileURLToPath(import.meta.url));
const TEMPLATE_DIR = resolve(CLI_DIR, "../templates", TEMPLATE_NAME);

function parseArgs(argv: string[]): CliOptions {
  const options: CliOptions = {
    targetDir: undefined,
    pluginName: undefined,
    pluginId: undefined,
    withSettings: undefined,
    yes: false,
    template: TEMPLATE_NAME,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (!arg) {
      continue;
    }

    if (arg === "--") {
      continue;
    }

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

    if (arg === "--yes") {
      options.yes = true;
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

function printHelp(): void {
  console.log("create-onin-plugin");
  console.log("");
  console.log("Usage:");
  console.log("  create-onin-plugin [target-dir] [options]");
  console.log("");
  console.log("Options:");
  console.log("  --template <name>      Template to use (default: svelte-view)");
  console.log("  --plugin-name <name>   Plugin display name");
  console.log("  --plugin-id <id>       Plugin manifest id");
  console.log("  --with-settings        Include settings schema example");
  console.log("  --no-with-settings     Skip settings schema example");
  console.log("  --yes                  Use defaults for missing answers");
  console.log("  --help                 Show this help message");
}

function printNextSteps(targetDir: string): void {
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
  console.log("");
  console.log("To load the plugin in Onin:");
  console.log("  Open Settings -> Plugins -> Import Local Plugin");
}

async function main(): Promise<void> {
  const options = parseArgs(process.argv.slice(2));

  if (process.argv.includes("--help")) {
    printHelp();
    return;
  }

  if (!SUPPORTED_TEMPLATES.includes(options.template as (typeof SUPPORTED_TEMPLATES)[number])) {
    console.error(
      `Unsupported template: ${options.template}\nSupported templates: ${SUPPORTED_TEMPLATES.join(", ")}`,
    );
    process.exitCode = 1;
    return;
  }

  try {
    const result = await scaffoldPlugin(options, TEMPLATE_DIR);
    printNextSteps(result.targetDir);
  } catch (error: unknown) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}

main();
