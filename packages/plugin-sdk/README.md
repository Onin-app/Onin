# @baize/plugin-sdk

Official SDK for developing plugins for the Baize application. This SDK provides TypeScript interfaces, types, and utilities for creating robust and type-safe plugins.

## Installation

```bash
npm install @baize/plugin-sdk
```

## Quick Start

### 1. Create a Plugin

```typescript
import { Plugin, PluginContext } from '@baize/plugin-sdk';

export default class MyPlugin implements Plugin {
  async activate(context: PluginContext): Promise<void> {
    // Plugin activation logic
    await context.app.showNotification('Hello from my plugin!');
    
    // Listen to events
    context.events.on('app:startup', () => {
      console.log('App started!');
    });
    
    // Store some data
    await context.storage.set('initialized', true);
  }

  async deactivate(): Promise<void> {
    // Cleanup logic when plugin is deactivated
    console.log('Plugin deactivated');
  }
}
```

### 2. Create Plugin Manifest

Create a `plugin.json` file in your plugin root:

```json
{
  "name": "my-awesome-plugin",
  "version": "1.0.0",
  "description": "An awesome plugin for Baize",
  "author": "Your Name",
  "main": "dist/index.js",
  "permissions": [
    "notifications",
    "storage"
  ],
  "engines": {
    "baize": ">=0.1.0"
  },
  "keywords": ["utility", "example"]
}
```

## Core Interfaces

### Plugin

The main interface that all plugins must implement:

```typescript
interface Plugin {
  activate(context: PluginContext): Promise<void>;
  deactivate(): Promise<void>;
}
```

### PluginContext

The context object provided to plugins during activation:

```typescript
interface PluginContext {
  app: AppAPI;      // Application API
  events: EventAPI; // Event system
  storage: StorageAPI; // Persistent storage
}
```

### AppAPI

Interface for interacting with the main application:

```typescript
interface AppAPI {
  showNotification(message: string): Promise<void>;
  getAppVersion(): Promise<string>;
  openDialog(options: DialogOptions): Promise<string | null>;
}
```

### EventAPI

Interface for the event system:

```typescript
interface EventAPI {
  on(event: string, handler: EventHandler): void;
  emit(event: string, data?: any): void;
  off(event: string, handler: EventHandler): void;
}
```

### StorageAPI

Interface for persistent data storage:

```typescript
interface StorageAPI {
  get(key: string): Promise<any>;
  set(key: string, value: any): Promise<void>;
  remove(key: string): Promise<void>;
  clear(): Promise<void>;
  keys(): Promise<string[]>;
}
```

## Utilities

### validatePluginManifest

Utility function to validate a plugin manifest:

```typescript
import { validatePluginManifest } from '@baize/plugin-sdk';

const manifest = { /* your manifest */ };
if (validatePluginManifest(manifest)) {
  console.log('Manifest is valid!');
}
```

### createPluginError

Utility function to create standardized plugin errors:

```typescript
import { createPluginError, PluginErrorCode } from '@baize/plugin-sdk';

throw createPluginError(
  'Something went wrong',
  PluginErrorCode.ACTIVATION_FAILED,
  'my-plugin'
);
```

## Communication Layer

The SDK includes a robust communication layer that handles all plugin-app communication with built-in error handling and retry mechanisms.

### CommunicationBridge

The core communication class that manages API calls:

```typescript
import { createCommunicationBridge } from '@baize/plugin-sdk';

const bridge = createCommunicationBridge('my-plugin', {
  maxRetries: 3,
  retryDelay: 1000,
  timeout: 5000,
  debug: true
});

// Make API calls
const result = await bridge.invoke('some_command', { param: 'value' });
```

### Plugin Context Factory

Create a complete plugin context with communication bridge:

```typescript
import { createPluginContext } from '@baize/plugin-sdk';

const context = createPluginContext({
  pluginId: 'my-plugin',
  communication: {
    maxRetries: 3,
    timeout: 5000
  }
});

// Use the context
await context.app.showNotification('Hello!');
```

### Safe API Utilities

Utility functions with built-in error handling:

```typescript
import { ApiUtils } from '@baize/plugin-sdk';

// Safe notification with fallback
await ApiUtils.showNotificationSafe('Message', (msg) => {
  console.log(`Fallback: ${msg}`);
});

// Safe version check with fallback
const version = await ApiUtils.getAppVersionSafe('1.0.0');

// Safe permission check
const hasPermission = await ApiUtils.checkPermissionSafe('storage');
```

## System Events

The SDK defines several system events that plugins can listen to:

- `app:startup` - Application is starting up
- `app:shutdown` - Application is shutting down
- `plugin:loaded` - A plugin was loaded
- `plugin:unloaded` - A plugin was unloaded
- `plugin:error` - A plugin encountered an error
- `settings:changed` - Application settings changed
- `theme:changed` - Application theme changed

## Error Handling

The SDK provides a comprehensive error system with specific error codes:

```typescript
enum PluginErrorCode {
  NOT_FOUND = 'NOT_FOUND',
  INVALID_MANIFEST = 'INVALID_MANIFEST',
  LOAD_FAILED = 'LOAD_FAILED',
  PERMISSION_DENIED = 'PERMISSION_DENIED',
  VERSION_INCOMPATIBLE = 'VERSION_INCOMPATIBLE',
  API_CALL_FAILED = 'API_CALL_FAILED',
  ACTIVATION_FAILED = 'ACTIVATION_FAILED'
}
```

## TypeScript Support

This SDK is written in TypeScript and provides full type definitions. All interfaces and types are exported for use in your plugin development.

## License

MIT