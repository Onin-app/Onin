import { stdin as input, stdout as output } from "node:process";
import { basename } from "node:path";
import { createInterface } from "node:readline/promises";

import type { Answers, CliOptions, Framework, Language } from "./types.js";
import { slugify, toTitleCase } from "./validators.js";

const FRAMEWORKS: Framework[] = ["svelte", "react", "vue", "vanilla", "solid"];
const DEFAULT_FRAMEWORK: Framework = "svelte";
const DEFAULT_LANGUAGE: Language = "ts";

const FRAMEWORK_LANGUAGES: Record<Framework, Language[]> = {
  svelte: ["ts", "js"],
  react: ["ts", "js"],
  vue: ["ts", "js"],
  vanilla: ["ts", "js"],
  solid: ["ts", "js"],
};

function getSupportedLanguages(framework: Framework): Language[] {
  return FRAMEWORK_LANGUAGES[framework];
}

function isFramework(value: string): value is Framework {
  return FRAMEWORKS.includes(value as Framework);
}

function isLanguage(value: string): value is Language {
  return value === "ts" || value === "js";
}

async function promptSelect<T extends string>(
  question: string,
  options: readonly T[],
  defaultValue: T,
  ask: (prompt: string) => Promise<string>,
): Promise<T> {
  while (true) {
    const optionsText = options.map((option, index) => `${index + 1}. ${option}`).join("\n");
    const answer = (await ask(`${question}\n${optionsText}\nSelect (${defaultValue}): `))
      .trim()
      .toLowerCase();

    if (!answer) {
      return defaultValue;
    }

    const selectedIndex = Number.parseInt(answer, 10);
    if (Number.isInteger(selectedIndex) && selectedIndex >= 1 && selectedIndex <= options.length) {
      return options[selectedIndex - 1]!;
    }

    const selectedOption = options.find((option) => option === answer);
    if (selectedOption) {
      return selectedOption;
    }

    output.write(`Invalid selection: ${answer}\n`);
  }
}

export async function promptForMissingOptions(initialOptions: CliOptions): Promise<Answers> {
  const initialFramework = initialOptions.framework ?? DEFAULT_FRAMEWORK;
  const initialLanguage = initialOptions.language ?? DEFAULT_LANGUAGE;

  if (initialOptions.yes) {
    const targetDir = initialOptions.targetDir || "my-onin-plugin";
    const packageName = slugify(basename(targetDir));
    const pluginName =
      initialOptions.pluginName || toTitleCase(packageName) || "My Onin Plugin";
    const pluginId =
      initialOptions.pluginId || `com.example.${packageName || "my-onin-plugin"}`;

    return {
      targetDir,
      pluginName,
      pluginId,
      withSettings: initialOptions.withSettings ?? true,
      framework: initialFramework,
      language: initialLanguage,
    };
  }

  const rl = createInterface({ input, output });

  try {
    const targetDirInput =
      initialOptions.targetDir || (await rl.question("Project directory name: ")).trim();
    const targetDir = targetDirInput || "my-onin-plugin";
    const packageName = slugify(basename(targetDir));

    const framework =
      initialOptions.framework ??
      (await promptSelect("Select a framework:", FRAMEWORKS, DEFAULT_FRAMEWORK, (prompt) =>
        rl.question(prompt),
      ));

    const supportedLanguages = getSupportedLanguages(framework);
    const defaultLanguage = supportedLanguages.includes(initialLanguage)
      ? initialLanguage
      : supportedLanguages[0]!;
    const language =
      initialOptions.language ??
      (await promptSelect("Select a language:", supportedLanguages, defaultLanguage, (prompt) =>
        rl.question(prompt),
      ));

    const pluginNameInput =
      initialOptions.pluginName ||
      (
        await rl.question(`Plugin name (${toTitleCase(packageName) || "My Onin Plugin"}): `)
      ).trim();
    const pluginName = pluginNameInput || toTitleCase(packageName) || "My Onin Plugin";

    const defaultPluginId = `com.example.${packageName || "my-onin-plugin"}`;
    const pluginIdInput =
      initialOptions.pluginId || (await rl.question(`Plugin ID (${defaultPluginId}): `)).trim();
    const pluginId = pluginIdInput || defaultPluginId;

    let withSettings = initialOptions.withSettings;
    if (withSettings === undefined) {
      const answer = (await rl.question("Include settings schema example? (Y/n): "))
        .trim()
        .toLowerCase();
      withSettings = answer !== "n";
    }

    return {
      targetDir,
      pluginName,
      pluginId,
      withSettings,
      framework,
      language,
    };
  } finally {
    rl.close();
  }
}
