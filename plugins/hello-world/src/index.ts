import type { Plugin, PluginContext } from '@baize/plugin-sdk';

/**
 * Hello World Plugin
 * 
 * A simple example plugin that demonstrates the basic plugin lifecycle
 * and shows how to use the plugin SDK APIs.
 */
export class HelloWorldPlugin implements Plugin {
  private context?: PluginContext;
  private isActive = false;

  /**
   * Plugin activation - called when the plugin is enabled
   */
  async activate(context: PluginContext): Promise<void> {
    console.log('Hello World Plugin: Activating... (Build system test)');
    
    this.context = context;
    this.isActive = true;

    try {
      // Show activation notification
      await context.app.showNotification('Hello World Plugin activated! 🎉');
      
      // Set up event listeners
      this.setupEventListeners();
      
      // Store activation time
      await context.storage.set('lastActivated', new Date().toISOString());
      
      console.log('Hello World Plugin: Successfully activated');
    } catch (error) {
      console.error('Hello World Plugin: Activation failed:', error);
      throw error;
    }
  }

  /**
   * Plugin deactivation - called when the plugin is disabled
   */
  async deactivate(): Promise<void> {
    console.log('Hello World Plugin: Deactivating...');
    
    try {
      if (this.context) {
        // Show deactivation notification
        await this.context.app.showNotification('Hello World Plugin deactivated. Goodbye! 👋');
        
        // Clean up event listeners
        this.cleanupEventListeners();
        
        // Store deactivation time
        await this.context.storage.set('lastDeactivated', new Date().toISOString());
      }
      
      this.isActive = false;
      this.context = undefined;
      
      console.log('Hello World Plugin: Successfully deactivated');
    } catch (error) {
      console.error('Hello World Plugin: Deactivation failed:', error);
      throw error;
    }
  }

  /**
   * Set up event listeners for app events
   */
  private setupEventListeners(): void {
    if (!this.context) return;

    // Listen for app ready event
    this.context.events.on('app:ready', this.onAppReady.bind(this));
    
    // Listen for app shutdown event
    this.context.events.on('app:shutdown', this.onAppShutdown.bind(this));
    
    // Listen for custom hello event (for demonstration)
    this.context.events.on('hello:greet', this.onGreetEvent.bind(this));
  }

  /**
   * Clean up event listeners
   */
  private cleanupEventListeners(): void {
    if (!this.context) return;

    this.context.events.off('app:ready', this.onAppReady.bind(this));
    this.context.events.off('app:shutdown', this.onAppShutdown.bind(this));
    this.context.events.off('hello:greet', this.onGreetEvent.bind(this));
  }

  /**
   * Handle app ready event
   */
  private async onAppReady(): Promise<void> {
    console.log('Hello World Plugin: App is ready!');
    
    if (this.context) {
      try {
        // Get app version and show it
        const version = await this.context.app.getAppVersion();
        console.log(`Hello World Plugin: Running on Baize v${version}`);
        
        // Emit a custom event
        this.context.events.emit('hello:plugin-ready', {
          pluginName: 'hello-world',
          timestamp: new Date().toISOString()
        });
      } catch (error) {
        console.error('Hello World Plugin: Error handling app ready:', error);
      }
    }
  }

  /**
   * Handle app shutdown event
   */
  private async onAppShutdown(): Promise<void> {
    console.log('Hello World Plugin: App is shutting down');
    
    if (this.context) {
      try {
        // Save shutdown time
        await this.context.storage.set('lastShutdown', new Date().toISOString());
      } catch (error) {
        console.error('Hello World Plugin: Error handling shutdown:', error);
      }
    }
  }

  /**
   * Handle custom greet event
   */
  private async onGreetEvent(data?: any): Promise<void> {
    console.log('Hello World Plugin: Received greet event:', data);
    
    if (this.context) {
      try {
        const message = data?.message || 'Hello from the Hello World Plugin!';
        await this.context.app.showNotification(message);
      } catch (error) {
        console.error('Hello World Plugin: Error handling greet event:', error);
      }
    }
  }

  /**
   * Get plugin status
   */
  public isPluginActive(): boolean {
    return this.isActive;
  }

  /**
   * Get stored data (for demonstration)
   */
  public async getStoredData(): Promise<any> {
    if (!this.context) return null;
    
    try {
      const data = {
        lastActivated: await this.context.storage.get('lastActivated'),
        lastDeactivated: await this.context.storage.get('lastDeactivated'),
        lastShutdown: await this.context.storage.get('lastShutdown')
      };
      return data;
    } catch (error) {
      console.error('Hello World Plugin: Error getting stored data:', error);
      return null;
    }
  }
}

// Export the plugin instance
export default HelloWorldPlugin;