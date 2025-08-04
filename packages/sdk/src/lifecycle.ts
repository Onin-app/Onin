// Plugin lifecycle definitions
export interface PluginLifecycle {
  onActivate(): void;
  onDeactivate(): void;
  onError(error: Error): void;
}