<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { MagnifyingGlass, Package, Star, Download, GithubLogo } from "phosphor-svelte";
  import { Button } from "bits-ui";
  import PluginCard from "./PluginCard.svelte";
  import { fetchPlugins, downloadAndInstallPlugin } from "$lib/api/marketplace";
  import type { MarketplacePlugin } from "$lib/types/marketplace";

  let plugins: MarketplacePlugin[] = $state([]);
  let installedPluginIds: Set<string> = $state(new Set());
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



  async function loadInstalledPlugins() {
    try {
      const installed = await invoke<any[]>("get_loaded_plugins");
      installedPluginIds = new Set(installed.map((p: any) => p.id));
      console.log("[Marketplace] Installed plugin IDs:", Array.from(installedPluginIds));
    } catch (e) {
      console.error("Failed to load installed plugins:", e);
    }
  }

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
      
      console.log("[Marketplace] Loaded plugins:", plugins.map(p => ({ id: p.id, name: p.name, icon: p.icon })));
      
      // 同时加载已安装插件列表
      await loadInstalledPlugins();
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

  let selectedPlugin = $state<MarketplacePlugin | null>(null);
  let loadingDetail = $state(false);

  async function handlePluginClick(plugin: MarketplacePlugin) {
    // 如果已经有下载链接，不需要再获取详情
    if (plugin.downloadUrl) {
      selectedPlugin = plugin;
      return;
    }

    // 获取插件详情（包含下载链接）
    loadingDetail = true;
    try {
      const { fetchPluginDetail } = await import("$lib/api/marketplace");
      const detail = await fetchPluginDetail(plugin.id);
      selectedPlugin = detail;
    } catch (e) {
      console.error("Failed to load plugin detail:", e);
      await invoke("show_notification", {
        options: {
          title: "加载失败",
          body: `无法获取插件详情`,
        },
      });
    } finally {
      loadingDetail = false;
    }
  }

  function closeDetail() {
    selectedPlugin = null;
  }

  async function handleInstall() {
    // 安装成功后刷新已安装插件列表
    try {
      await invoke("refresh_plugins");
      await loadInstalledPlugins();  // 重新加载已安装列表
      await invoke("show_notification", {
        options: {
          title: "安装成功",
          body: "插件已安装，请在已安装列表中查看",
        },
      });
    } catch (e) {
      console.error("Failed to refresh plugins:", e);
    }
    closeDetail();
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
          {@const isInstalled = installedPluginIds.has(plugin.id)}
          {#if isInstalled}
            {console.log(`[Marketplace] Plugin ${plugin.id} (${plugin.name}) is installed`)}
          {/if}
          <PluginCard 
            {plugin} 
            {isInstalled}
            onclick={() => handlePluginClick(plugin)}
            oninstall={handleInstall}
          />
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

<!-- 插件详情弹窗 -->
{#if selectedPlugin}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    onclick={closeDetail}
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="relative max-h-[80vh] w-full max-w-2xl overflow-auto rounded-lg bg-white p-6 shadow-xl dark:bg-neutral-900"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- 关闭按钮 -->
      <button
        class="absolute top-4 right-4 rounded p-1 hover:bg-neutral-100 dark:hover:bg-neutral-800"
        onclick={closeDetail}
        aria-label="关闭"
      >
        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>

      {#if loadingDetail}
        <div class="flex h-64 items-center justify-center">
          <div class="text-neutral-500">加载中...</div>
        </div>
      {:else}
        <!-- 插件头部 -->
        <div class="mb-6 flex items-start gap-4">
          <div class="flex h-20 w-20 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-neutral-100 to-neutral-200 dark:from-neutral-800 dark:to-neutral-700">
            {#if selectedPlugin.icon}
              <img src={selectedPlugin.icon} alt={selectedPlugin.name} class="h-16 w-16 rounded object-contain" />
            {:else}
              <div class="text-4xl">🧩</div>
            {/if}
          </div>

          <div class="flex-1">
            <h2 class="mb-2 text-2xl font-bold">{selectedPlugin.name}</h2>
            <p class="mb-2 text-neutral-600 dark:text-neutral-400">{selectedPlugin.description}</p>
            <div class="flex items-center gap-4 text-sm text-neutral-500">
              <span>作者: {selectedPlugin.author}</span>
              <span>分类: {selectedPlugin.category}</span>
              {#if selectedPlugin.version}
                <span>版本: {selectedPlugin.version}</span>
              {/if}
            </div>
          </div>
        </div>

        <!-- 统计信息 -->
        <div class="mb-6 flex gap-6 rounded-lg bg-neutral-50 p-4 dark:bg-neutral-800">
          <div class="flex items-center gap-2">
            <Star class="h-5 w-5 text-yellow-500" />
            <div>
              <div class="text-lg font-semibold">{selectedPlugin.stars}</div>
              <div class="text-xs text-neutral-500">Stars</div>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <Download class="h-5 w-5 text-blue-500" />
            <div>
              <div class="text-lg font-semibold">{selectedPlugin.downloads}</div>
              <div class="text-xs text-neutral-500">下载</div>
            </div>
          </div>
          {#if selectedPlugin.size}
            <div class="flex items-center gap-2">
              <Package class="h-5 w-5 text-green-500" />
              <div>
                <div class="text-lg font-semibold">{(selectedPlugin.size / 1024 / 1024).toFixed(2)} MB</div>
                <div class="text-xs text-neutral-500">大小</div>
              </div>
            </div>
          {/if}
        </div>

        <!-- 更新说明 -->
        {#if selectedPlugin.releaseNotes}
          <div class="mb-6">
            <h3 class="mb-2 font-semibold">更新说明</h3>
            <div class="rounded-lg bg-neutral-50 p-4 text-sm dark:bg-neutral-800">
              <pre class="whitespace-pre-wrap font-sans">{selectedPlugin.releaseNotes}</pre>
            </div>
          </div>
        {/if}

        <!-- 关键词 -->
        {#if selectedPlugin.keywords && selectedPlugin.keywords.length > 0}
          <div class="mb-6">
            <h3 class="mb-2 font-semibold">标签</h3>
            <div class="flex flex-wrap gap-2">
              {#each selectedPlugin.keywords as keyword}
                <span class="rounded bg-neutral-100 px-2 py-1 text-xs dark:bg-neutral-800">
                  {keyword}
                </span>
              {/each}
            </div>
          </div>
        {/if}

        <!-- 底部操作 -->
        <div class="border-t border-neutral-200 pt-4 dark:border-neutral-700">
          <a
            href={selectedPlugin.repository}
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center gap-2 text-sm text-blue-600 hover:underline dark:text-blue-400"
          >
            <GithubLogo class="h-4 w-4" />
            查看源码
          </a>
        </div>
      {/if}
    </div>
  </div>
{/if}
