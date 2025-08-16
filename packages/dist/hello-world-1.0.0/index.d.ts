import type { Plugin, PluginContext } from '@baize/plugin-sdk';
/**
 * Hello World Plugin
 *
 * A simple example plugin that demonstrates the basic plugin lifecycle
 * and shows how to use the plugin SDK APIs.
 */
export declare class HelloWorldPlugin implements Plugin {
    private context?;
    private isActive;
    /**
     * Plugin activation - called when the plugin is enabled
     */
    activate(context: PluginContext): Promise<void>;
    /**
     * Plugin deactivation - called when the plugin is disabled
     */
    deactivate(): Promise<void>;
    /**
     * Set up event listeners for app events
     */
    private setupEventListeners;
    /**
     * Clean up event listeners
     */
    private cleanupEventListeners;
    /**
     * Handle app ready event
     */
    private onAppReady;
    /**
     * Handle app shutdown event
     */
    private onAppShutdown;
    /**
     * Handle custom greet event
     */
    private onGreetEvent;
    /**
     * Get plugin status
     */
    isPluginActive(): boolean;
    /**
     * Get stored data (for demonstration)
     */
    getStoredData(): Promise<any>;
}
export default HelloWorldPlugin;
//# sourceMappingURL=index.d.ts.map