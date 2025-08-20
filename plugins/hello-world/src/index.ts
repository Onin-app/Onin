import { definePlugin, useApp, useEvents, useStorage } from '@baize/plugin-sdk';

/**
 * Hello World Plugin - Functional Style
 * 
 * A simple example plugin that demonstrates the functional plugin lifecycle
 * and shows how to use the new hook-style plugin SDK APIs.
 */

// Event handler functions
let onAppReady: (() => Promise<void>) | undefined;
let onAppShutdown: (() => Promise<void>) | undefined;
let onGreetEvent: ((data?: any) => Promise<void>) | undefined;

/**
 * Handle app ready event
 */
async function handleAppReady(): Promise<void> {
  console.log('Hello World Plugin: App is ready!');
  
  try {
    const app = useApp();
    const events = useEvents();
    
    // Get app version and show it
    const version = await app.getAppVersion();
    console.log(`Hello World Plugin: Running on Baize v${version}`);
    
    // Emit a custom event
    events.emit('hello:plugin-ready', {
      pluginName: 'hello-world',
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    console.error('Hello World Plugin: Error handling app ready:', error);
  }
}

/**
 * Handle app shutdown event
 */
async function handleAppShutdown(): Promise<void> {
  console.log('Hello World Plugin: App is shutting down');
  
  try {
    const storage = useStorage();
    // Save shutdown time
    await storage.set('lastShutdown', new Date().toISOString());
  } catch (error) {
    console.error('Hello World Plugin: Error handling shutdown:', error);
  }
}

/**
 * Handle custom greet event
 */
async function handleGreetEvent(data?: any): Promise<void> {
  console.log('Hello World Plugin: Received greet event:', data);
  
  try {
    const app = useApp();
    const message = data?.message || 'Hello from the Hello World Plugin!';
    await app.showNotification(message);
  } catch (error) {
    console.error('Hello World Plugin: Error handling greet event:', error);
  }
}

/**
 * Set up event listeners for app events
 */
function setupEventListeners(): void {
  const events = useEvents();

  // Create bound functions to store references for cleanup
  onAppReady = handleAppReady;
  onAppShutdown = handleAppShutdown;
  onGreetEvent = handleGreetEvent;

  // Listen for app ready event
  events.on('app:ready', onAppReady);
  
  // Listen for app shutdown event
  events.on('app:shutdown', onAppShutdown);
  
  // Listen for custom hello event (for demonstration)
  events.on('hello:greet', onGreetEvent);
}

/**
 * Clean up event listeners
 */
function cleanupEventListeners(): void {
  const events = useEvents();

  if (onAppReady) {
    events.off('app:ready', onAppReady);
    onAppReady = undefined;
  }
  
  if (onAppShutdown) {
    events.off('app:shutdown', onAppShutdown);
    onAppShutdown = undefined;
  }
  
  if (onGreetEvent) {
    events.off('hello:greet', onGreetEvent);
    onGreetEvent = undefined;
  }
}

/**
 * Utility function to get stored data (for demonstration)
 * This can be called from external code if needed
 */
export async function getStoredData(): Promise<any> {
  try {
    const storage = useStorage();
    const data = {
      lastActivated: await storage.get('lastActivated'),
      lastDeactivated: await storage.get('lastDeactivated'),
      lastShutdown: await storage.get('lastShutdown')
    };
    return data;
  } catch (error) {
    console.error('Hello World Plugin: Error getting stored data:', error);
    return null;
  }
}

// Define the functional plugin
const helloWorldPlugin = definePlugin({
  meta: {
    name: 'Hello World Plugin',
    version: '1.0.0',
    description: 'A simple example plugin demonstrating functional plugin development',
    author: 'Baize Team'
  },
  
  /**
   * Plugin activation - called when the plugin is enabled
   */
  onActivate: async (context) => {
    console.log('Hello World Plugin: Activating... (Build system test)');

    try {
      // Use destructuring to get APIs from context
      const { app, storage } = context;
      
      // Show activation notification
      await app.showNotification('Hello World Plugin activated! 🎉');
      
      // Set up event listeners
      setupEventListeners();
      
      // Store activation time
      await storage.set('lastActivated', new Date().toISOString());
      
      console.log('Hello World Plugin: Successfully activated');
    } catch (error) {
      console.error('Hello World Plugin: Activation failed:', error);
      throw error;
    }
  },

  /**
   * Plugin deactivation - called when the plugin is disabled
   */
  onDeactivate: async () => {
    console.log('Hello World Plugin: Deactivating...');
    
    try {
      const app = useApp();
      const storage = useStorage();
      
      // Show deactivation notification
      await app.showNotification('Hello World Plugin deactivated. Goodbye! 👋');
      
      // Clean up event listeners
      cleanupEventListeners();
      
      // Store deactivation time
      await storage.set('lastDeactivated', new Date().toISOString());
      
      console.log('Hello World Plugin: Successfully deactivated');
    } catch (error) {
      console.error('Hello World Plugin: Deactivation failed:', error);
      throw error;
    }
  }
});

// Export the functional plugin as default
export default helloWorldPlugin;