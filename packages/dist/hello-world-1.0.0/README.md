# Hello World Plugin

A simple example plugin that demonstrates the basic functionality of the Baize plugin system.

## Features

- **Plugin Lifecycle**: Demonstrates proper activation and deactivation
- **Notifications**: Shows how to display notifications to users
- **Event System**: Listens to and emits custom events
- **Storage API**: Stores and retrieves persistent data
- **Error Handling**: Proper error handling throughout the plugin

## What it does

When activated, this plugin:

1. Shows a welcome notification
2. Sets up event listeners for app events
3. Stores the activation timestamp
4. Responds to custom events
5. Demonstrates storage usage

When deactivated, it:

1. Shows a goodbye notification
2. Cleans up event listeners
3. Stores the deactivation timestamp

## Development

### Building

```bash
npm run build
```

### Development Mode

```bash
npm run dev
```

### Testing

```bash
npm test
```

## Plugin Structure

```
hello-world/
├── src/
│   └── index.ts          # Main plugin implementation
├── dist/                 # Compiled output (generated)
├── package.json          # Node.js package configuration
├── tsconfig.json         # TypeScript configuration
├── plugin.json           # Plugin manifest
└── README.md            # This file
```

## API Usage Examples

### Notifications

```typescript
await context.app.showNotification('Hello World! 🎉');
```

### Events

```typescript
// Listen to events
context.events.on('app:ready', this.onAppReady.bind(this));

// Emit events
context.events.emit('hello:plugin-ready', { timestamp: new Date() });
```

### Storage

```typescript
// Store data
await context.storage.set('lastActivated', new Date().toISOString());

// Retrieve data
const lastActivated = await context.storage.get('lastActivated');
```

## Requirements

- Baize >= 0.1.0
- @baize/plugin-sdk

## Permissions

- `notifications`: Required to show user notifications
- `storage`: Required to store persistent data