import type { AppInfo } from './type';

export interface PluginAPI {
  // UI 相关
  showNotification(title: string, message: string): void;
  registerCommand(command: string, handler: () => void): void;

  // 数据存储
  getConfig(key: string): Promise<any>;
  setConfig(key: string, value: any): Promise<void>;

  // 系统访问
  readFile(path: string): Promise<string>;
  writeFile(path: string, content: string): Promise<void>;
  executeCommand(command: string): Promise<string>;

  // 应用功能
  searchApps(query: string): Promise<AppInfo[]>;
  launchApp(appId: string): Promise<void>;
}