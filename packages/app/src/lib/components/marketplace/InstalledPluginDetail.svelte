<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openExternalLink } from "$lib/utils/link";
  import { Star, Download, Package, GithubLogo } from "phosphor-svelte";
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
  } from "$lib/components/ui/dialog";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import {
    formatPluginVersion,
    isValidPluginVersion,
  } from "$lib/utils/pluginVersion";
  import { renderMarkdown, setupImageFallback } from "$lib/utils/markdown";

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
    auto_detach?: boolean;
    terminate_on_bg?: boolean;
    run_at_startup?: boolean;
    stars?: number;
    downloads?: number;
    repository?: string;
  }

  interface Props {
    open: boolean;
    pluginId: string;
    onOpenChange: (open: boolean) => void;
  }

  let { open = $bindable(), pluginId, onOpenChange }: Props = $props();
  let detail = $state<PluginDetail | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

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

  // 生成插件 icon 的 URL
  async function getPluginIconUrl(
    plugin: PluginDetail,
  ): Promise<string | undefined> {
    if (!plugin.icon) {
      return undefined;
    }

    if (
      plugin.icon.startsWith("http://") ||
      plugin.icon.startsWith("https://")
    ) {
      return plugin.icon;
    }

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
      const result = await invoke<PluginDetail>("get_plugin_detail", {
        pluginId,
      });
      detail = result;

      if (result.install_source === "marketplace") {
        try {
          const { fetchPluginDetail } = await import("$lib/api/marketplace");
          const marketDetail = await fetchPluginDetail(result.id);

          detail = {
            ...result,
            readme: marketDetail.readme || result.readme,
            stars: marketDetail.stars,
            downloads: marketDetail.downloads,
            repository: marketDetail.repository,
            version: isValidPluginVersion(marketDetail.version)
              ? marketDetail.version
              : result.version,
          };
        } catch (marketError) {
          console.error(
            "Failed to load market detail, using local data:",
            marketError,
          );
        }
      }
    } catch (e) {
      console.error("Failed to load plugin detail:", e);
      error = e instanceof Error ? e.message : "加载失败";
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open && pluginId) {
      loadDetail();
    }
  });
</script>

<Dialog {open} {onOpenChange}>
  <DialogContent class="flex h-[80vh] max-w-2xl flex-col p-6">
    <div use:setupImageFallback class="flex-1 overflow-hidden">
      <ScrollArea
        class="h-full w-full"
        viewportClass="h-full w-full overflow-x-hidden pr-2"
      >
        {#if loading}
          <div class="flex h-64 items-center justify-center">
            <div class="text-muted-foreground text-sm">加载中...</div>
          </div>
        {:else if error}
          <div class="flex h-64 flex-col items-center justify-center">
            <p class="text-destructive text-base font-medium">加载失败</p>
            <p class="text-muted-foreground mt-1 text-xs">{error}</p>
          </div>
        {:else if detail}
          <!-- 插件头部 -->
          <div class="mb-6 flex items-start gap-4">
            <div
              class="bg-muted flex h-16 w-16 shrink-0 items-center justify-center rounded-lg"
            >
              {#await getPluginIconUrl(detail)}
                <div class="text-2xl">🧩</div>
              {:then iconUrl}
                {#if iconUrl}
                  <img
                    src={iconUrl}
                    alt={detail.name}
                    class="h-12 w-12 rounded object-contain"
                  />
                {:else}
                  <div class="text-2xl">🧩</div>
                {/if}
              {:catch}
                <div class="text-2xl">🧩</div>
              {/await}
            </div>

            <div class="flex-1">
              <h2 class="text-foreground mb-1 text-xl font-bold">
                {detail.name}
              </h2>
              <p class="text-muted-foreground mb-2 text-xs">
                {detail.description}
              </p>
              <div
                class="text-muted-foreground flex flex-wrap items-center gap-3 text-xs"
              >
                {#if detail.author}
                  <span>作者: {detail.author}</span>
                {/if}
                {#if isValidPluginVersion(detail.version)}
                  <span>版本: {formatPluginVersion(detail.version)}</span>
                {/if}
                <span
                  >来源: {detail.install_source === "local"
                    ? "本地导入"
                    : "插件市场"}</span
                >
              </div>
            </div>
          </div>

          <!-- 市场插件统计信息 -->
          {#if detail.install_source === "marketplace" && (detail.stars !== undefined || detail.downloads !== undefined)}
            <div class="bg-muted/50 mb-6 flex justify-around rounded-xl p-3">
              {#if detail.stars !== undefined}
                <div class="flex items-center gap-3">
                  <Star class="h-6 w-6 text-yellow-500" weight="fill" />
                  <div>
                    <div class="text-foreground text-base font-semibold">
                      {detail.stars}
                    </div>
                    <div class="text-muted-foreground text-[10px]">Stars</div>
                  </div>
                </div>
              {/if}
              {#if detail.downloads !== undefined}
                <div class="flex items-center gap-3">
                  <Download class="h-6 w-6 text-blue-500" weight="fill" />
                  <div>
                    <div class="text-foreground text-base font-semibold">
                      {detail.downloads}
                    </div>
                    <div class="text-muted-foreground text-[10px]">
                      Downloads
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          {/if}

          <!-- 配置信息 -->
          <div class="bg-muted/20 mb-6 rounded-xl border p-4 text-xs">
            <h3 class="text-foreground mb-2 text-sm font-semibold">运行配置</h3>
            <div class="text-muted-foreground grid grid-cols-2 gap-2">
              <div>
                自动分离窗口: {detail.auto_detach ? "是" : "否"}
              </div>
              <div>
                后台运行保留: {detail.terminate_on_bg === false ? "是" : "否"}
              </div>
              <div>
                开机启动: {detail.run_at_startup ? "是" : "否"}
              </div>
              <div>状态: {detail.enabled ? "已启用" : "已禁用"}</div>
            </div>
          </div>

          <!-- README -->
          {#if detail.readme}
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
                {@html renderMarkdown(detail.readme, detail.repository)}
              </div>
            </div>
          {/if}

          <!-- 底部操作 -->
          {#if detail.repository}
            <div class="border-t pt-4">
              <a
                href={detail.repository}
                target="_blank"
                rel="noopener noreferrer"
                class="text-primary flex items-center gap-2 text-xs hover:underline"
                onclick={(e) =>
                  detail?.repository && openExternalLink(detail.repository, e)}
              >
                <GithubLogo class="h-4 w-4" />
                查看源码
              </a>
            </div>
          {/if}
        {/if}
      </ScrollArea>
    </div>
  </DialogContent>
</Dialog>
