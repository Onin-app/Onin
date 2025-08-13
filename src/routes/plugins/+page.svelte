<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { Button } from "bits-ui";

  import { goto } from "$app/navigation";
  import { escapeHandler } from "$lib/stores/escapeHandler";

  let searchQuery = $state("");
  let showAllPlugins = $state(true); // true: 全部, false: 已安装

  const handleEsc = () => {
    console.log("Plugins page ESC handler executing");
    goto("/");
  };

  onMount(() => {
    console.log("Plugins component has mounted");
    // Register this page's ESC handler
    escapeHandler.set(handleEsc);
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

  const handleImportPlugin = () => {
    // TODO: 实现手动导入插件功能
    console.log("手动导入插件");
  };

  const togglePluginView = () => {
    showAllPlugins = !showAllPlugins;
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
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Content Area -->
      <div class="flex-1 p-2">
        <div
          class="h-full rounded-lg border border-neutral-200 bg-white p-6 dark:border-neutral-700 dark:bg-neutral-900"
        >
          <!-- 这里是插件内容区域，你可以在这里填充具体内容 -->
          <div class="flex h-full items-center justify-center text-neutral-500">
            <p>插件内容区域 - 请在这里添加你的插件管理功能</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</main>
