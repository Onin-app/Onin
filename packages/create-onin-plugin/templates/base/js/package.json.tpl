{
  "name": "__PACKAGE_NAME__",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "build": "node ./scripts/build.mjs",
    "pack:plugin": "npm run build && bestzip plugin.zip manifest.json icon.svg dist"
  },
  "dependencies": {
    "onin-sdk": "^0.0.3"
  },
  "devDependencies": {
    "bestzip": "^2.2.1"
  }
}
