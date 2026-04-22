<script lang="ts">
  /**
   * Plugins Page
   *
   * 插件管理页面 - 重构后版本
   * 使用 composables 和提取的组件实现关注点分离
   */
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { Tabs } from "bits-ui";
  import { CheckCircle, Storefront } from "phosphor-svelte";
  import { goto } from "$app/navigation";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";

  // Composable
  import {
    usePluginList,
    type PluginManifest,
  } from "$lib/composables/usePluginList.svelte";

  // Components
  import PluginSettings from "$lib/components/plugins/PluginSettings.svelte";
  import InstalledPluginDetail from "$lib/components/marketplace/InstalledPluginDetail.svelte";
  import PluginCard from "$lib/components/plugins/PluginCard.svelte";
  import PluginsHeader from "$lib/components/plugins/PluginsHeader.svelte";
  import EmptyPluginState from "$lib/components/plugins/EmptyPluginState.svelte";

  // ===== Composable =====
  const pluginList = usePluginList();

  // ===== Local State =====
  let activeTab = $state("installed");
  let currentSettingsPlugin: PluginManifest | null = $state(null);
  let detailDialogOpen = $state(false);
  let selectedPluginForDetail: string | null = $state(null);
  let marketRefreshTrigger = $state(0);
  let unlisten = $state<null | (() => void)>(null);

  // ===== Event Handlers =====
  const handleEsc = () => {
    goto("/");
  };

  const handleBackToSettings = () => {
    goto("/settings");
  };

  const openPluginSettings = (plugin: PluginManifest) => {
    currentSettingsPlugin = plugin;
  };

  const closePluginSettings = () => {
    currentSettingsPlugin = null;
  };
  
  const handleRefresh = () => {
    if (activeTab === "market") {
      marketRefreshTrigger++;
    } else {
      pluginList.refreshPlugins();
    }
  };

  const openPluginDetail = (pluginId: string) => {
    selectedPluginForDetail = pluginId;
    detailDialogOpen = true;
  };

  const handleDetailDialogOpenChange = (open: boolean) => {
    detailDialogOpen = open;
    if (!open) {
      selectedPluginForDetail = null;
    }
  };

  // ===== Lifecycle =====
  onMount(async () => {
    escapeHandler.set(handleEsc);
    await pluginList.loadPlugins(false);
    unlisten = await pluginList.setupListeners();
  });

  onDestroy(() => {
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(null);
    }
    if (unlisten) {
      unlisten();
    }
  });
</script>

{#if selectedPluginForDetail}
  <InstalledPluginDetail
    bind:open={detailDialogOpen}
    pluginId={selectedPluginForDetail}
    onOpenChange={handleDetailDialogOpenChange}
  />
{/if}

<div class="h-[100vh] w-full bg-transparent p-1">
  <main
    class="flex h-full w-full flex-col overflow-hidden rounded-xl bg-neutral-100 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
    data-tauri-drag-region
  >
    {#if currentSettingsPlugin && currentSettingsPlugin.settings}
      <PluginSettings
        pluginId={currentSettingsPlugin.id}
        pluginName={currentSettingsPlugin.name}
        schema={currentSettingsPlugin.settings}
        onback={closePluginSettings}
      />
    {:else}
      <!-- Header -->
      <PluginsHeader
        bind:searchQuery={pluginList.state.searchQuery}
        onBack={handleBackToSettings}
        onRefresh={handleRefresh}
        onImport={pluginList.importPlugin}
        onSearchChange={pluginList.setSearchQuery}
      />

      <!-- Tabs Content -->
      <div class="flex-1 overflow-hidden px-4 py-3">
        <Tabs.Root bind:value={activeTab} class="flex h-full flex-col">
          <Tabs.List
            class="mb-3 inline-flex items-center gap-1 border-b border-neutral-200 dark:border-neutral-700"
          >
            <Tabs.Trigger
              value="installed"
              class="inline-flex items-center justify-center border-b-2 border-transparent px-3 py-2 text-sm font-medium text-neutral-600 transition-colors hover:text-neutral-900 data-[state=active]:border-neutral-900 data-[state=active]:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-100 dark:data-[state=active]:border-neutral-100 dark:data-[state=active]:text-neutral-100"
            >
              <CheckCircle class="mr-1.5 h-4 w-4" />
              已安装
            </Tabs.Trigger>
            <Tabs.Trigger
              value="market"
              class="inline-flex items-center justify-center border-b-2 border-transparent px-3 py-2 text-sm font-medium text-neutral-600 transition-colors hover:text-neutral-900 data-[state=active]:border-neutral-900 data-[state=active]:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-100 dark:data-[state=active]:border-neutral-100 dark:data-[state=active]:text-neutral-100"
            >
              <Storefront class="mr-1.5 h-4 w-4" />
              插件市场
            </Tabs.Trigger>
          </Tabs.List>

          <Tabs.Content value="installed" class="flex-1 overflow-hidden">
            <AppScrollArea
              class="h-full w-full"
              viewportClass="h-full w-full overflow-x-hidden pr-2"
            >
              {#if pluginList.filteredPlugins.length > 0}
                <div class="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
                  {#each pluginList.filteredPlugins as plugin (plugin.dir_name || plugin.id)}
                    <PluginCard
                      {plugin}
                      imageErrors={pluginList.state.imageErrors}
                      onExecute={pluginList.executePlugin}
                      onToggle={pluginList.togglePlugin}
                      onSettings={openPluginSettings}
                      onUninstall={pluginList.uninstallPlugin}
                      onViewDetail={openPluginDetail}
                      onImageError={pluginList.handleImageError}
                    />
                  {/each}
                </div>
              {:else}
                <EmptyPluginState />
              {/if}
            </AppScrollArea>
          </Tabs.Content>

          <Tabs.Content value="market" class="flex-1 overflow-hidden">
            {#await import("$lib/components/marketplace/MarketplaceView.svelte")}
              <div class="flex h-full items-center justify-center">
                <div class="text-neutral-500">加载中...</div>
              </div>
            {:then { default: MarketplaceView }}
              <MarketplaceView 
                active={activeTab === "market"} 
                refreshTrigger={marketRefreshTrigger}
              />
            {:catch error}
              <div
                class="flex h-full flex-col items-center justify-center text-neutral-500"
              >
                <Storefront class="mb-4 h-12 w-12 opacity-50" />
                <p class="text-lg">插件市场加载失败</p>
                <p class="mt-2 text-sm">{error.message}</p>
              </div>
            {/await}
          </Tabs.Content>
        </Tabs.Root>
      </div>
    {/if}
  </main>
</div>
