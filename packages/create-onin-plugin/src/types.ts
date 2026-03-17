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
