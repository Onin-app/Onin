import { basename, resolve } from "node:path";

import { promptForMissingOptions } from "./prompts.js";
import {
  buildSettingsBlock,
  copyTemplateDir,
  ensureTargetDirectory,
  renderPackageJson,
} from "./render.js";
import type { CliOptions, TemplateContext } from "./types.js";
import { isValidPackageName, isValidPluginId, slugify } from "./validators.js";

export async function scaffoldPlugin(
  options: CliOptions,
  baseTemplateDir: string,
  adapterTemplateDirs: Record<string, string>,
): Promise<{ targetDir: string }> {
  const answers = await promptForMissingOptions(options);
  const targetDir = resolve(process.cwd(), answers.targetDir);
  const packageName = slugify(basename(targetDir));
  const adapterTemplateDir = adapterTemplateDirs[options.framework];

  if (!adapterTemplateDir) {
    throw new Error(`Unsupported framework: ${options.framework}`);
  }

  if (!isValidPackageName(packageName)) {
    throw new Error(
      `Invalid project directory name: ${packageName}\nUse lowercase letters, numbers, dots, or hyphens only.`,
    );
  }

  if (!isValidPluginId(answers.pluginId)) {
    throw new Error(
      `Invalid plugin ID: ${answers.pluginId}\nUse lowercase letters, numbers, dots, and hyphens only.`,
    );
  }

  await ensureTargetDirectory(targetDir);

  const context: TemplateContext = {
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

  await copyTemplateDir(baseTemplateDir, targetDir, context, new Set(["package.json.tpl"]));
  await copyTemplateDir(adapterTemplateDir, targetDir, context, new Set(["package.fragment.json"]));
  await renderPackageJson(
    resolve(baseTemplateDir, "package.json.tpl"),
    resolve(adapterTemplateDir, "package.fragment.json"),
    resolve(targetDir, "package.json"),
    context,
  );

  return { targetDir: answers.targetDir };
}
