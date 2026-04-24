export type ItemType = "App" | "Folder" | "File";
export type Source =
  | "Application"
  | "Custom"
  | "Command"
  | "FileCommand"
  | "FileSearch"
  | "Plugin"
  | "Extension";
export type IconType = "Base64" | "Iconfont" | "Url";
export type AppOrigin = "Hkey" | "Shortcut" | "Uwp";

export interface LaunchableItem {
  name: string;
  description?: string;
  keywords: CommandKeyword[];
  path: string;
  icon: string;
  icon_type: IconType;
  item_type: ItemType;
  source: Source;
  action?: string;
  origin?: AppOrigin;
  source_display?: string;
  matches?: CommandMatch[];
  requires_confirmation?: boolean;
  trigger_mode?: "matched" | "preview";
}

export interface CommandKeyword {
  name: string;
  disabled?: boolean;
  is_default?: boolean;
}

/**
 * 命令匹配配置
 *
 * 三层优雅降级模型：
 * 1. 开发者层：只需配置 extensions（如 [".png", ".jpg"]）
 * 2. 系统层：自动将 extensions 映射为内部 MIME 类型
 * 3. 运行层：优先使用 MIME 类型判断，fallback 到 extensions
 */
export interface CommandMatch {
  type: "text" | "image" | "file" | "folder";
  name: string;
  description: string;
  /** 正则表达式（仅 type="text" 时使用，作为额外的匹配条件） */
  regexp?: string;
  /** 最小数量（text: 字符数, file/image/folder: 文件数量） */
  min?: number;
  /** 最大数量（text: 字符数, file/image/folder: 文件数量） */
  max?: number;
  /** 文件扩展名数组（如 [".png", ".jpg"]），支持通配符 "*" */
  extensions?: string[];
}

export type CommandAction =
  | { System: string }
  | { App: string }
  | { File: string }
  | { PluginEntry: { plugin_id: string } }
  | { PluginCommand: { plugin_id: string; command_code: string } }
  | { Extension: { extension_id: string; command_code: string } };

export interface Command {
  name: string;
  title: string;
  description?: string;
  english_name: string;
  keywords: CommandKeyword[];
  icon: string;
  source: Source;
  action: CommandAction;
  path?: string;
  origin?: AppOrigin;
  matches?: CommandMatch[];
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

export type SortMode = "smart" | "frequency" | "recent" | "default";

export interface CommandUsageStats {
  command_name: string;
  usage_count: number;
  last_used: number;
}

export interface AppConfig {
  auto_paste_time_limit: number;
  auto_clear_time_limit: number;
  sort_mode: SortMode;
  enable_usage_tracking: boolean;
  marketplace_api_url?: string;
  disabled_extension_ids?: string[];
}
