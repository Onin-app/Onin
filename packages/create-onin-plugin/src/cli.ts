#!/usr/bin/env node

import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

import { scaffoldPlugin } from "./scaffold.js";
import type { CliOptions, Framework, Language } from "./types.js";

const DEFAULT_FRAMEWORK: Framework = "svelte";
const DEFAULT_LANGUAGE: Language = "ts";
const SUPPORTED_FRAMEWORKS: Framework[] = ["svelte", "react", "vue", "vanilla", "solid"];
const SUPPORTED_LANGUAGES: Language[] = ["ts", "js"];
const CLI_DIR = dirname(fileURLToPath(import.meta.url));
const BASE_TEMPLATE_DIRS: Record<Language, string> = {
  ts: resolve(CLI_DIR, "../templates/base/ts"),
  js: resolve(CLI_DIR, "../templates/base/js"),
};
const ADAPTER_TEMPLATE_DIRS: Record<Framework, Partial<Record<Language, string>>> = {
  svelte: {
    ts: resolve(CLI_DIR, "../templates/adapters/svelte/ts"),
  },
  react: {
    ts: resolve(CLI_DIR, "../templates/adapters/react/ts"),
    js: resolve(CLI_DIR, "../templates/adapters/react/js"),
  },
  vue: {
    ts: resolve(CLI_DIR, "../templates/adapters/vue/ts"),
  },
  vanilla: {
    ts: resolve(CLI_DIR, "../templates/adapters/vanilla/ts"),
    js: resolve(CLI_DIR, "../templates/adapters/vanilla/js"),
  },
  solid: {
    ts: resolve(CLI_DIR, "../templates/adapters/solid/ts"),
  },
};

function parseArgs(argv: string[]): CliOptions {
  const options: CliOptions = {
    targetDir: undefined,
    pluginName: undefined,
    pluginId: undefined,
    withSettings: undefined,
    yes: false,
    framework: DEFAULT_FRAMEWORK,
    language: DEFAULT_LANGUAGE,
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

    if (arg === "--framework") {
      const value = argv[i + 1];
      if (
        value === "svelte" ||
        value === "react" ||
        value === "vue" ||
        value === "vanilla" ||
        value === "solid"
      ) {
        options.framework = value;
      }
      i += 1;
      continue;
    }

    if (arg === "--language") {
      const value = argv[i + 1];
      if (value === "ts" || value === "js") {
        options.language = value;
      }
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

function findUnsupportedOption(argv: string[]): string | undefined {
  const knownFlags = new Set([
    "--",
    "--framework",
    "--plugin-name",
    "--plugin-id",
    "--language",
    "--yes",
    "--with-settings",
    "--no-with-settings",
    "--help",
  ]);

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (!arg || !arg.startsWith("--")) {
      continue;
    }

    if (!knownFlags.has(arg)) {
      return arg;
    }

    if (
      arg === "--framework" ||
      arg === "--language" ||
      arg === "--plugin-name" ||
      arg === "--plugin-id"
    ) {
      i += 1;
    }
  }

  return undefined;
}

function printHelp(): void {
  console.log("create-onin-plugin");
  console.log("");
  console.log("Usage:");
  console.log("  create-onin-plugin [target-dir] [options]");
  console.log("");
  console.log("Options:");
  console.log(`  --framework <name>     Framework to use (default: ${DEFAULT_FRAMEWORK})`);
  console.log(`  --language <name>      Language to use (default: ${DEFAULT_LANGUAGE})`);
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
  const argv = process.argv.slice(2);
  const unsupportedOption = findUnsupportedOption(argv);
  if (unsupportedOption) {
    console.error(`Unsupported option: ${unsupportedOption}`);
    process.exitCode = 1;
    return;
  }

  const options = parseArgs(argv);

  if (process.argv.includes("--help")) {
    printHelp();
    return;
  }

  if (!SUPPORTED_FRAMEWORKS.includes(options.framework)) {
    console.error(
      `Unsupported framework: ${options.framework}\nSupported frameworks: ${SUPPORTED_FRAMEWORKS.join(", ")}`,
    );
    process.exitCode = 1;
    return;
  }

  if (!SUPPORTED_LANGUAGES.includes(options.language)) {
    console.error(
      `Unsupported language: ${options.language}\nSupported languages: ${SUPPORTED_LANGUAGES.join(", ")}`,
    );
    process.exitCode = 1;
    return;
  }

  try {
    const result = await scaffoldPlugin(options, BASE_TEMPLATE_DIRS, ADAPTER_TEMPLATE_DIRS);
    printNextSteps(result.targetDir);
  } catch (error: unknown) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}

main();
