export interface AppInfo {
  name: string;
  path: string;
  icon?: string;
  origin?: string;
}

export enum Theme {
  LIGHT = "light",
  DARK = "dark",
  SYSTEM = "system",
}
