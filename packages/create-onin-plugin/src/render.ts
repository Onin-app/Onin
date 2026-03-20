import { copyFile, mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import { join, relative } from "node:path";

import type { PackageJsonField, PackageJsonShape, TemplateContext } from "./types.js";
import { isNodeError } from "./validators.js";

export async function ensureTargetDirectory(targetDir: string): Promise<void> {
  try {
    const targetStat = await stat(targetDir);
    if (!targetStat.isDirectory()) {
      throw new Error(
        `Target path exists and is not a directory: ${targetDir}\nChoose a new directory name and try again.`,
      );
    }

    const entries = await readdir(targetDir);
    if (entries.length > 0) {
      throw new Error(
        `Target directory is not empty: ${targetDir}\nUse an empty directory or choose a new project name.`,
      );
    }
  } catch (error: unknown) {
    if (isNodeError(error) && error.code === "ENOENT") {
      await mkdir(targetDir, { recursive: true });
      return;
    }

    throw error;
  }
}

export function renderTemplate(content: string, context: TemplateContext): string {
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

export async function copyTemplateDir(
  sourceDir: string,
  targetDir: string,
  context: TemplateContext,
  skipRelativePaths: Set<string> = new Set(),
): Promise<void> {
  await copyTemplateDirInternal(sourceDir, sourceDir, targetDir, context, skipRelativePaths);
}

async function copyTemplateDirInternal(
  rootSourceDir: string,
  sourceDir: string,
  targetDir: string,
  context: TemplateContext,
  skipRelativePaths: Set<string>,
): Promise<void> {
  const entries = await readdir(sourceDir, { withFileTypes: true });

  for (const entry of entries) {
    const sourcePath = join(sourceDir, entry.name);
    const relativePath = relative(rootSourceDir, sourcePath).replaceAll("\\", "/");
    if (skipRelativePaths.has(relativePath)) {
      continue;
    }

    const outputName = entry.name.endsWith(".tpl") ? entry.name.slice(0, -4) : entry.name;
    const targetPath = join(targetDir, outputName);

    if (entry.isDirectory()) {
      await mkdir(targetPath, { recursive: true });
      await copyTemplateDirInternal(rootSourceDir, sourcePath, targetPath, context, skipRelativePaths);
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

function mergeOptionalRecordField(
  basePkg: PackageJsonShape,
  adapterPkg: PackageJsonShape,
  field: PackageJsonField,
): Record<string, string> | undefined {
  const merged = {
    ...(basePkg[field] ?? {}),
    ...(adapterPkg[field] ?? {}),
  };

  return Object.keys(merged).length > 0 ? merged : undefined;
}

export async function renderPackageJson(
  baseTemplatePath: string,
  adapterFragmentPath: string,
  targetPath: string,
  context: TemplateContext,
): Promise<void> {
  const [baseTemplate, adapterFragment] = await Promise.all([
    readFile(baseTemplatePath, "utf8"),
    readFile(adapterFragmentPath, "utf8"),
  ]);

  const renderedBase = renderTemplate(baseTemplate, context);
  const renderedAdapter = renderTemplate(adapterFragment, context);
  const basePkg = JSON.parse(renderedBase) as PackageJsonShape;
  const adapterPkg = JSON.parse(renderedAdapter) as PackageJsonShape;
  const scripts = mergeOptionalRecordField(basePkg, adapterPkg, "scripts");
  const dependencies = mergeOptionalRecordField(basePkg, adapterPkg, "dependencies");
  const devDependencies = mergeOptionalRecordField(basePkg, adapterPkg, "devDependencies");

  const mergedPkg: PackageJsonShape = {
    ...basePkg,
    ...adapterPkg,
  };

  if (scripts) {
    mergedPkg.scripts = scripts;
  }

  if (dependencies) {
    mergedPkg.dependencies = dependencies;
  }

  if (devDependencies) {
    mergedPkg.devDependencies = devDependencies;
  }

  removeEmptyRecordField(mergedPkg, "scripts");
  removeEmptyRecordField(mergedPkg, "dependencies");
  removeEmptyRecordField(mergedPkg, "devDependencies");

  await writeFile(targetPath, `${JSON.stringify(mergedPkg, null, 2)}\n`, "utf8");
}

function removeEmptyRecordField(pkg: PackageJsonShape, field: PackageJsonField): void {
  const record = pkg[field];
  if (record && Object.keys(record).length === 0) {
    delete pkg[field];
  }
}

export function buildSettingsBlock(withSettings: boolean): string {
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
