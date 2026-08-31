<script lang="ts">
  /**
   * PluginsHeader Component
   *
   * 插件页面头部组件
   * 包含返回按钮、标题、搜索框、刷新和导入按钮
   */
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
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
  class="flex items-center justify-between border-b px-4 py-3"
  data-tauri-drag-region
>
  <div class="flex items-center gap-2">
    <Button
      variant="ghost"
      size="icon"
      class="h-8 w-8"
      onclick={onBack}
      aria-label="返回设置"
    >
      <ArrowLeft class="h-4 w-4" />
    </Button>
    <h2 class="text-foreground text-base font-semibold">插件管理</h2>
  </div>

  <div class="flex items-center gap-2">
    <!-- 搜索框 -->
    <div class="relative">
      <MagnifyingGlass
        class="text-muted-foreground absolute top-1/2 left-2.5 h-4 w-4 -translate-y-1/2"
      />
      <Input
        type="text"
        bind:value={searchQuery}
        oninput={(e) => onSearchChange(e.currentTarget.value)}
        placeholder="搜索插件..."
        class="h-8 w-56 pl-9 text-xs"
      />
    </div>

    <!-- 刷新插件按钮 -->
    <Button
      variant="outline"
      size="sm"
      class="h-8 gap-1.5 text-xs"
      onclick={onRefresh}
    >
      <ArrowClockwise class="h-3.5 w-3.5" />
      刷新
    </Button>

    <!-- 手动导入插件按钮 -->
    <Button size="sm" class="h-8 gap-1.5 text-xs" onclick={onImport}>
      <Plus class="h-3.5 w-3.5" />
      导入插件
    </Button>
  </div>
</div>
