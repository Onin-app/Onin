export type ItemType = 'App' | 'Folder' | 'File';
export type Source = 'Application' | 'Custom' | 'Command' | 'FileCommand'
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
}

export interface CommandKeyword {
  name: string;
  disabled?: boolean;
}

export type CommandAction = { System: string } | { App: string };

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
