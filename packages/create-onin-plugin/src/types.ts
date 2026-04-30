export type Framework = 'svelte' | 'react' | 'vue' | 'vanilla' | 'solid';
export type Language = 'ts' | 'js';

export interface CliOptions {
  targetDir: string | undefined;
  pluginName: string | undefined;
  pluginId: string | undefined;
  withSettings: boolean | undefined;
  withRelease: boolean | undefined;
  yes: boolean;
  framework: Framework | undefined;
  language: Language | undefined;
}

export interface Answers {
  targetDir: string;
  pluginName: string;
  pluginId: string;
  withSettings: boolean;
  withRelease: boolean;
  framework: Framework;
  language: Language;
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
  withRelease: boolean;
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

export type PackageJsonField = 'scripts' | 'dependencies' | 'devDependencies';
