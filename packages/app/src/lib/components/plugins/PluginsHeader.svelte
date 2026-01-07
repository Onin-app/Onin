<script lang="ts">
  /**
   * PluginsHeader Component
   *
   * 插件页面头部组件
   * 包含返回按钮、标题、搜索框、刷新和导入按钮
   */
  import { Button } from "bits-ui";
  import {
    ArrowLeft,
    MagnifyingGlass,
    ArrowClockwise,
    Plus,
  } from "phosphor-svelte";

  interface Props {
    searchQuery: string;
    onBack: () => void;
    onRefresh: () => void;
    onImport: () => void;
    onSearchChange: (query: string) => void;
  }

  let {
    searchQuery = $bindable(),
    onBack,
    onRefresh,
    onImport,
    onSearchChange,
  }: Props = $props();
</script>

<div
  class="flex items-center justify-between border-b border-neutral-200 px-4 py-3 dark:border-neutral-700"
  data-tauri-drag-region
>
  <div class="flex items-center gap-2">
    <Button.Root
      class="rounded p-1.5 hover:bg-neutral-200 dark:hover:bg-neutral-700"
      onclick={onBack}
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
        class="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-neutral-400"
      />
      <input
        type="text"
        bind:value={searchQuery}
        oninput={(e) => onSearchChange(e.currentTarget.value)}
        placeholder="搜索插件..."
        class="h-8 w-56 rounded border border-neutral-300 bg-white py-1.5 pl-9 pr-3 text-sm text-neutral-900 focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-900 dark:text-neutral-100 dark:focus:border-neutral-400"
      />
    </div>

    <!-- 刷新插件按钮 -->
    <Button.Root
      class="inline-flex h-8 items-center justify-center rounded bg-neutral-900 px-3 text-sm font-medium text-white hover:bg-neutral-800 active:scale-[0.98] dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-200"
      onclick={onRefresh}
    >
      <ArrowClockwise class="mr-1.5 h-3.5 w-3.5" />
      刷新
    </Button.Root>

    <!-- 手动导入插件按钮 -->
    <Button.Root
      class="inline-flex h-8 items-center justify-center rounded bg-neutral-900 px-3 text-sm font-medium text-white hover:bg-neutral-800 active:scale-[0.98] dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-200"
      onclick={onImport}
    >
      <Plus class="mr-1.5 h-3.5 w-3.5" />
      导入插件
    </Button.Root>
  </div>
</div>
