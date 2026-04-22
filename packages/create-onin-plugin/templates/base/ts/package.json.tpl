{
  "name": "__PACKAGE_NAME__",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "build:background": "vite build --config vite.background.config.ts",
    "build": "npm run build:index && npm run build:background",
    "pack:plugin": "npm run build && bestzip plugin.zip manifest.json icon.svg dist"
  },
  "dependencies": {
    "onin-sdk": "^1.13.0"
  },
  "devDependencies": {
    "bestzip": "^2.3.0"
  }
}
