export type ItemType = 'App' | 'Folder' | 'File';

export interface LaunchableItem {
  name: string;
  path: string;
  icon: string;
  item_type: ItemType;
}

export enum Theme {
  LIGHT = "light",
  DARK = "dark",
  SYSTEM = "system",
}
