<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    MagnifyingGlass,
    Package,
    Star,
    Download,
    GithubLogo,
  } from "phosphor-svelte";
  import { Button, Dialog } from "bits-ui";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import PluginCard from "./PluginCard.svelte";
  import { fetchPlugins } from "$lib/api/marketplace";
  import type { MarketplacePlugin } from "$lib/types/marketplace";
  import type { PluginManifest } from "$lib/composables/usePluginList.svelte";
  import { marked } from "marked";

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

  // Markdown 渲染函数
  function renderMarkdown(markdown: string): string {
    try {
      return marked.parse(markdown, { async: false }) as string;
    } catch (e) {
      console.error("Failed to render markdown:", e);
      return markdown;
    }
  }

  // 处理 markdown 中的链接点击
  async function handleMarkdownClick(event: MouseEvent) {
    const target = event.target as HTMLElement;

    // 检查是否点击了链接
    if (target.tagName === "A") {
      event.preventDefault();
      const href = target.getAttribute("href");

      if (href) {
        try {
          const { openUrl } = await import("@tauri-apps/plugin-opener");
          await openUrl(href);
        } catch (e) {
          console.error("Failed to open link:", e);
        }
      }
    }
  }

  async function loadInstalledPlugins() {
    try {
      const installed = await invoke<PluginManifest[]>("get_loaded_plugins");
      // 只有来源明确为“marketplace”的插件才在市场显示为“已安装”
      // 排除本地导入(@local)或其他非市场来源的版本
      // 同时考虑 id 和 dir_name，因为市场 ID 可能对应后端的 dir_name
      const versions = new Map<string, string>();
      installed
        .filter((p) => p.install_source === "marketplace")
        .forEach((p) => {
          if (p.id) versions.set(p.id, p.version);
          if (p.dir_name) versions.set(p.dir_name, p.version);
        });
      installedVersions = versions;
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

      // 去重：防止插件在多个分类中出现导致 Svelte Key 重复
      const uniquePlugins: MarketplacePlugin[] = [];
      const seenIds = new Set<string>();

      for (const p of response.data) {
        if (p.id && !seenIds.has(p.id)) {
          seenIds.add(p.id);
          uniquePlugins.push(p);
        }
      }

      plugins = uniquePlugins;
      total = response.meta.total;

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
  let detailDialogOpen = $state(false);

  async function handlePluginClick(plugin: MarketplacePlugin) {
    // 总是获取插件详情以获取完整信息（包括 releaseNotes）
    loadingDetail = true;
    selectedPlugin = plugin; // 先显示基本信息
    detailDialogOpen = true;

    try {
      const { fetchPluginDetail } = await import("$lib/api/marketplace");
      const detail = await fetchPluginDetail(plugin.id);
      selectedPlugin = detail; // 更新为完整详情
    } catch (e) {
      console.error("Failed to load plugin detail:", e);
      // 即使获取详情失败，也显示基本信息
      await invoke("show_notification", {
        options: {
          title: "提示",
          body: `无法获取完整插件详情，显示基本信息`,
        },
      });
    } finally {
      loadingDetail = false;
    }
  }

  function handleDetailDialogOpenChange(open: boolean) {
    detailDialogOpen = open;
    if (!open) {
      selectedPlugin = null;
    }
  }

  async function handleInstall(_isUpdate: boolean = false) {
    // 强制刷新已安装版本信息
    await loadInstalledPlugins();
    // 强制 Svelte 刷新依赖此 Map 的派生状态
    installedVersions = new Map(installedVersions);
    detailDialogOpen = false;
  }

  // 事件监听清理函数
  let unlistenFns: UnlistenFn[] = [];

  onMount(async () => {
    loadPlugins();

    // 监听插件卸载事件，刷新已安装列表
    const unlistenUninstalled = await listen<string>(
      "plugin-uninstalled",
      async () => {
        await loadInstalledPlugins();
      },
    );
    unlistenFns.push(unlistenUninstalled);

    // 监听插件安装事件，刷新已安装列表
    const unlistenInstalled = await listen<string>(
      "plugin-installed",
      async () => {
        await loadInstalledPlugins();
      },
    );
    unlistenFns.push(unlistenInstalled);
  });

  onDestroy(() => {
    // 清理事件监听
    unlistenFns.forEach((fn) => fn());
  });

  // 搜索和筛选变化时重新加载
  $effect(() => {
    if (searchQuery !== undefined || selectedCategory !== undefined) {
      page = 1;
      loadPlugins();
    }
  });

  // 当进入市场页时，强制刷新一次本地已安装列表，确保安装状态/按钮显示正确 (P2 修复)
  $effect(() => {
    if (active) {
      loadInstalledPlugins();
    }
  });

  // 监听外部刷新触发
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
  <AppScrollArea class="flex-1" viewportClass="h-full w-full overflow-x-hidden">
    <div class="pr-2">
        {#if loading}
          <div class="flex h-full min-h-64 items-center justify-center text-neutral-500">
            <div class="text-center">
              <div class="mb-2 text-lg">加载中...</div>
              <div class="text-sm">正在获取插件列表</div>
            </div>
          </div>
        {:else if error}
          <div
            class="flex h-full min-h-64 flex-col items-center justify-center text-neutral-500"
          >
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
          <div
            class="flex h-full min-h-64 flex-col items-center justify-center text-neutral-500"
          >
            <Package class="mb-4 h-12 w-12 opacity-50" />
            <p class="text-lg">没有找到插件</p>
            <p class="mt-2 text-sm">尝试调整搜索条件</p>
          </div>
        {:else}
          <div class="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
            {#each plugins as plugin (plugin.id)}
              {@const installedVersion = installedVersions.get(plugin.id)}
              {@const isInstalled = !!installedVersion}
              <PluginCard
                {plugin}
                {isInstalled}
                {installedVersion}
                showStats={true}
                onclick={() => handlePluginClick(plugin)}
                oninstall={(isUpdate) => handleInstall(isUpdate)}
              />
            {/each}
          </div>

          {#if totalPages > 1}
            <div class="mt-4 flex items-center justify-center gap-2 pb-1">
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
  </AppScrollArea>
</div>

<!-- 插件详情弹窗 -->
{#if selectedPlugin}
  <Dialog.Root
    open={detailDialogOpen}
    onOpenChange={handleDetailDialogOpenChange}
  >
    <Dialog.Portal>
      <Dialog.Overlay
        class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/50"
      />
      <Dialog.Content
        class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] fixed top-[50%] left-[50%] z-50 h-[80vh] w-full max-w-2xl translate-x-[-50%] translate-y-[-50%] overflow-hidden rounded-lg bg-white p-6 shadow-xl dark:bg-neutral-900"
      >
        <Dialog.Close
          class="absolute top-4 right-4 rounded p-1 hover:bg-neutral-100 dark:hover:bg-neutral-800"
          aria-label="关闭"
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
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </Dialog.Close>

        <AppScrollArea
          class="h-full w-full"
          viewportClass="h-full w-full overflow-x-hidden pr-2"
        >
            {#if loadingDetail}
              <div class="flex h-64 items-center justify-center">
                <div class="text-neutral-500">加载中...</div>
              </div>
            {:else}
              <!-- 插件头部 -->
              <div class="mb-6 flex items-start gap-4">
            <div
              class="flex h-20 w-20 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-neutral-100 to-neutral-200 dark:from-neutral-800 dark:to-neutral-700"
            >
              {#if selectedPlugin.icon}
                <img
                  src={selectedPlugin.icon}
                  alt={selectedPlugin.name}
                  class="h-16 w-16 rounded object-contain"
                />
              {:else}
                <div class="text-4xl">🧩</div>
              {/if}
            </div>

            <div class="flex-1">
              <h2 class="mb-2 text-2xl font-bold">{selectedPlugin.name}</h2>
              <p class="mb-2 text-neutral-600 dark:text-neutral-400">
                {selectedPlugin.description}
              </p>
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
          <div
            class="mb-6 flex justify-around rounded-lg bg-neutral-50 p-4 dark:bg-neutral-800"
          >
            <div class="flex items-center gap-3">
              <Star class="h-8 w-8 text-yellow-500" weight="fill" />
              <div>
                <div class="text-xl font-semibold">{selectedPlugin.stars}</div>
                <div class="text-xs text-neutral-500">Stars</div>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <Download class="h-8 w-8 text-blue-500" weight="fill" />
              <div>
                <div class="text-xl font-semibold">
                  {selectedPlugin.downloads}
                </div>
                <div class="text-xs text-neutral-500">Downloads</div>
              </div>
            </div>
            {#if selectedPlugin.size}
              <div class="flex items-center gap-3">
                <Package class="h-8 w-8 text-green-500" weight="fill" />
                <div>
                  <div class="text-xl font-semibold">
                    {(selectedPlugin.size / 1024 / 1024).toFixed(2)} MB
                  </div>
                  <div class="text-xs text-neutral-500">Size</div>
                </div>
              </div>
            {/if}
          </div>

          <!-- README -->
          {#if selectedPlugin.readme}
            <div class="mb-6">
              <h3 class="mb-3 text-lg font-semibold">插件说明</h3>
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="prose prose-sm dark:prose-invert max-w-none rounded-lg bg-neutral-50 p-4 dark:bg-neutral-800"
                onclick={handleMarkdownClick}
              >
                {@html renderMarkdown(selectedPlugin.readme)}
              </div>
            </div>
          {/if}

          <!-- 更新说明 -->
          {#if selectedPlugin.releaseNotes}
            <div class="mb-6">
              <h3 class="mb-3 text-lg font-semibold">更新说明</h3>
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="prose prose-sm dark:prose-invert max-w-none rounded-lg bg-neutral-50 p-4 dark:bg-neutral-800"
                onclick={handleMarkdownClick}
              >
                {@html renderMarkdown(selectedPlugin.releaseNotes)}
              </div>
            </div>
          {/if}

          <!-- 关键词 -->
          {#if selectedPlugin.keywords && selectedPlugin.keywords.length > 0}
            <div class="mb-6">
              <h3 class="mb-2 font-semibold">标签</h3>
              <div class="flex flex-wrap gap-2">
                {#each selectedPlugin.keywords as keyword}
                  <span
                    class="rounded bg-neutral-100 px-2 py-1 text-xs dark:bg-neutral-800"
                  >
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
        </AppScrollArea>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
{/if}
