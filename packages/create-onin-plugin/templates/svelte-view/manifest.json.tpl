{
  "id": "__PLUGIN_ID__",
  "name": "__PLUGIN_NAME__",
  "version": "0.1.0",
  "description": "__PLUGIN_DESCRIPTION__",
  "entry": "dist/index.html",
  "icon": "icon.svg",
  "type": "webview",
  "display_mode": "inline",
  "lifecycle": "dist/lifecycle.js",
  "commands": [
    {
      "code": "open",
      "name": "Open __PLUGIN_NAME__",
      "description": "Open the plugin UI",
      "keywords": [
        {
          "name": "__KEYWORD__",
          "type": "prefix"
        }
      ]
    }
  ]
}
