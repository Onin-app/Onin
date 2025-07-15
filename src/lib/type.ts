export type ItemType = 'App' | 'Folder' | 'File';
export type Source = 'Application' | 'Custom'

export interface LaunchableItem {
  name: string;
  path: string;
  icon: string;
  item_type: ItemType;
  source: Source
}

export enum Theme {
  LIGHT = "light",
  DARK = "dark",
  SYSTEM = "system",
}
