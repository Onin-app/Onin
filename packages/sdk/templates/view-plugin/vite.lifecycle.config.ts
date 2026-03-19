// Legacy example kept for reference only.
// New plugins should prefer a single source declaration:
// - src/plugin.ts exports setup and mount
// - src/main.ts mounts the plugin via mountPlugin(plugin, target)
// - scripts/build.mjs emits both dist/index.html and dist/background.js
