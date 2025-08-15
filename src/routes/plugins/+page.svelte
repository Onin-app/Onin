<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { Button } from "bits-ui";

  import { goto } from "$app/navigation";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { pluginStore } from "$lib/stores/pluginStore";
  import PluginList from "$lib/components/plugins/PluginList.svelte";
  import PluginSearch from "$lib/components/plugins/PluginSearch.svelte";
  import PluginDetailsModal from "$lib/components/plugins/PluginDetailsModal.svelte";
  import PluginErrorBoundary from "$lib/components/plugins/PluginErrorBoundary.svelte";
  import type { PluginInfo } from "$lib/types/plugin";

  let searchQuery = $state("");
  let showAllPlugins = $state(true); // true: 全部, false: 已安装
  let selectedPlugin = $state<PluginInfo | null>(null);
  let showDetailsModal = $state(false);

  // Subscribe to plugin store
  const plugins = pluginStore.plugins;
  const loading = pluginStore.loading;
  const pluginErrors = pluginStore.pluginErrors;

  const handleEsc = () => {
    console.log("Plugins page ESC handler executing");
    goto("/");
  };

  onMount(() => {
    console.log("Plugins component has mounted");
    // Register this page's ESC handler
    escapeHandler.set(handleEsc);
    // Load plugins using store action
    pluginStore.actions.loadPlugins();
  });

  onDestroy(() => {
    // On destroy, reset the handler if it's still ours
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(() => {});
    }
  });

  const handleBackToSettings = () => {
    goto("/settings");
  };

  const handleImportPlugin = async () => {
    // TODO: 实现文件选择对话框
    console.log("手动导入插件");
    // For now, just refresh the plugin list
    await pluginStore.actions.refreshPlugins();
  };

  const handleTogglePlugin = async (event: CustomEvent<{ plugin: PluginInfo }>) => {
    const { plugin } = event.detail;
    await pluginStore.actions.togglePlugin(plugin.name);
  };

  const handleViewDetails = (event: CustomEvent<{ plugin: PluginInfo }>) => {
    selectedPlugin = event.detail.plugin;
    showDetailsModal = true;
  };

  const handleSaveConfig = async (event: CustomEvent<{ plugin: PluginInfo; config: Record<string, any> }>) => {
    const { plugin, config } = event.detail;
    try {
      // TODO: 调用 Tauri 命令保存插件配置
      // await invoke('save_plugin_config', { name: plugin.name, config });
      console.log(`Saving config for plugin ${plugin.name}:`, config);
    } catch (error) {
      console.error("Failed to save plugin config:", error);
    }
  };

  const handleClearError = (event: CustomEvent<{ plugin: string }>) => {
    pluginStore.actions.clearPluginError(event.detail.plugin);
  };

  const handleClearAllErrors = () => {
    pluginStore.actions.clearErrors();
  };

  const handleRetryPlugin = async (event: CustomEvent<{ plugin: string }>) => {
    const { plugin } = event.detail;
    await pluginStore.actions.togglePlugin(plugin);
  };

  const handleRefresh = async () => {
    await pluginStore.actions.refreshPlugins();
  };
</script>

<main
  class="flex h-[100vh] w-full bg-neutral-100 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
  data-tauri-drag-region
>
  <div class="flex h-full w-full flex-col">
    <!-- Header -->
    <div
      class="flex items-center justify-between border-b border-neutral-200 p-4 dark:border-neutral-700"
    >
      <div class="flex items-center gap-3">
        <Button.Root
          class="rounded p-2 hover:bg-neutral-200 dark:hover:bg-neutral-700"
          onclick={handleBackToSettings}
          aria-label="返回设置"
        >
          <svg
            class="h-5 w-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 19l-7-7 7-7"
            />
          </svg>
        </Button.Root>
        <h2 class="text-xl font-semibold">插件管理</h2>
      </div>

      <PluginSearch 
        bind:searchQuery 
        bind:showAllPlugins 
        on:importPlugin={handleImportPlugin}
      />
    </div>

    <!-- Main Content Area -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Content Area -->
      <div class="flex-1 p-2">
        <div
          class="h-full rounded-lg border border-neutral-200 bg-white p-6 dark:border-neutral-700 dark:bg-neutral-900 overflow-y-auto"
        >
          {#if $loading}
            <div class="flex h-full items-center justify-center">
              <div class="flex items-center gap-3 text-neutral-500">
                <svg class="h-6 w-6 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                </svg>
                <span>正在加载插件...</span>
              </div>
            </div>
          {:else}
            <!-- Error display -->
            <PluginErrorBoundary 
              errors={$pluginErrors}
              on:clearError={handleClearError}
              on:clearAllErrors={handleClearAllErrors}
              on:retryPlugin={handleRetryPlugin}
            />

            <!-- Refresh button -->
            <div class="mb-4 flex justify-end">
              <Button.Root
                class="rounded px-3 py-1 text-sm font-medium transition-colors hover:bg-neutral-100 dark:hover:bg-neutral-800"
                onclick={handleRefresh}
              >
                <svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                </svg>
                刷新
              </Button.Root>
            </div>

            <PluginList 
              plugins={$plugins} 
              {searchQuery} 
              {showAllPlugins}
              on:togglePlugin={handleTogglePlugin}
              on:viewDetails={handleViewDetails}
            />
          {/if}
        </div>
      </div>
    </div>
  </div>
</main>

<!-- Plugin Details Modal -->
<PluginDetailsModal 
  plugin={selectedPlugin}
  bind:open={showDetailsModal}
  on:togglePlugin={handleTogglePlugin}
  on:saveConfig={handleSaveConfig}
/>
