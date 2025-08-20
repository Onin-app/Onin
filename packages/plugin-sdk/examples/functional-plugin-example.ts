/**
 * Example of a functional plugin using the new definePlugin API
 * 
 * This example demonstrates how to create a plugin using the functional approach
 * instead of the traditional class-based approach.
 */

import { definePlugin } from '../src/plugin';

// Example 1: Minimal plugin with just activation
export const minimalPlugin = definePlugin({
  onActivate: async (context) => {
    await context.app.showNotification('Minimal plugin activated!');
  }
});

// Example 2: Full plugin with metadata and both lifecycle functions
export const fullPlugin = definePlugin({
  meta: {
    name: 'Example Functional Plugin',
    version: '1.0.0',
    description: 'A demonstration of the functional plugin API',
    author: 'Plugin Developer'
  },
  onActivate: async (context) => {
    // Access application API
    await context.app.showNotification('Functional plugin activated!');
    
    // Set up event listeners
    context.events.on('user-action', (data) => {
      console.log('User action received:', data);
    });
    
    // Store some initial data
    await context.storage.set('activation-time', new Date().toISOString());
    
    console.log('Plugin activated successfully');
  },
  onDeactivate: async () => {
    console.log('Plugin is being deactivated');
    // Cleanup logic here
  }
});

// Example 3: Plugin with only deactivation (cleanup-only plugin)
export const cleanupOnlyPlugin = definePlugin({
  meta: {
    name: 'Cleanup Plugin',
    version: '1.0.0',
    description: 'A plugin that only handles cleanup'
  },
  onDeactivate: async () => {
    console.log('Performing cleanup operations...');
    // Cleanup logic here
  }
});

// Example 4: Plugin with error handling
export const robustPlugin = definePlugin({
  meta: {
    name: 'Robust Plugin',
    version: '1.0.0',
    description: 'A plugin with proper error handling'
  },
  onActivate: async (context) => {
    try {
      await context.app.showNotification('Robust plugin starting...');
      
      // Simulate some initialization that might fail
      const config = await context.storage.get('plugin-config');
      if (!config) {
        await context.storage.set('plugin-config', {
          initialized: true,
          timestamp: Date.now()
        });
      }
      
      await context.app.showNotification('Robust plugin activated successfully!');
    } catch (error) {
      console.error('Plugin activation failed:', error);
      await context.app.showNotification('Plugin activation failed - check console for details');
      throw error; // Re-throw to let the plugin system handle it
    }
  },
  onDeactivate: async () => {
    try {
      console.log('Robust plugin deactivating...');
      // Cleanup operations
    } catch (error) {
      console.error('Plugin deactivation error:', error);
      // Don't re-throw deactivation errors to avoid blocking shutdown
    }
  }
});

// Example 5: Plugin using modern JavaScript features
export const modernPlugin = definePlugin({
  meta: {
    name: 'Modern JS Plugin',
    version: '2.0.0-beta.1',
    description: 'Demonstrates modern JavaScript features in plugins'
  },
  onActivate: async (context) => {
    // Using destructuring
    const { app, events, storage } = context;
    
    // Using arrow functions and async/await
    const handleUserEvent = async (eventData: any) => {
      const { type, payload } = eventData;
      await app.showNotification(`Received ${type} event`);
      
      // Store event history
      const history = await storage.get('event-history') || [];
      history.push({ type, timestamp: Date.now(), payload });
      await storage.set('event-history', history.slice(-10)); // Keep last 10 events
    };
    
    // Set up event listener
    events.on('user-event', handleUserEvent);
    
    // Use template literals
    await app.showNotification(`Modern plugin activated at ${new Date().toLocaleTimeString()}`);
  }
});

// Default export - this would be the main plugin if this file was a plugin entry point
export default fullPlugin;