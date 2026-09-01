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
  import { ScrollArea } from "$lib/components/ui/scroll-area";

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

<div class="h-[100vh] w-full bg-transparent p-0 select-none">
  <main
    class="border-border/70 bg-background text-foreground relative flex h-full w-full flex-col overflow-hidden rounded-2xl border ring-1 ring-white/10 dark:ring-white/5"
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
            class="bg-muted/50 border-border/40 mb-3 inline-flex w-fit items-center gap-1 rounded-xl border p-1"
          >
            <Tabs.Trigger
              value="installed"
              class="text-muted-foreground hover:text-foreground data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:border-border/50 inline-flex cursor-pointer items-center justify-center rounded-lg px-3 py-1.5 text-xs font-medium transition-[transform,background-color,color,box-shadow] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-[0.97] data-[state=active]:border data-[state=active]:shadow-2xs"
            >
              <CheckCircle class="mr-1.5 h-3.5 w-3.5" />
              已安装
            </Tabs.Trigger>
            <Tabs.Trigger
              value="market"
              class="text-muted-foreground hover:text-foreground data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:border-border/50 inline-flex cursor-pointer items-center justify-center rounded-lg px-3 py-1.5 text-xs font-medium transition-[transform,background-color,color,box-shadow] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-[0.97] data-[state=active]:border data-[state=active]:shadow-2xs"
            >
              <Storefront class="mr-1.5 h-3.5 w-3.5" />
              插件市场
            </Tabs.Trigger>
          </Tabs.List>

          <Tabs.Content value="installed" class="flex-1 overflow-hidden">
            <ScrollArea
              class="h-full w-full"
              viewportClass="h-full w-full overflow-x-hidden pr-2"
            >
              {#if pluginList.filteredPlugins.length > 0}
                <div
                  class="grid grid-cols-1 gap-2.5 pb-4 md:grid-cols-2 xl:grid-cols-3"
                >
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
            </ScrollArea>
          </Tabs.Content>

          <Tabs.Content value="market" class="flex-1 overflow-hidden">
            {#await import("$lib/components/marketplace/MarketplaceView.svelte")}
              <div class="flex h-full items-center justify-center">
                <div class="text-muted-foreground text-xs">加载市场中...</div>
              </div>
            {:then { default: MarketplaceView }}
              <MarketplaceView
                active={activeTab === "market"}
                refreshTrigger={marketRefreshTrigger}
              />
            {:catch error}
              <div
                class="text-muted-foreground flex h-full flex-col items-center justify-center"
              >
                <Storefront class="mb-3 h-10 w-10 opacity-40" />
                <p class="text-foreground text-sm font-medium">
                  插件市场加载失败
                </p>
                <p class="text-muted-foreground mt-1 text-xs">
                  {error.message}
                </p>
              </div>
            {/await}
          </Tabs.Content>
        </Tabs.Root>
      </div>
    {/if}
  </main>
</div>
