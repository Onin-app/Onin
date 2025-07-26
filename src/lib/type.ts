export type ItemType = 'App' | 'Folder' | 'File';
export type Source = 'Application' | 'Custom' | 'Command'
export type IconType = 'Base64' | 'Iconfont'

export interface LaunchableItem {
  name: string;
  aliases: string[];
  path: string;
  icon: string;
  icon_type: IconType;
  item_type: ItemType;
  source: Source;
  action?: string;
}

export enum Theme {
  LIGHT = "light",
  DARK = "dark",
  SYSTEM = "system",
}
