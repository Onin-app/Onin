<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Star, Download, Package, GithubLogo } from "phosphor-svelte";
  import { marked } from "marked";

  interface PluginDetail {
    id: string;
    name: string;
    version: string;
    description: string;
    author?: string;
    icon?: string;
    dir_name: string;
    enabled: boolean;
    install_source: "local" | "marketplace";
    readme?: string;
    // 市场插件可能有的额外字段
    stars?: number;
    downloads?: number;
    repository?: string;
  }

  interface Props {
    pluginId: string;
    onclose: () => void;
  }

  let { pluginId, onclose }: Props = $props();
  let detail = $state<PluginDetail | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

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

    if (target.tagName === "A") {
      event.preventDefault();
      const href = target.getAttribute("href");

      if (href) {
        try {
          const { openUrl } = await import("@tauri-apps/plugin-opener");
          await openUrl(href);
          console.log("[InstalledPluginDetail] Opened URL in browser:", href);
        } catch (e) {
          console.error("Failed to open link:", e);
        }
      }
    }
  }

  // 生成插件 icon 的 URL
  async function getPluginIconUrl(plugin: PluginDetail): Promise<string | undefined> {
    if (!plugin.icon) {
      return undefined;
    }

    // 如果是完整 URL（marketplace 插件），直接返回
    if (
      plugin.icon.startsWith("http://") ||
      plugin.icon.startsWith("https://")
    ) {
      return plugin.icon;
    }

    // 如果是相对路径（本地插件），通过插件服务器访问
    try {
      const port = await invoke<number>("get_plugin_server_port");
      return `http://127.0.0.1:${port}/plugin/${plugin.dir_name}/${plugin.icon}`;
    } catch (e) {
      console.error("Failed to get plugin server port:", e);
      return undefined;
    }
  }

  // 加载插件详情
  async function loadDetail() {
    loading = true;
    error = null;

    try {
      // 先从本地获取基本信息
      const result = await invoke<PluginDetail>("get_plugin_detail", {
        pluginId,
      });
      detail = result;
      console.log("[InstalledPluginDetail] Loaded local detail:", result);

      // 如果是市场插件，从接口获取完整详情
      if (result.install_source === "marketplace") {
        try {
          const { fetchPluginDetail } = await import("$lib/api/marketplace");
          const marketDetail = await fetchPluginDetail(result.id);
          console.log("[InstalledPluginDetail] Loaded market detail:", marketDetail);
          
          // 合并数据：优先使用市场数据
          detail = {
            ...result,
            readme: marketDetail.readme || result.readme,
            stars: marketDetail.stars,
            downloads: marketDetail.downloads,
            repository: marketDetail.repository,
            version: marketDetail.version || result.version,
          };
        } catch (marketError) {
          console.error("Failed to load market detail, using local data:", marketError);
          // 如果市场接口失败，继续使用本地数据
        }
      }
    } catch (e) {
      console.error("Failed to load plugin detail:", e);
      error = e instanceof Error ? e.message : "加载失败";
    } finally {
      loading = false;
    }
  }

  // 组件挂载时加载详情
  $effect(() => {
    loadDetail();
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={onclose}
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
      onclick={onclose}
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
    </button>

    {#if loading}
      <div class="flex h-64 items-center justify-center">
        <div class="text-neutral-500">加载中...</div>
      </div>
    {:else if error}
      <div class="flex h-64 flex-col items-center justify-center">
        <p class="text-lg text-red-500">加载失败</p>
        <p class="mt-2 text-sm text-neutral-500">{error}</p>
      </div>
    {:else if detail}
      <!-- 插件头部 -->
      <div class="mb-6 flex items-start gap-4">
        <div
          class="flex h-20 w-20 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-neutral-100 to-neutral-200 dark:from-neutral-800 dark:to-neutral-700"
        >
          {#await getPluginIconUrl(detail)}
            <div class="text-4xl">🧩</div>
          {:then iconUrl}
            {#if iconUrl}
              <img
                src={iconUrl}
                alt={detail.name}
                class="h-16 w-16 rounded object-contain"
              />
            {:else}
              <div class="text-4xl">🧩</div>
            {/if}
          {:catch}
            <div class="text-4xl">🧩</div>
          {/await}
        </div>

        <div class="flex-1">
          <div class="mb-2 flex items-center gap-2">
            <h2 class="text-2xl font-bold">{detail.name}</h2>
            {#if detail.install_source === "local"}
              <span
                class="rounded bg-orange-500 px-2 py-0.5 text-xs font-medium text-white"
              >
                本地
              </span>
            {:else}
              <span
                class="rounded bg-blue-500 px-2 py-0.5 text-xs font-medium text-white"
              >
                市场
              </span>
            {/if}
          </div>
          <p class="mb-2 text-neutral-600 dark:text-neutral-400">
            {detail.description}
          </p>
          <div class="flex items-center gap-4 text-sm text-neutral-500">
            {#if detail.author}
              <span>作者: {detail.author}</span>
            {/if}
            <span>版本: {detail.version}</span>
            <span>ID: {detail.id}</span>
          </div>
        </div>
      </div>

      <!-- 统计信息（仅市场插件） -->
      {#if detail.install_source === "marketplace" && (detail.stars || detail.downloads)}
        <div
          class="mb-6 flex justify-around rounded-lg bg-neutral-50 p-4 dark:bg-neutral-800"
        >
          {#if detail.stars}
            <div class="flex items-center gap-3">
              <Star class="h-8 w-8 text-yellow-500" weight="fill" />
              <div>
                <div class="text-xl font-semibold">{detail.stars}</div>
                <div class="text-xs text-neutral-500">Stars</div>
              </div>
            </div>
          {/if}
          {#if detail.downloads}
            <div class="flex items-center gap-3">
              <Download class="h-8 w-8 text-blue-500" weight="fill" />
              <div>
                <div class="text-xl font-semibold">{detail.downloads}</div>
                <div class="text-xs text-neutral-500">Downloads</div>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      <!-- README -->
      {#if detail.readme}
        <div class="mb-6">
          <h3 class="mb-3 text-lg font-semibold">插件说明</h3>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="prose prose-sm dark:prose-invert max-w-none rounded-lg bg-neutral-50 p-4 dark:bg-neutral-800"
            onclick={handleMarkdownClick}
          >
            {@html renderMarkdown(detail.readme)}
          </div>
        </div>
      {:else}
        <div class="mb-6 text-center text-neutral-500">
          <Package class="mx-auto mb-2 h-12 w-12 opacity-50" />
          <p>该插件没有提供说明文档</p>
        </div>
      {/if}

      <!-- GitHub 链接（如果有） -->
      {#if detail.repository}
        <div class="border-t border-neutral-200 pt-4 dark:border-neutral-700">
          <a
            href={detail.repository}
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center gap-2 text-sm text-blue-600 hover:underline dark:text-blue-400"
          >
            <GithubLogo class="h-4 w-4" />
            查看源码
          </a>
        </div>
      {/if}
    {/if}
  </div>
</div>
