<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { MagnifyingGlass, Funnel, Package } from "phosphor-svelte";
  import { Button } from "bits-ui";
  import PluginCard from "./PluginCard.svelte";
  import { fetchPlugins } from "$lib/api/marketplace";
  import type { MarketplacePlugin } from "$lib/types/marketplace";

  let plugins: MarketplacePlugin[] = $state([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let searchQuery = $state("");
  let selectedCategory = $state<string>("all");
  let page = $state(1);
  let total = $state(0);
  const limit = 20;

  const categories = [
    { value: "all", label: "全部" },
    { value: "productivity", label: "效率工具" },
    { value: "utility", label: "实用工具" },
    { value: "development", label: "开发工具" },
    { value: "entertainment", label: "娱乐" },
    { value: "other", label: "其他" },
  ];



  async function loadPlugins() {
    loading = true;
    error = null;

    try {
      const response = await fetchPlugins({
        page,
        limit,
        category: selectedCategory === "all" ? undefined : selectedCategory,
        keyword: searchQuery || undefined,
      });

      plugins = response.data;
      total = response.meta.total;
    } catch (e) {
      console.error("Failed to load marketplace plugins:", e);
      error = e instanceof Error ? e.message : "加载失败";
      
      // 如果 API 不可用，显示提示
      if (error.includes("fetch")) {
        error = "插件市场服务暂时不可用，请稍后再试";
      }
    } finally {
      loading = false;
    }
  }

  async function handlePluginClick(plugin: MarketplacePlugin) {
    // TODO: 打开插件详情页
    console.log("Open plugin detail:", plugin.id);
    
    await invoke("show_notification", {
      options: {
        title: "功能开发中",
        body: `插件详情页即将推出`,
      },
    });
  }

  onMount(() => {
    loadPlugins();
  });

  // 搜索和筛选变化时重新加载
  $effect(() => {
    if (searchQuery !== undefined || selectedCategory !== undefined) {
      page = 1;
      loadPlugins();
    }
  });

  const totalPages = $derived(Math.ceil(total / limit));
</script>

<div class="flex h-full flex-col">
  <!-- 筛选栏 -->
  <div class="mb-3 flex items-center gap-2">
    <!-- 搜索框 -->
    <div class="relative flex-1">
      <MagnifyingGlass
        class="absolute top-1/2 left-2.5 h-4 w-4 -translate-y-1/2 text-neutral-400"
      />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="搜索插件..."
        class="h-9 w-full rounded border border-neutral-300 bg-white py-1.5 pr-3 pl-9 text-sm text-neutral-900 focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-900 dark:text-neutral-100 dark:focus:border-neutral-400"
      />
    </div>

    <!-- 分类筛选 -->
    <select
      bind:value={selectedCategory}
      class="h-9 rounded border border-neutral-300 bg-white px-3 text-sm text-neutral-900 focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-900 dark:text-neutral-100"
    >
      {#each categories as category}
        <option value={category.value}>{category.label}</option>
      {/each}
    </select>


  </div>

  <!-- 插件列表 -->
  <div class="flex-1 overflow-auto">
    {#if loading}
      <div class="flex h-full items-center justify-center text-neutral-500">
        <div class="text-center">
          <div class="mb-2 text-lg">加载中...</div>
          <div class="text-sm">正在获取插件列表</div>
        </div>
      </div>
    {:else if error}
      <div class="flex h-full flex-col items-center justify-center text-neutral-500">
        <Package class="mb-4 h-12 w-12 opacity-50" />
        <p class="text-lg">加载失败</p>
        <p class="mt-2 text-sm">{error}</p>
        <Button.Root
          class="mt-4 inline-flex h-8 items-center justify-center rounded bg-neutral-900 px-3 text-sm font-medium text-white hover:bg-neutral-800 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-200"
          onclick={loadPlugins}
        >
          重试
        </Button.Root>
      </div>
    {:else if plugins.length === 0}
      <div class="flex h-full flex-col items-center justify-center text-neutral-500">
        <Package class="mb-4 h-12 w-12 opacity-50" />
        <p class="text-lg">没有找到插件</p>
        <p class="mt-2 text-sm">尝试调整搜索条件</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
        {#each plugins as plugin (plugin.id)}
          <PluginCard {plugin} onclick={() => handlePluginClick(plugin)} />
        {/each}
      </div>

      <!-- 分页 -->
      {#if totalPages > 1}
        <div class="mt-4 flex items-center justify-center gap-2">
          <Button.Root
            class="h-8 rounded border border-neutral-300 bg-white px-3 text-sm hover:bg-neutral-50 disabled:opacity-50 dark:border-neutral-600 dark:bg-neutral-900 dark:hover:bg-neutral-800"
            disabled={page === 1}
            onclick={() => {
              page--;
              loadPlugins();
            }}
          >
            上一页
          </Button.Root>

          <span class="text-sm text-neutral-600 dark:text-neutral-400">
            {page} / {totalPages}
          </span>

          <Button.Root
            class="h-8 rounded border border-neutral-300 bg-white px-3 text-sm hover:bg-neutral-50 disabled:opacity-50 dark:border-neutral-600 dark:bg-neutral-900 dark:hover:bg-neutral-800"
            disabled={page === totalPages}
            onclick={() => {
              page++;
              loadPlugins();
            }}
          >
            下一页
          </Button.Root>
        </div>
      {/if}
    {/if}
  </div>
</div>
