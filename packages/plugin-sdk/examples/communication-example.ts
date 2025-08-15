/**
 * Example demonstrating the API Communication Layer
 * 
 * This example shows how to use the communication bridge to interact
 * with the main application from a plugin.
 */

import { 
  createPluginContext, 
  PluginInitializer,
  CommunicationBridge,
  createCommunicationBridge,
  ApiUtils
} from '../src/index';

// Example 1: Basic plugin using the communication layer
class ExamplePlugin {
  private context: any;
  private initializer: PluginInitializer;

  constructor() {
    this.initializer = new PluginInitializer({
      pluginId: 'example-communication-plugin',
      communication: {
        maxRetries: 3,
        retryDelay: 1000,
        timeout: 5000,
        debug: true
      }
    });
  }

  async activate() {
    try {
      // Initialize the plugin context with communication bridge
      this.context = await this.initializer.initialize();

      // Example: Show a notification
      await this.context.app.showNotification('Plugin activated successfully!');

      // Example: Get app version
      const version = await this.context.app.getAppVersion();
      console.log(`Running on app version: ${version}`);

      // Example: Check permissions
      const hasNotificationPermission = await this.context.app.hasPermission('notifications');
      if (!hasNotificationPermission) {
        const granted = await this.context.app.requestPermission('notifications');
        console.log(`Notification permission ${granted ? 'granted' : 'denied'}`);
      }

      // Example: Use storage
      await this.context.storage.set('plugin-config', {
        enabled: true,
        lastActivated: new Date().toISOString()
      });

      const config = await this.context.storage.get('plugin-config');
      console.log('Plugin config:', config);

      // Example: Listen to events
      this.context.events.on('app-theme-changed', (theme: string) => {
        console.log(`App theme changed to: ${theme}`);
      });

    } catch (error) {
      console.error('Failed to activate plugin:', error);
    }
  }

  async deactivate() {
    try {
      await this.context?.app.showNotification('Plugin deactivated');
      await this.initializer.cleanup();
    } catch (error) {
      console.error('Failed to deactivate plugin:', error);
    }
  }
}

// Example 2: Using the communication bridge directly
async function directCommunicationExample() {
  // Create a communication bridge
  const bridge = createCommunicationBridge('direct-example-plugin', {
    maxRetries: 2,
    timeout: 3000,
    debug: true
  });

  try {
    // Direct API calls using the bridge
    await bridge.invoke('show_notification', { 
      message: 'Direct communication example' 
    });

    const appInfo = await bridge.invoke('get_app_info');
    console.log('App info:', appInfo);

    // Storage operations
    await bridge.invoke('storage_set', { 
      key: 'example-data', 
      value: { timestamp: Date.now() } 
    });

    const data = await bridge.invoke('storage_get', { key: 'example-data' });
    console.log('Retrieved data:', data);

  } catch (error) {
    console.error('Direct communication failed:', error);
  }
}

// Example 3: Using utility functions for safe API calls
async function safeApiExample() {
  try {
    // These functions have built-in error handling and fallbacks
    await ApiUtils.showNotificationSafe('Safe notification example');
    
    const version = await ApiUtils.getAppVersionSafe('1.0.0');
    console.log(`App version (with fallback): ${version}`);

    const hasPermission = await ApiUtils.checkPermissionSafe('storage');
    console.log(`Has storage permission: ${hasPermission}`);

  } catch (error) {
    console.error('Safe API example failed:', error);
  }
}

// Example 4: Error handling and retry demonstration
async function errorHandlingExample() {
  const bridge = createCommunicationBridge('error-example-plugin', {
    maxRetries: 3,
    retryDelay: 500,
    timeout: 2000,
    debug: true
  });

  try {
    // This might fail and trigger retry logic
    await bridge.invoke('unreliable_command', { data: 'test' });
  } catch (error) {
    console.error('Command failed after retries:', error);
  }

  try {
    // This will timeout quickly
    await bridge.invoke('slow_command', {}, { timeout: 100 });
  } catch (error) {
    console.error('Command timed out:', error);
  }
}

// Export examples for use in tests or documentation
export {
  ExamplePlugin,
  directCommunicationExample,
  safeApiExample,
  errorHandlingExample
};

// Example usage (commented out to avoid execution during import)
/*
async function runExamples() {
  console.log('=== Plugin Communication Examples ===');
  
  // Example 1: Full plugin lifecycle
  const plugin = new ExamplePlugin();
  await plugin.activate();
  // ... plugin runs ...
  await plugin.deactivate();
  
  // Example 2: Direct communication
  await directCommunicationExample();
  
  // Example 3: Safe API calls
  await safeApiExample();
  
  // Example 4: Error handling
  await errorHandlingExample();
}

// Uncomment to run examples
// runExamples().catch(console.error);
*/