{
  "name": "__PACKAGE_NAME__",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build:index": "vite build",
    "build:lifecycle": "vite build --config vite.lifecycle.config.ts",
    "build": "npm run build:index && npm run build:lifecycle",
    "pack:plugin": "npm run build && bestzip plugin.zip manifest.json icon.svg dist"
  },
  "dependencies": {
    "onin-sdk": "^1.6.0",
    "svelte": "^5.0.0"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^6.2.4",
    "bestzip": "^2.2.1",
    "typescript": "^5.5.0",
    "vite": "^7.3.1"
  }
}
