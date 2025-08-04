interface PluginAPI {
    showNotification(title: string, message: string): void;
    registerCommand(command: string, handler: () => void): void;
    readFile(path: string): Promise<string>;
    executeCommand(command: string): Promise<string>;
}

interface PluginLifecycle {
    onActivate(): void;
    onDeactivate(): void;
    onError(error: Error): void;
}

interface AppInfo {
    id: string;
    name: string;
    version: string;
}

export type { AppInfo, PluginAPI, PluginLifecycle };
