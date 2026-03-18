// Legacy example kept for reference only.
// New plugins should prefer a single source declaration:
// - src/plugin.ts exports background and ui
// - src/background.ts registers background
// - src/main.ts mounts ui
// - scripts/build.mjs emits both dist/index.html and dist/lifecycle.js
