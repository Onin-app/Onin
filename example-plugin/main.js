import { invoke } from '@tauri-apps/api/core';

// This is a simplified version of the plugin API for demonstration purposes.
// A real plugin would use a more robust and feature-rich API provided by the host application.
const pluginApi = {
  async readFile(path) {
    return await invoke('plugin:__internal_api__', {
      plugin_id: 'com.example.myplugin',
      method: 'readFile',
      params: { path },
    });
  },
};

document.addEventListener('DOMContentLoaded', () => {
  const readFileBtn = document.getElementById('read-file-btn');
  const fileContentEl = document.getElementById('file-content');

  readFileBtn.addEventListener('click', async () => {
    try {
      // For this example, we'll try to read the plugin's own manifest file.
      // A real plugin would likely read a user-specified file.
      const content = await pluginApi.readFile('plugins/com.example.myplugin/manifest.json');
      fileContentEl.textContent = content;
    } catch (error) {
      fileContentEl.textContent = `Error: ${error}`;
    }
  });
});