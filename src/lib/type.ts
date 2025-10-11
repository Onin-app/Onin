export type ItemType = 'App' | 'Folder' | 'File';
export type Source = 'Application' | 'Custom' | 'Command' | 'FileCommand' | 'Plugin'
export type IconType = 'Base64' | 'Iconfont'
export type AppOrigin = 'Hkey' | 'Shortcut' | 'Uwp';


export interface LaunchableItem {
  name: string;
  keywords: CommandKeyword[];
  path: string;
  icon: string;
  icon_type: IconType;
  item_type: ItemType;
  source: Source;
  action?: string;
  origin?: AppOrigin;
  source_display?: string;
}

export interface CommandKeyword {
  name: string;
  disabled?: boolean;
  is_default?: boolean;
}

export type CommandAction =
  | { System: string }
  | { App: string }
  | { File: string }
  | { Plugin: string }
  | { PluginCommand: { plugin_id: string; command_code: string } };

export interface Command {
  name: string;
  title: string;
  english_name: string;
  keywords: CommandKeyword[];
  icon: string;
  source: Source;
  action: CommandAction;
  path?: string;
  origin?: AppOrigin;
}

export enum Theme {
  LIGHT = "light",
  DARK = "dark",
  SYSTEM = "system",
}

export interface Shortcut {
  shortcut: string;
  command_name: string;
  command_title?: string;
  readonly?: boolean;
}
