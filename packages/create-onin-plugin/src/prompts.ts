import { basename } from "node:path";

import { cancel, confirm, intro, isCancel, outro, select, text } from "@clack/prompts";

import type { Answers, CliOptions, Framework, Language } from "./types.js";
import { slugify, toTitleCase } from "./validators.js";

const FRAMEWORKS: { value: Framework; label: string }[] = [
  { value: "svelte", label: "Svelte" },
  { value: "react", label: "React" },
  { value: "vue", label: "Vue" },
  { value: "vanilla", label: "Vanilla" },
  { value: "solid", label: "Solid" },
];

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

function ensurePromptValue<T>(value: T | symbol): T {
  if (isCancel(value)) {
    cancel("Plugin creation cancelled.");
    process.exit(1);
  }

  return value;
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

  intro("create-onin-plugin");

  const targetDir =
    initialOptions.targetDir ||
    ensurePromptValue(
      await text({
        message: "Project directory name",
        placeholder: "my-onin-plugin",
        defaultValue: "my-onin-plugin",
      }),
    ).trim() ||
      "my-onin-plugin";

  const packageName = slugify(basename(targetDir));
  const defaultPluginName = toTitleCase(packageName) || "My Onin Plugin";
  const defaultPluginId = `com.example.${packageName || "my-onin-plugin"}`;

  const framework =
    initialOptions.framework ??
    ensurePromptValue(
      await select<Framework>({
        message: "Select a framework",
        initialValue: initialFramework,
        options: FRAMEWORKS,
      }),
    );

  const supportedLanguages = getSupportedLanguages(framework);
  const languageDefault = supportedLanguages.includes(initialLanguage)
    ? initialLanguage
    : supportedLanguages[0]!;
  const language =
    initialOptions.language ??
    ensurePromptValue(
      await select<Language>({
        message: "Select a language",
        initialValue: languageDefault,
        options: supportedLanguages.map((value) => ({
          value,
          label: value.toUpperCase(),
        })),
      }),
    );

  const pluginName =
    initialOptions.pluginName ||
    ensurePromptValue(
      await text({
        message: "Plugin name",
        placeholder: defaultPluginName,
        defaultValue: defaultPluginName,
      }),
    ).trim() ||
      defaultPluginName;

  const pluginId =
    initialOptions.pluginId ||
    ensurePromptValue(
      await text({
        message: "Plugin ID",
        placeholder: defaultPluginId,
        defaultValue: defaultPluginId,
      }),
    ).trim() ||
      defaultPluginId;

  const withSettings =
    initialOptions.withSettings ??
    ensurePromptValue(
      await confirm({
        message: "Include settings schema example?",
        initialValue: true,
      }),
    );

  outro("Project configuration captured.");

  return {
    targetDir,
    pluginName,
    pluginId,
    withSettings,
    framework,
    language,
  };
}
