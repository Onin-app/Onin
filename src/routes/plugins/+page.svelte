<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { Button, Tabs, Switch } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";
  import {
    ArrowLeft,
    MagnifyingGlass,
    ArrowClockwise,
    Plus,
    CheckCircle,
    Storefront,
    PuzzlePiece,
    Gear,
    Trash,
    Package,
    Star,
    Download,
    GithubLogo,
  } from "phosphor-svelte";
  import PluginSettings from "$lib/components/plugin/PluginSettings.svelte";
  import type { PluginSettingsSchema } from "$lib/types/plugin-settings";

  interface PluginManifest {
    id: string;
    name: string;
    version: string;
    description: string;
    entry: string;
    author?: string;
    downloads?: number;
    stars?: number;
    enabled?: boolean;
    settings?: PluginSettingsSchema;
  }

  import { goto } from "$app/navigation";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { listen } from "@tauri-apps/api/event";

  let searchQuery = $state("");
  let activeTab = $state("installed");
  let plugins: PluginManifest[] = $state([]);
  let currentSettingsPlugin: PluginManifest | null = $state(null);

  const handleEsc = () => {
    goto("/");
  };

  async function loadPlugins() {
    try {
      const result = await invoke("load_plugins");
      plugins = (result as PluginManifest[]).map((plugin) => ({
        ...plugin,
        // Mock 数据
        stars: plugin.stars ?? Math.floor(Math.random() * 1000),
        downloads: plugin.downloads ?? Math.floor(Math.random() * 10000),
        enabled: plugin.enabled ?? true,
      }));
      console.log("Loaded plugins state:", plugins);
    } catch (error) {
      console.error("Failed to load plugins via invoke:", error);
    }
  }

  onMount(async () => {
    escapeHandler.set(handleEsc);
    await loadPlugins();

    // Listen for settings schema registration events
    listen<string>(
      "plugin-settings-schema-registered",
      async (event) => {
        const pluginId = event.payload;
        console.log("[Plugins Page] Settings schema registered for plugin:", pluginId);

        // Refresh the specific plugin to get updated schema
        try {
          const updatedPlugin = await invoke<PluginManifest>(
            "get_plugin_with_schema",
            { pluginId },
          );
          
          console.log("[Plugins Page] Updated plugin data:", updatedPlugin);
          console.log("[Plugins Page] Has settings:", updatedPlugin.settings);
          console.log("[Plugins Page] Settings fields:", updatedPlugin.settings?.fields);
          
          plugins = plugins.map((p) =>
            p.id === pluginId ? { ...p, ...updatedPlugin } : p,
          );
          
          console.log("[Plugins Page] Plugins list updated");
        } catch (error) {
          console.error("[Plugins Page] Failed to refresh plugin schema:", error);
        }
      },
    );
  });

  onDestroy(() => {
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(() => {});
    }
  });

  const handleBackToSettings = () => {
    goto("/settings");
  };

  const handleRefreshPlugins = async () => {
    try {
      console.log("正在刷新插件...");
      const result = await invoke("refresh_plugins");
      plugins = (result as PluginManifest[]).map((plugin) => ({
        ...plugin,
        // Mock 数据
        stars: plugin.stars ?? Math.floor(Math.random() * 1000),
        downloads: plugin.downloads ?? Math.floor(Math.random() * 10000),
      }));
      console.log("插件刷新成功:", plugins);

      await invoke("show_notification", {
        options: {
          title: "刷新成功",
          body: `已刷新 ${plugins.length} 个插件`,
        },
      });
    } catch (error) {
      console.error("刷新插件失败:", error);

      await invoke("show_notification", {
        options: {
          title: "刷新失败",
          body: "无法刷新插件列表，请重试",
        },
      });
    }
  };

  const handleImportPlugin = () => {
    console.log("手动导入插件");
  };

  const executePlugin = async (pluginId: string) => {
    try {
      await invoke("execute_plugin_entry", { pluginId });
      console.log(`Successfully executed plugin with ID: ${pluginId}`);
    } catch (e) {
      console.error(`Failed to execute plugin with ID ${pluginId}:`, e);
    }
  };

  const togglePlugin = async (pluginId: string, enabled: boolean) => {
    // 找到插件名称
    const plugin = plugins.find((p) => p.id === pluginId);
    const pluginName = plugin?.name || pluginId;

    try {
      await invoke("toggle_plugin", { pluginId, enabled });

      // 更新本地状态
      plugins = plugins.map((p) => (p.id === pluginId ? { ...p, enabled } : p));

      console.log(
        `Plugin ${pluginId} is now ${enabled ? "enabled" : "disabled"}`,
      );

      // 刷新命令列表，使搜索框和指令设置中的插件命令也更新
      try {
        await invoke("refresh_commands");
        console.log("Commands refreshed after plugin toggle");
      } catch (refreshError) {
        console.error("Failed to refresh commands:", refreshError);
      }

      await invoke("show_notification", {
        options: {
          title: enabled ? "插件已启用" : "插件已禁用",
          body: `${pluginName} ${enabled ? "已启用" : "已禁用"}`,
        },
      });
    } catch (error) {
      console.error("Failed to toggle plugin:", error);

      await invoke("show_notification", {
        options: {
          title: "操作失败",
          body: `无法${enabled ? "启用" : "禁用"}插件 ${pluginName}`,
        },
      });
    }
  };

  const uninstallPlugin = async (pluginId: string) => {
    console.log(`Uninstall plugin ${pluginId}`);
    // TODO: 实现插件卸载功能
  };

  const openPluginSettings = (plugin: PluginManifest) => {
    currentSettingsPlugin = plugin;
  };

  const closePluginSettings = () => {
    currentSettingsPlugin = null;
  };

  const filteredPlugins = $derived(
    plugins.filter((plugin) =>
      plugin.name.toLowerCase().includes(searchQuery.toLowerCase()),
    ),
  );
