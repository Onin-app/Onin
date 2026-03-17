import { stdin as input, stdout as output } from "node:process";
import { basename } from "node:path";
import { createInterface } from "node:readline/promises";

import type { Answers, CliOptions } from "./types.js";
import { slugify, toTitleCase } from "./validators.js";

export async function promptForMissingOptions(initialOptions: CliOptions): Promise<Answers> {
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
    };
  }

  const rl = createInterface({ input, output });

  try {
    const targetDirInput =
      initialOptions.targetDir || (await rl.question("Project directory name: ")).trim();
    const targetDir = targetDirInput || "my-onin-plugin";
    const packageName = slugify(basename(targetDir));

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
    };
  } finally {
    rl.close();
  }
}
