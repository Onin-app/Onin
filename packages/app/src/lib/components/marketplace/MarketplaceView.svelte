<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { openExternalLink } from "$lib/utils/link";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    MagnifyingGlass,
    Package,
    Star,
    Download,
    GithubLogo,
  } from "phosphor-svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
  } from "$lib/components/ui/dialog";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import PluginCard from "./PluginCard.svelte";
  import { fetchPlugins } from "$lib/api/marketplace";
  import type { MarketplacePlugin } from "$lib/types/marketplace";
  import type { PluginManifest } from "$lib/composables/usePluginList.svelte";
  import {
    formatPluginVersion,
    isValidPluginVersion,
  } from "$lib/utils/pluginVersion";
  import { renderMarkdown, setupImageFallback } from "$lib/utils/markdown";

  interface Props {
    active?: boolean;
    refreshTrigger?: number;
  }

  let { active = false, refreshTrigger = 0 }: Props = $props();

  let plugins: MarketplacePlugin[] = $state([]);
  let installedVersions: Map<string, string> = $state(new Map());
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

  // 处理 markdown 中的链接点击
  async function handleMarkdownClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const anchor = target.closest("a");

    if (anchor) {
      const href = anchor.getAttribute("href");
      if (href) {
        await openExternalLink(href, event);
      }
    }
  }

  async function loadInstalledPlugins() {
    try {
      const installed = await invoke<PluginManifest[]>("get_loaded_plugins");
      const versions = new Map<string, string>();
      for (const p of installed) {
        if (p.install_source === "marketplace") {
          if (p.id) versions.set(p.id, p.version);
          if (p.dir_name && p.dir_name !== p.id) {
            versions.set(p.dir_name, p.version);
          }
        }
      }
      installedVersions = versions;
    } catch (e) {
      console.error("Failed to load installed plugins:", e);
    }
  }

  async function loadPlugins() {
    loading = true;
    error = null;
    try {
      const result = await fetchPlugins({
        page,
        limit,
        keyword: searchQuery || undefined,
        category: selectedCategory === "all" ? undefined : selectedCategory,
      });
      plugins = result.data;
      total = result.meta.total;
      await loadInstalledPlugins();
    } catch (e: any) {
      error = e.message || "加载插件失败";
      console.error("Failed to load plugins:", e);
    } finally {
      loading = false;
    }
  }

  let searchTimeout: number | null = null;
  $effect(() => {
    searchQuery;
    selectedCategory;

    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }

    searchTimeout = setTimeout(() => {
      page = 1;
      loadPlugins();
    }, 300) as unknown as number;
  });

  let selectedPlugin = $state<MarketplacePlugin | null>(null);
  let detailDialogOpen = $state(false);
  let loadingDetail = $state(false);

  async function handlePluginClick(plugin: MarketplacePlugin) {
    selectedPlugin = plugin;
    detailDialogOpen = true;
  }

  function handleDetailDialogOpenChange(open: boolean) {
    detailDialogOpen = open;
    if (!open) {
      selectedPlugin = null;
    }
  }

  let unlistenPluginChanged: UnlistenFn | null = null;
  onMount(async () => {
    loadPlugins();

    unlistenPluginChanged = await listen("plugin-changed", () => {
      loadInstalledPlugins();
    });
  });

  onDestroy(() => {
    if (unlistenPluginChanged) {
      unlistenPluginChanged();
    }
  });

  $effect(() => {
    if (active) {
      loadInstalledPlugins();
    }
  });

  $effect(() => {
    if (refreshTrigger > 0 && active) {
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
        class="text-muted-foreground/60 absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2"
      />
      <Input
        type="text"
        bind:value={searchQuery}
        placeholder="搜索插件..."
        class="h-8.5 w-full rounded-xl pl-8 text-xs font-normal transition-[border-color,box-shadow] duration-140"
      />
    </div>

    <!-- 分类筛选 -->
    <select
      bind:value={selectedCategory}
      class="border-border/60 bg-background text-foreground focus:border-primary focus:ring-primary/20 h-8.5 cursor-pointer rounded-xl border px-3 text-xs shadow-2xs transition-[border-color,box-shadow] duration-140 outline-none focus:ring-2"
    >
      {#each categories as category}
        <option value={category.value}>{category.label}</option>
      {/each}
    </select>
  </div>

  <!-- 插件列表 -->
  <ScrollArea class="flex-1" viewportClass="h-full w-full overflow-x-hidden">
    <div class="pr-2 pb-4">
      {#if loading}
        <div
          class="text-muted-foreground flex h-full min-h-64 items-center justify-center"
        >
          <div class="text-center">
            <div class="text-foreground/80 mb-1 text-sm font-medium">
              加载中...
            </div>
            <div class="text-muted-foreground/60 text-xs">正在获取插件列表</div>
          </div>
        </div>
      {:else if error}
        <div
          class="text-muted-foreground flex h-full min-h-64 flex-col items-center justify-center"
        >
          <Package class="mb-3 h-10 w-10 opacity-40" />
          <p class="text-foreground text-sm font-medium">加载失败</p>
          <p class="text-muted-foreground mt-1 text-xs">{error}</p>
          <Button
            size="sm"
            class="mt-3 rounded-xl text-xs transition-transform active:scale-95"
            onclick={loadPlugins}>重试</Button
          >
        </div>
      {:else if plugins.length === 0}
        <div
          class="text-muted-foreground flex h-full min-h-64 flex-col items-center justify-center"
        >
          <Package class="mb-3 h-10 w-10 opacity-40" />
          <p class="text-foreground text-sm font-medium">没有找到插件</p>
          <p class="text-muted-foreground/60 mt-1 text-xs">尝试调整搜索条件</p>
        </div>
      {:else}
        <div class="grid grid-cols-1 gap-2.5 md:grid-cols-2 xl:grid-cols-3">
          {#each plugins as plugin (plugin.id)}
            {@const installedVersion = installedVersions.get(plugin.id)}
            {@const isInstalled = !!installedVersion}
            <PluginCard
              {plugin}
              {isInstalled}
              {installedVersion}
              showStats={true}
              onclick={() => handlePluginClick(plugin)}
            />
          {/each}
        </div>

        {#if totalPages > 1}
          <div class="mt-4 flex items-center justify-center gap-2 pb-1">
            <Button
              variant="outline"
              size="sm"
              class="h-7.5 rounded-xl px-2.5 text-xs shadow-2xs transition-transform duration-120 active:scale-95"
              disabled={page === 1}
              onclick={() => {
                page--;
                loadPlugins();
              }}
            >
              上一页
            </Button>

            <span class="text-muted-foreground font-mono text-xs">
              {page} / {totalPages}
            </span>

            <Button
              variant="outline"
              size="sm"
              class="h-7.5 rounded-xl px-2.5 text-xs shadow-2xs transition-transform duration-120 active:scale-95"
              disabled={page === totalPages}
              onclick={() => {
                page++;
                loadPlugins();
              }}
            >
              下一页
            </Button>
          </div>
        {/if}
      {/if}
    </div>
  </ScrollArea>
</div>

<!-- 插件详情弹窗 -->
{#if selectedPlugin}
  <Dialog open={detailDialogOpen} onOpenChange={handleDetailDialogOpenChange}>
    <DialogContent class="flex h-[80vh] max-w-2xl flex-col p-6">
      <div use:setupImageFallback class="flex-1 overflow-hidden">
        <ScrollArea
          class="h-full w-full"
          viewportClass="h-full w-full overflow-x-hidden pr-2"
        >
          {#if loadingDetail}
            <div class="flex h-64 items-center justify-center">
              <div class="text-muted-foreground text-sm">加载中...</div>
            </div>
          {:else}
            <!-- 插件头部 -->
            <div class="mb-6 flex items-start gap-4">
              <div
                class="bg-muted flex h-16 w-16 shrink-0 items-center justify-center rounded-lg"
              >
                {#if selectedPlugin.icon}
                  <img
                    src={selectedPlugin.icon}
                    alt={selectedPlugin.name}
                    class="h-12 w-12 rounded object-contain"
                  />
                {:else}
                  <div class="text-3xl">🧩</div>
                {/if}
              </div>

              <div class="flex-1">
                <h2 class="text-foreground mb-1 text-xl font-bold">
                  {selectedPlugin.name}
                </h2>
                <p class="text-muted-foreground mb-2 text-xs">
                  {selectedPlugin.description}
                </p>
                <div
                  class="text-muted-foreground flex flex-wrap items-center gap-3 text-xs"
                >
                  <span>作者: {selectedPlugin.author}</span>
                  <span>分类: {selectedPlugin.category}</span>
                  {#if isValidPluginVersion(selectedPlugin.version)}
                    <span
                      >版本: {formatPluginVersion(selectedPlugin.version)}</span
                    >
                  {/if}
                </div>
              </div>
            </div>

            <!-- 统计信息 -->
            <div class="bg-muted/50 mb-6 flex justify-around rounded-xl p-3">
              <div class="flex items-center gap-3">
                <Star class="h-6 w-6 text-yellow-500" weight="fill" />
                <div>
                  <div class="text-foreground text-base font-semibold">
                    {selectedPlugin.stars}
                  </div>
                  <div class="text-muted-foreground text-[10px]">Stars</div>
                </div>
              </div>
              <div class="flex items-center gap-3">
                <Download class="h-6 w-6 text-blue-500" weight="fill" />
                <div>
                  <div class="text-foreground text-base font-semibold">
                    {selectedPlugin.downloads}
                  </div>
                  <div class="text-muted-foreground text-[10px]">Downloads</div>
                </div>
              </div>
              {#if selectedPlugin.size}
                <div class="flex items-center gap-3">
                  <Package class="h-6 w-6 text-emerald-500" weight="fill" />
                  <div>
                    <div class="text-foreground text-base font-semibold">
                      {(selectedPlugin.size / 1024 / 1024).toFixed(2)} MB
                    </div>
                    <div class="text-muted-foreground text-[10px]">Size</div>
                  </div>
                </div>
              {/if}
            </div>

            <!-- README -->
            {#if selectedPlugin.readme}
              <div class="mb-6">
                <h3 class="text-foreground mb-2 text-sm font-semibold">
                  插件说明
                </h3>
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="prose prose-sm dark:prose-invert bg-muted/40 max-w-none rounded-xl p-4"
                  onclick={handleMarkdownClick}
                >
                  {@html renderMarkdown(
                    selectedPlugin.readme,
                    selectedPlugin.repository,
                  )}
                </div>
              </div>
            {/if}

            <!-- 更新说明 -->
            {#if selectedPlugin.releaseNotes}
              <div class="mb-6">
                <h3 class="text-foreground mb-2 text-sm font-semibold">
                  更新说明
                </h3>
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="prose prose-sm dark:prose-invert bg-muted/40 max-w-none rounded-xl p-4"
                  onclick={handleMarkdownClick}
                >
                  {@html renderMarkdown(
                    selectedPlugin.releaseNotes,
                    selectedPlugin.repository,
                  )}
                </div>
              </div>
            {/if}

            <!-- 关键词 -->
            {#if selectedPlugin.keywords && selectedPlugin.keywords.length > 0}
              <div class="mb-6">
                <h3 class="text-foreground mb-2 text-xs font-semibold">标签</h3>
                <div class="flex flex-wrap gap-1.5">
                  {#each selectedPlugin.keywords as keyword}
                    <span
                      class="bg-muted text-muted-foreground rounded-md px-2 py-0.5 text-xs"
                    >
                      {keyword}
                    </span>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- 底部操作 -->
            <div class="border-t pt-4">
              <a
                href={selectedPlugin.repository}
                target="_blank"
                rel="noopener noreferrer"
                class="text-primary flex items-center gap-2 text-xs hover:underline"
                onclick={(e) =>
                  selectedPlugin?.repository &&
                  openExternalLink(selectedPlugin.repository, e)}
              >
                <GithubLogo class="h-4 w-4" />
                查看源码
              </a>
            </div>
          {/if}
        </ScrollArea>
      </div>
    </DialogContent>
  </Dialog>
{/if}
