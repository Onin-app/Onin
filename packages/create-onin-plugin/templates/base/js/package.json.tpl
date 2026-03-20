{
  "name": "__PACKAGE_NAME__",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "build:lifecycle": "vite build --config vite.lifecycle.config.js",
    "build": "npm run build:index && npm run build:lifecycle",
    "pack:plugin": "npm run build && bestzip plugin.zip manifest.json icon.svg dist"
  },
  "dependencies": {
    "onin-sdk": "^1.6.0"
  },
  "devDependencies": {
    "bestzip": "^2.2.1"
  }
}
