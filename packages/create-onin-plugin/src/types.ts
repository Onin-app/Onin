export interface CliOptions {
  targetDir: string | undefined;
  pluginName: string | undefined;
  pluginId: string | undefined;
  withSettings: boolean | undefined;
  yes: boolean;
  template: string;
}

export interface Answers {
  targetDir: string;
  pluginName: string;
  pluginId: string;
  withSettings: boolean;
}

export interface TemplateContext {
  packageName: string;
  pluginName: string;
  pluginId: string;
  pluginDescription: string;
  keyword: string;
  settingsImport: string;
  settingsBlock: string;
  settingsNote: string;
}

export interface PackageJsonShape {
  name?: string;
  version?: string;
  private?: boolean;
  type?: string;
  scripts?: Record<string, string>;
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
}

export type PackageJsonField = "scripts" | "dependencies" | "devDependencies";
