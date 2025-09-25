<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { Button } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";

  interface PluginManifest {
    id: string;
    name: string;
    version: string;
    description: string;
    entry: string;
  }

  import { goto } from "$app/navigation";
  import { escapeHandler } from "$lib/stores/escapeHandler";

  let searchQuery = $state("");
  let showAllPlugins = $state(true); // true: 全部, false: 已安装
  let plugins: PluginManifest[] = $state([]);

  const handleEsc = () => {
    goto("/");
  };

  onMount(async () => {
    // Register this page's ESC handler
    escapeHandler.set(handleEsc);

    try {
      const result = await invoke("load_plugins");
      plugins = result as PluginManifest[];
      console.log("Loaded plugins state:", plugins);
    } catch (error) {
      console.error("Failed to load plugins via invoke:", error);
    }
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

  const handleRefreshPlugins = async () => {
    try {
      console.log("正在刷新插件...");
      const result = await invoke("refresh_plugins");
      plugins = result as PluginManifest[];
      console.log("插件刷新成功:", plugins);
      
      // 显示成功通知
      await invoke("show_notification", {
        options: {
          title: "插件管理",
          body: `成功刷新 ${plugins.length} 个插件`,
        },
      });
    } catch (error) {
      console.error("刷新插件失败:", error);
      
      // 显示错误通知
      await invoke("show_notification", {
        options: {
          title: "插件管理",
          body: "刷新插件失败，请查看控制台了解详情",
        },
      });
    }
  };

  const handleImportPlugin = () => {
    // TODO: 实现手动导入插件功能
    console.log("手动导入插件");
  };

  const togglePluginView = () => {
    showAllPlugins = !showAllPlugins;
  };
  const executePlugin = async (pluginId: string) => {
    try {
      await invoke("execute_plugin_entry", { pluginId });
      console.log(`Successfully executed plugin with ID: ${pluginId}`);
    } catch (e) {
      console.error(`Failed to execute plugin with ID ${pluginId}:`, e);
    }
  };

  const showNotification = async () => {
    try {
      await invoke("show_notification", {
        options: {
          title: "来自 Tauri 的通知",
          body: "这是一个通过 invoke 调用的通知！",
        },
      });
      console.log("Notification sent successfully.");
    } catch (e) {
      console.error("Failed to send notification:", e);
    }
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

      <div class="flex items-center gap-3">
        <!-- 搜索框 -->
        <div class="relative">
          <svg
            class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-neutral-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="搜索插件..."
            class="bg-background text-foreground w-64 rounded border border-neutral-300 py-2 pr-4 pl-10 text-sm focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:focus:border-neutral-400"
          />
        </div>

        <!-- 全部/已安装切换 -->
        <div
          class="flex rounded border border-neutral-300 dark:border-neutral-600"
        >
          <Button.Root
            class="px-3 py-2 text-sm {showAllPlugins
              ? 'bg-neutral-200 text-neutral-900 dark:bg-neutral-600 dark:text-neutral-100'
              : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-700'}"
            onclick={() => (showAllPlugins = true)}
          >
            全部
          </Button.Root>
          <Button.Root
            class="px-3 py-2 text-sm {!showAllPlugins
              ? 'bg-neutral-200 text-neutral-900 dark:bg-neutral-600 dark:text-neutral-100'
              : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-700'}"
            onclick={() => (showAllPlugins = false)}
          >
            已安装
          </Button.Root>
        </div>

        <!-- 刷新插件按钮 -->
        <Button.Root
          class="rounded-input bg-green-500 text-white shadow-mini hover:bg-green-600 inline-flex h-9 items-center justify-center px-4 text-sm font-medium active:scale-[0.98] active:transition-all"
          onclick={handleRefreshPlugins}
        >
          <svg
            class="mr-2 h-4 w-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
          刷新插件
        </Button.Root>

        <!-- 手动导入插件按钮 -->
        <Button.Root
          class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex h-9 items-center justify-center px-4 text-sm font-medium active:scale-[0.98] active:transition-all"
          onclick={handleImportPlugin}
        >
          <svg
            class="mr-2 h-4 w-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 4v16m8-8H4"
            />
          </svg>
          手动导入
        </Button.Root>

        <!-- 显示通知按钮 -->
        <Button.Root
          class="rounded-input shadow-mini inline-flex h-9 items-center justify-center bg-blue-500 px-4 text-sm font-medium text-white hover:bg-blue-600 active:scale-[0.98] active:transition-all"
          onclick={showNotification}
        >
          显示通知
        </Button.Root>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Content Area -->
      <div class="flex-1 p-2">
        <div
          class="h-full overflow-auto rounded-lg border border-neutral-200 bg-white p-6 dark:border-neutral-700 dark:bg-neutral-900"
        >
          <!-- 这里是插件内容区域，你可以在这里填充具体内容 -->
          <div>
            {#if plugins.length > 0}
              <ul class="space-y-4">
                {#each plugins as plugin (plugin.id)}
                  <li
                    class="cursor-pointer rounded-lg border border-neutral-200 p-4 hover:bg-neutral-50 dark:border-neutral-700 dark:hover:bg-neutral-800"
                    on:click={() => executePlugin(plugin.id)}
                  >
                    <h3 class="text-lg font-semibold">{plugin.name}</h3>
                    <p class="text-sm text-neutral-500 dark:text-neutral-400">
                      {plugin.description}
                    </p>
                    <span
                      class="mt-2 inline-block rounded bg-neutral-200 px-2 py-1 text-xs dark:bg-neutral-700"
                      >{plugin.version}</span
                    >
                  </li>
                {/each}
              </ul>
            {:else}
              <div
                class="flex h-full items-center justify-center text-neutral-500"
              >
                <p>没有找到插件</p>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
</main>
