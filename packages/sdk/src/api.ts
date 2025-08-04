// API definitions
export interface PluginAPI {
  showNotification(title: string, message: string): void;
  registerCommand(command: string, handler: () => void): void;
  readFile(path: string): Promise<string>;
  executeCommand(command: string): Promise<string>;
}