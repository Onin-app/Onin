# @baize/plugin-sdk

SDK for developing Baize plugins.

## Installation

```bash
npm install @baize/plugin-sdk
```

## Usage

```typescript
import { Plugin, PluginContext } from '@baize/plugin-sdk';

export default class MyPlugin implements Plugin {
  async activate(context: PluginContext): Promise<void> {
    // Plugin activation logic
    await context.app.showNotification('Plugin activated!');
  }

  async deactivate(): Promise<void> {
    // Plugin deactivation logic
  }
}
```

## API Reference

Coming soon...