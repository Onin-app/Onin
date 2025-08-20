/**
 * Example demonstrating Hook-style API usage
 * 
 * This example shows how to use the new functional hooks API
 * to create a plugin with modern, React-style hooks.
 */

import { definePlugin, useApp, useEvents, useStorage, onActivate, onDeactivate } from '../src';

// Example plugin using hooks
export default definePlugin({
  meta: {
    name: 'hooks-example',
    version: '1.0.0',
    description: 'Example plugin demonstrating hook-style API',
    author: 'Baize Team'
  },

  async onActivate() {
    // Register activation handlers using hooks
    onActivate(async () => {
      const app = useApp();
      await app.showNotification('Plugin activated with hooks!');
      
      const storage = useStorage();
      await storage.set('activation-time', new Date().toISOString());
    });

    // Register event listeners
    onActivate(() => {
      const events = useEvents();
      
      events.on('user-action', (data) => {
        console.log('User action received:', data);
      });
      
      events.on('app-ready', async () => {
        const app = useApp();
        await app.showNotification('App is ready!');
      });
    });

    // Register cleanup handlers
    onDeactivate(async () => {
      const app = useApp();
      await app.showNotification('Plugin is deactivating...');
      
      const storage = useStorage();
      await storage.set('deactivation-time', new Date().toISOString());
    });

    onDeactivate(() => {
      const events = useEvents();
      // Clean up event listeners
      events.off('user-action', () => {});
      events.off('app-ready', () => {});
    });

    // Execute the activation handlers
    // (This would normally be done by the plugin manager)
  },

  async onDeactivate() {
    // Execute the deactivation handlers
    // (This would normally be done by the plugin manager)
  }
});

// Alternative approach: Direct hook usage in lifecycle functions
export const directHooksPlugin = definePlugin({
  meta: {
    name: 'direct-hooks-example',
    version: '1.0.0',
    description: 'Example using hooks directly in lifecycle functions'
  },

  async onActivate() {
    // Use hooks directly in the activation function
    const app = useApp();
    const events = useEvents();
    const storage = useStorage();

    // Show activation notification
    await app.showNotification('Direct hooks plugin activated!');

    // Set up event listeners
    events.on('data-update', async (data) => {
      await storage.set('latest-data', data);
      await app.showNotification(`Data updated: ${JSON.stringify(data)}`);
    });

    // Store activation info
    await storage.set('plugin-info', {
      name: 'direct-hooks-example',
      activatedAt: new Date().toISOString(),
      version: '1.0.0'
    });

    // Emit ready event
    events.emit('plugin-ready', { pluginName: 'direct-hooks-example' });
  },

  async onDeactivate() {
    const app = useApp();
    const storage = useStorage();

    // Show deactivation notification
    await app.showNotification('Direct hooks plugin deactivated!');

    // Clean up storage
    await storage.remove('latest-data');
    
    // Update plugin info
    const info = await storage.get('plugin-info');
    if (info) {
      await storage.set('plugin-info', {
        ...info,
        deactivatedAt: new Date().toISOString()
      });
    }
  }
});

// Example showing error handling with hooks
export const errorHandlingPlugin = definePlugin({
  meta: {
    name: 'error-handling-example',
    version: '1.0.0',
    description: 'Example showing error handling with hooks'
  },

  async onActivate() {
    try {
      const app = useApp();
      const storage = useStorage();

      // Attempt to load previous state
      const previousState = await storage.get('plugin-state');
      
      if (previousState) {
        await app.showNotification(`Restored state: ${JSON.stringify(previousState)}`);
      } else {
        await app.showNotification('Starting fresh - no previous state found');
        await storage.set('plugin-state', { initialized: true });
      }

      // Set up error-prone event handler
      const events = useEvents();
      events.on('risky-operation', async (data) => {
        try {
          // Simulate risky operation
          if (data.shouldFail) {
            throw new Error('Simulated failure');
          }
          
          await storage.set('last-operation', data);
          await app.showNotification('Risky operation succeeded');
        } catch (error) {
          console.error('Risky operation failed:', error);
          await app.showNotification(`Operation failed: ${error.message}`);
        }
      });

    } catch (error) {
      console.error('Plugin activation failed:', error);
      
      // Try to show error notification if possible
      try {
        const app = useApp();
        await app.showNotification(`Plugin activation error: ${error.message}`);
      } catch {
        console.error('Could not show error notification');
      }
    }
  },

  async onDeactivate() {
    try {
      const app = useApp();
      await app.showNotification('Error handling plugin deactivated safely');
    } catch (error) {
      console.error('Deactivation error:', error);
    }
  }
});