<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { PluginManifest } from '$lib/type';

  let plugins: PluginManifest[] = [];
  let loadError: string | null = null;

  onMount(async () => {
    await loadPlugins();
  });

  async function loadPlugins() {
    try {
      loadError = null;
      plugins = await invoke('get_installed_plugins');
    } catch (e) {
      console.error('Failed to load plugins', e);
      loadError = '加载插件失败，请检查控制台日志';
    }
  }

  async function installPlugin() {
    try {
      await invoke('install_plugin');
      await loadPlugins();
    } catch (e) {
      console.error('Failed to install plugin', e);
    }
  }

  async function uninstallPlugin(pluginId: string) {
    try {
      await invoke('uninstall_plugin', { pluginId });
      plugins = plugins.filter(p => p.id !== pluginId);
    } catch (e) {
      console.error('Failed to uninstall plugin', e);
    }
  }

  async function openPluginConfig(pluginId: string) {
    try {
      await invoke('open_plugin_config', { pluginId });
    } catch (e) {
      console.error('Failed to open plugin config', e);
    }
  }
</script>

<div class="p-4">
  <div class="flex justify-between items-center mb-4">
    <h1 class="text-2xl font-bold">插件管理</h1>
    <button on:click={installPlugin} class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
      安装插件
    </button>
  </div>

  {#if loadError}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
      <p>{loadError}</p>
    </div>
  {/if}

  {#if plugins.length === 0 && !loadError}
    <div class="bg-gray-100 border border-gray-300 px-4 py-3 rounded mb-4">
      <p>未安装任何插件</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each plugins as plugin}
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
          <h2 class="text-lg font-semibold">{plugin.name}</h2>
          <p class="text-gray-600 dark:text-gray-400">{plugin.description}</p>
          <div class="mt-4 flex justify-between items-center">
            <span class="text-sm text-gray-500">{plugin.version}</span>
            <div>
              <button on:click={() => openPluginConfig(plugin.id)} class="text-blue-500 hover:underline mr-2">配置</button>
              <button on:click={() => uninstallPlugin(plugin.id)} class="text-red-500 hover:underline">卸载</button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>