</script>

{#if currentSettingsPlugin && currentSettingsPlugin.settings}
  <PluginSettings
    pluginId={currentSettingsPlugin.id}
    pluginName={currentSettingsPlugin.name}
    schema={currentSettingsPlugin.settings}
    onback={closePluginSettings}
  />
{:else}
  <main
    class="flex h-[100vh] w-full flex-col bg-neutral-100 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
    data-tauri-drag-region
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between border-b border-neutral-200 px-4 py-3 dark:border-neutral-700"
    >
      <div class="flex items-center gap-2">
        <Button.Root
          class="rounded p-1.5 hover:bg-neutral-200 dark:hover:bg-neutral-700"
          onclick={handleBackToSettings}
          aria-label="返回设置"
        >
          <ArrowLeft class="h-5 w-5" />
        </Button.Root>
        <h2 class="text-lg font-semibold">插件管理</h2>
      </div>

      <div class="flex items-center gap-2">
      <!-- 搜索框 -->
      <div class="relative">
        <MagnifyingGlass
          class="absolute top-1/2 left-2.5 h-4 w-4 -translate-y-1/2 text-neutral-400"
        />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="搜索插件..."
          class="h-8 w-56 rounded border border-neutral-300 bg-white py-1.5 pr-3 pl-9 text-sm text-neutral-900 focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-900 dark:text-neutral-100 dark:focus:border-neutral-400"
        />
      </div>

      <!-- 刷新插件按钮 -->
      <Button.Root
        class="inline-flex h-8 items-center justify-center rounded bg-neutral-900 px-3 text-sm font-medium text-white hover:bg-neutral-800 active:scale-[0.98] dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-200"
        onclick={handleRefreshPlugins}
      >
        <ArrowClockwise class="mr-1.5 h-3.5 w-3.5" />
        刷新
      </Button.Root>

        <!-- 手动导入插件按钮 -->
        <Button.Root
          class="inline-flex h-8 items-center justify-center rounded bg-neutral-900 px-3 text-sm font-medium text-white hover:bg-neutral-800 active:scale-[0.98] dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-200"
          onclick={handleImportPlugin}
        >
          <Plus class="mr-1.5 h-3.5 w-3.5" />
          导入插件
        </Button.Root>
      </div>
    </div>

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

      <Tabs.Content value="installed" class="flex-1 overflow-auto">
        {#if filteredPlugins.length > 0}
          <div class="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
            {#each filteredPlugins as plugin (plugin.id)}
              <div
                class="group flex flex-col rounded-lg border border-neutral-200 bg-white p-3 transition-all hover:border-neutral-300 hover:shadow-sm dark:border-neutral-700 dark:bg-neutral-900 dark:hover:border-neutral-600"
              >
                <!-- 顶部：图标和信息（左右结构） -->
                <div class="mb-2 flex items-start gap-3">
                  <!-- 左侧图标 -->
                  <button
                    class="flex h-16 w-16 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-neutral-100 to-neutral-200 transition-transform hover:scale-105 dark:from-neutral-800 dark:to-neutral-700"
                    onclick={() => executePlugin(plugin.id)}
                  >
                    <PuzzlePiece class="h-8 w-8" />
                  </button>

                  <!-- 右侧信息 -->
                  <div class="flex min-w-0 flex-1 flex-col">
                    <div class="mb-1 flex items-start justify-between gap-2">
                      <h3
                        class="truncate text-base leading-tight font-semibold"
                      >
                        {plugin.name}
                      </h3>
                      <Button.Root
                        class="shrink-0 rounded p-1 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-neutral-100 dark:hover:bg-neutral-800"
                        onclick={(e: MouseEvent) => {
                          e.stopPropagation();
                          console.log("Open GitHub for", plugin.id);
                        }}
                        aria-label="查看 GitHub"
                      >
                        <GithubLogo class="h-4 w-4" />
                      </Button.Root>
                    </div>
                    <p
                      class="line-clamp-2 text-sm text-neutral-500 dark:text-neutral-400"
                    >
                      {plugin.description}
                    </p>
                  </div>
                </div>

                <!-- 作者和 ID -->
                <div
                  class="mb-2 flex items-center gap-2 text-xs text-neutral-400"
                >
                  {#if plugin.author}
                    <span class="truncate">{plugin.author}</span>
                  {/if}
                  <span class="text-neutral-300 dark:text-neutral-600">
                    ID: {plugin.id}
                  </span>
                </div>

                <!-- 底部：统计和操作 -->
                <div
                  class="flex items-center justify-between border-t border-neutral-200 pt-2 dark:border-neutral-700"
                >
                  <!-- 左侧：收藏和下载数 -->
                  <div class="flex items-center gap-3 text-xs text-neutral-500">
                    <div class="flex items-center gap-1">
                      <Star class="h-3.5 w-3.5" />
                      <span>{plugin.stars ?? 0}</span>
                    </div>
                    <div class="flex items-center gap-1">
                      <Download class="h-3.5 w-3.5" />
                      <span>{plugin.downloads ?? 0}</span>
                    </div>
                  </div>

                  <!-- 右侧：操作按钮 -->
                  <div class="flex items-center gap-1">
                    <!-- 启用/禁用开关 -->
                    <Switch.Root
                      checked={plugin.enabled !== false}
                      onCheckedChange={(checked) => {
                        togglePlugin(plugin.id, checked);
                      }}
                      class="focus-visible:ring-foreground focus-visible:ring-offset-background data-[state=checked]:bg-foreground data-[state=unchecked]:bg-dark-10 data-[state=unchecked]:shadow-mini-inset dark:data-[state=checked]:bg-foreground peer inline-flex h-[20px] min-h-[20px] w-[36px] shrink-0 cursor-pointer items-center rounded-full px-[2px] transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      <Switch.Thumb
                        class="bg-background data-[state=unchecked]:shadow-mini dark:border-background/30 dark:bg-foreground dark:shadow-popover pointer-events-none block size-[16px] shrink-0 rounded-full transition-transform data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0 dark:border dark:data-[state=unchecked]:border"
                      />
                    </Switch.Root>

                    <!-- 设置按钮 -->
                    {#if plugin.settings && plugin.settings.fields.length > 0}
                      <Button.Root
                        class="rounded px-2 py-1 text-xs transition-colors hover:bg-neutral-100 dark:hover:bg-neutral-800"
                        onclick={(e: MouseEvent) => {
                          e.stopPropagation();
                          openPluginSettings(plugin);
                        }}
                        aria-label="插件设置"
                      >
                        <Gear class="h-4 w-4" />
                      </Button.Root>
                    {/if}

                    <!-- 卸载按钮 -->
                    <Button.Root
                      class="rounded px-2 py-1 text-xs transition-colors hover:bg-red-100 hover:text-red-600 dark:hover:bg-red-900/20 dark:hover:text-red-400"
                      onclick={(e: MouseEvent) => {
                        e.stopPropagation();
                        uninstallPlugin(plugin.id);
                      }}
                      aria-label="卸载插件"
                    >
                      <Trash class="h-4 w-4" />
                    </Button.Root>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div
            class="flex h-full flex-col items-center justify-center text-neutral-500"
          >
            <Package class="mb-4 h-12 w-12 opacity-50" />
            <p class="text-lg">没有找到插件</p>
            <p class="mt-2 text-sm">尝试导入插件或调整搜索条件</p>
          </div>
        {/if}
      </Tabs.Content>

      <Tabs.Content value="market" class="flex-1 overflow-auto">
        <div
          class="flex h-full flex-col items-center justify-center text-neutral-500"
        >
          <Storefront class="mb-4 h-12 w-12 opacity-50" />
          <p class="text-lg">插件市场即将推出</p>
          <p class="mt-2 text-sm">敬请期待更多精彩插件</p>
        </div>
      </Tabs.Content>
      </Tabs.Root>
    </div>
  </main>
{/if}
