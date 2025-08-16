#!/usr/bin/env node

/**
 * Plugin Development Server
 * 
 * Provides hot reloading and development tools for plugin development.
 * Watches for file changes and automatically rebuilds plugins.
 */

import { watch } from 'chokidar';
import { join, resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { existsSync, mkdirSync, writeFileSync } from 'fs';
import { buildPlugin } from './build-plugin.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const WORKSPACE_ROOT = resolve(__dirname, '..');
const PLUGINS_DIR = join(WORKSPACE_ROOT, 'plugins');

/**
 * Development server class
 */
class PluginDevServer {
  constructor(options = {}) {
    this.options = {
      verbose: false,
      debounceMs: 300,
      ...options
    };
    this.buildQueue = new Map();
    this.isBuilding = new Set();
  }

  /**
   * Start the development server
   */
  async start() {
    console.log('🚀 Starting Baize Plugin Development Server...');
    
    if (!existsSync(PLUGINS_DIR)) {
      console.error('❌ Plugins directory not found');
      process.exit(1);
    }

    // Watch for TypeScript file changes
    const watcher = watch([
      join(PLUGINS_DIR, '*/src/**/*.ts'),
      join(PLUGINS_DIR, '*/plugin.json'),
      join(PLUGINS_DIR, '*/package.json')
    ], {
      ignored: [
        /node_modules/,
        /dist/,
        /\.git/,
        /\.test\.ts$/
      ],
      persistent: true,
      ignoreInitial: true
    });

    watcher.on('change', (path) => this.handleFileChange(path, 'changed'));
    watcher.on('add', (path) => this.handleFileChange(path, 'added'));
    watcher.on('unlink', (path) => this.handleFileChange(path, 'removed'));

    // Watch for new plugins
    const pluginWatcher = watch(PLUGINS_DIR, {
      ignored: /node_modules/,
      persistent: true,
      depth: 1
    });

    pluginWatcher.on('addDir', (path) => {
      if (path !== PLUGINS_DIR) {
        const pluginName = path.split(/[/\\]/).pop();
        console.log(`📁 New plugin directory detected: ${pluginName}`);
      }
    });

    console.log('👀 Watching for changes...');
    console.log('📁 Plugins directory:', PLUGINS_DIR);
    console.log('⚡ Hot reload enabled');
    console.log('🛑 Press Ctrl+C to stop\n');

    // Handle graceful shutdown
    process.on('SIGINT', () => {
      console.log('\n🛑 Shutting down development server...');
      watcher.close();
      pluginWatcher.close();
      process.exit(0);
    });
  }

  /**
   * Handle file change events
   */
  async handleFileChange(filePath, eventType) {
    const pluginName = this.extractPluginName(filePath);
    if (!pluginName) return;

    const fileName = filePath.split(/[/\\]/).pop();
    
    if (this.options.verbose) {
      console.log(`📝 File ${eventType}: ${pluginName}/${fileName}`);
    }

    // Debounce builds for the same plugin
    this.debounceBuild(pluginName, filePath);
  }

  /**
   * Extract plugin name from file path
   */
  extractPluginName(filePath) {
    const pathParts = filePath.split(/[/\\]/);
    const pluginsIndex = pathParts.indexOf('plugins');
    
    if (pluginsIndex === -1 || pluginsIndex + 1 >= pathParts.length) {
      return null;
    }
    
    return pathParts[pluginsIndex + 1];
  }

  /**
   * Debounce build operations to avoid excessive rebuilds
   */
  debounceBuild(pluginName, filePath) {
    // Clear existing timeout
    if (this.buildQueue.has(pluginName)) {
      clearTimeout(this.buildQueue.get(pluginName));
    }

    // Set new timeout
    const timeoutId = setTimeout(() => {
      this.buildPlugin(pluginName, filePath);
      this.buildQueue.delete(pluginName);
    }, this.options.debounceMs);

    this.buildQueue.set(pluginName, timeoutId);
  }

  /**
   * Build a specific plugin
   */
  async buildPlugin(pluginName, triggerFile) {
    if (this.isBuilding.has(pluginName)) {
      if (this.options.verbose) {
        console.log(`⏳ Build already in progress for ${pluginName}, skipping...`);
      }
      return;
    }

    this.isBuilding.add(pluginName);
    
    try {
      const pluginPath = join(PLUGINS_DIR, pluginName);
      const startTime = Date.now();
      
      console.log(`🔄 Rebuilding ${pluginName}...`);
      
      const success = await buildPlugin(pluginPath, {
        clean: false,
        verbose: this.options.verbose
      });

      const duration = Date.now() - startTime;
      
      if (success) {
        console.log(`✅ ${pluginName} rebuilt successfully (${duration}ms)`);
        this.notifyBuildSuccess(pluginName, duration);
      } else {
        console.log(`❌ ${pluginName} build failed`);
        this.notifyBuildError(pluginName);
      }
      
    } catch (error) {
      console.error(`❌ Error building ${pluginName}:`, error.message);
      this.notifyBuildError(pluginName, error);
    } finally {
      this.isBuilding.delete(pluginName);
    }
  }

  /**
   * Notify about successful build (could be extended to send to main app)
   */
  notifyBuildSuccess(pluginName, duration) {
    // Send hot reload notification to main app if available
    this.sendHotReloadNotification(pluginName, 'success', { duration });
    
    if (this.options.verbose) {
      console.log(`📡 Build success notification sent for ${pluginName}`);
    }
  }

  /**
   * Notify about build error (could be extended to send to main app)
   */
  notifyBuildError(pluginName, error) {
    // Send error notification to main app if available
    this.sendHotReloadNotification(pluginName, 'error', { 
      message: error?.message || 'Build failed' 
    });
    
    if (this.options.verbose) {
      console.log(`📡 Build error notification sent for ${pluginName}`);
    }
  }

  /**
   * Send hot reload notification to main Baize app
   * @param {string} pluginName - Plugin name
   * @param {string} status - Build status (success/error)
   * @param {object} data - Additional data
   */
  async sendHotReloadNotification(pluginName, status, data = {}) {
    try {
      // Create notification file that the main app can watch
      const notificationDir = join(WORKSPACE_ROOT, '.kiro', 'dev-notifications');
      if (!existsSync(notificationDir)) {
        mkdirSync(notificationDir, { recursive: true });
      }

      const notification = {
        type: 'plugin-build',
        pluginName,
        status,
        timestamp: new Date().toISOString(),
        ...data
      };

      const notificationFile = join(notificationDir, `${pluginName}-${Date.now()}.json`);
      writeFileSync(notificationFile, JSON.stringify(notification, null, 2));

      // Clean up old notifications (keep only last 10)
      this.cleanupNotifications(notificationDir, pluginName);

    } catch (error) {
      if (this.options.verbose) {
        console.warn('⚠️  Failed to send hot reload notification:', error.message);
      }
    }
  }

  /**
   * Clean up old notification files
   * @param {string} notificationDir - Notification directory
   * @param {string} pluginName - Plugin name
   */
  async cleanupNotifications(notificationDir, pluginName) {
    try {
      const { readdirSync, statSync, unlinkSync } = await import('fs');
      
      const files = readdirSync(notificationDir)
        .filter(file => file.startsWith(`${pluginName}-`) && file.endsWith('.json'))
        .map(file => ({
          name: file,
          path: join(notificationDir, file),
          time: statSync(join(notificationDir, file)).mtime
        }))
        .sort((a, b) => b.time - a.time);

      // Keep only the 10 most recent notifications
      files.slice(10).forEach(file => {
        unlinkSync(file.path);
      });

    } catch (error) {
      // Ignore cleanup errors
    }
  }

  /**
   * Get current build status
   */
  getStatus() {
    return {
      building: Array.from(this.isBuilding),
      queued: Array.from(this.buildQueue.keys()),
      timestamp: new Date().toISOString()
    };
  }
}

/**
 * Main CLI function
 */
async function main() {
  const args = process.argv.slice(2);
  
  const options = {
    verbose: args.includes('--verbose') || args.includes('-v'),
    debounceMs: parseInt(args.find(arg => arg.startsWith('--debounce='))?.split('=')[1]) || 300
  };

  const server = new PluginDevServer(options);
  
  try {
    await server.start();
  } catch (error) {
    console.error('❌ Failed to start development server:', error);
    process.exit(1);
  }
}

// Run main function
main();

export { PluginDevServer };