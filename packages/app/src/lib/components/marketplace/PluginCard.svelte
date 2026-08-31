<script lang="ts">
  import {
    GithubLogo,
    Download,
    Star,
    DownloadSimple,
    Check,
  } from "phosphor-svelte";
  import type { MarketplacePlugin } from "$lib/types/marketplace";
  import { downloadAndInstallPlugin } from "$lib/api/marketplace";
  import { invoke } from "@tauri-apps/api/core";
  import { openExternalLink } from "$lib/utils/link";
  import {
    comparePluginVersions,
    formatPluginVersion,
    isValidPluginVersion,
  } from "$lib/utils/pluginVersion";
  import { toast } from "svelte-sonner";

  interface Props {
    plugin: MarketplacePlugin;
    isInstalled?: boolean;
    installedVersion?: string;
    onclick?: () => void;
    showStats?: boolean; // 是否显示统计信息（star 和 downloads）
  }

  let {
    plugin,
    isInstalled = false,
    installedVersion,
    onclick,
    showStats = true,
  }: Props = $props();

  let imageError = $state(false);
  let installing = $state(false);

  function isNewerVersion(
    marketplaceVersion?: string,
    localVersion?: string,
  ): boolean {
    return comparePluginVersions(marketplaceVersion, localVersion) > 0;
  }

  // 仅当市场版本高于本地版本时显示更新
  const hasUpdate = $derived(
    isInstalled && isNewerVersion(plugin.version, installedVersion),
  );

  function handleImageError() {
    imageError = true;
  }

  function formatNumber(num: number): string {
    if (num >= 1000) {
      return (num / 1000).toFixed(1) + "k";
    }
    return num.toString();
  }

  async function handleInstall(e: MouseEvent) {
    e.stopPropagation();

    if (installing || (isInstalled && !hasUpdate)) {
      return;
    }

    try {
      installing = true;

      let currentDownloadUrl = plugin.downloadUrl;
      let currentIcon = plugin.icon;
      let currentVersion = plugin.version;

      // 如果缺少下载地址（列表接口通常不返回），则先获取插件详情
      if (!currentDownloadUrl) {
        const { fetchPluginDetail } = await import("$lib/api/marketplace");
        const detail = await fetchPluginDetail(plugin.id);
        currentDownloadUrl = detail.downloadUrl;
        if (detail.icon) currentIcon = detail.icon;
        if (isValidPluginVersion(detail.version))
          currentVersion = detail.version;
      }

      if (!currentDownloadUrl) {
        throw new Error("未能获取到插件下载地址");
      }

      await downloadAndInstallPlugin(
        currentDownloadUrl,
        plugin.id,
        currentIcon,
        hasUpdate, // 如果有更新，则使用覆盖模式
        currentVersion,
      );
      const isUpdate = hasUpdate;
      const actionName = isUpdate ? "更新" : "安装";
      toast.success(`插件 ${plugin.name} ${actionName}成功`);
    } catch (error) {
      console.error("Failed to install plugin:", error);
      toast.error(`安装失败: ${String(error)}`);
    } finally {
      installing = false;
    }
  }
</script>

<div
  class="group border-border/60 bg-card hover:border-border flex cursor-pointer flex-col rounded-2xl border p-3.5 text-left shadow-2xs transition-[border-color,box-shadow,transform] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] hover:shadow-xs active:scale-[0.99]"
  {onclick}
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onclick?.();
    }
  }}
>
  <!-- 顶部：图标和信息 -->
  <div class="mb-2 flex items-start gap-3">
    <!-- 图标 -->
    <div
      class="bg-muted/60 border-border/50 flex h-13 w-13 shrink-0 items-center justify-center rounded-xl border shadow-xs"
    >
      {#if plugin.icon && !imageError}
        <img
          src={plugin.icon}
          alt={plugin.name}
          class="h-9 w-9 rounded-md object-contain"
          onerror={handleImageError}
          crossorigin="anonymous"
        />
      {:else}
        <div class="text-2xl">🧩</div>
      {/if}
    </div>

    <!-- 右侧信息 -->
    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
      <!-- 标题、版本和 GitHub 链接 -->
      <div class="flex items-start justify-between gap-2">
        <div class="flex min-w-0 items-baseline gap-2">
          <h3
            class="text-foreground truncate text-sm leading-tight font-semibold tracking-tight"
          >
            {plugin.name}
          </h3>
          {#if isValidPluginVersion(plugin.version)}
            <span
              class="text-muted-foreground/60 shrink-0 font-mono text-[11px]"
              >{formatPluginVersion(plugin.version)}</span
            >
          {/if}
        </div>
        <a
          href={plugin.repository}
          target="_blank"
          rel="noopener noreferrer"
          class="text-muted-foreground hover:text-foreground hover:bg-muted shrink-0 rounded-lg p-1 opacity-0 transition-all duration-120 group-hover:opacity-100"
          onclick={(e) => {
            e.stopPropagation();
            plugin.repository && openExternalLink(plugin.repository, e);
          }}
          aria-label="查看 GitHub"
        >
          <GithubLogo class="h-3.5 w-3.5" />
        </a>
      </div>

      <!-- 描述 -->
      <p class="text-muted-foreground/75 line-clamp-1 text-xs leading-normal">
        {plugin.description}
      </p>

      <!-- 作者和 ID -->
      <div
        class="text-muted-foreground/60 flex items-center justify-between gap-2 text-xs"
      >
        <span class="truncate text-[11.5px]">{plugin.author}</span>
        <span class="shrink-0 font-mono text-[10px]">ID: {plugin.id}</span>
      </div>
    </div>
  </div>

  <!-- 底部：统计、分类和安装按钮 -->
  <div
    class="border-border/40 flex items-center justify-between border-t pt-2.5"
  >
    <!-- 左侧：统计或分类 -->
    {#if showStats}
      <!-- 统计信息（市场列表） -->
      <div class="text-muted-foreground/70 flex items-center gap-3 text-xs">
        <div class="flex items-center gap-1">
          <Star class="h-3.5 w-3.5 text-amber-500/80" />
          <span class="font-mono text-[11px]">{formatNumber(plugin.stars)}</span
          >
        </div>
        <div class="flex items-center gap-1">
          <Download class="h-3.5 w-3.5 text-blue-500/80" />
          <span class="font-mono text-[11px]"
            >{formatNumber(plugin.downloads)}</span
          >
        </div>
      </div>
    {:else}
      <!-- 分类标签（已安装列表） -->
      <span
        class="bg-muted/70 text-muted-foreground rounded-md px-1.5 py-0.5 text-[10px] font-medium"
      >
        {plugin.category}
      </span>
    {/if}

    <!-- 右侧：分类和安装按钮 -->
    <div class="flex items-center gap-2">
      <!-- 分类标签（仅在市场列表显示） -->
      {#if showStats}
        <span
          class="bg-muted/70 text-muted-foreground rounded-md px-1.5 py-0.5 text-[10px] font-medium"
        >
          {plugin.category}
        </span>
      {/if}

      <!-- 安装按钮 -->
      {#if plugin.id}
        <button
          class="bg-primary text-primary-foreground hover:bg-primary/90 flex cursor-pointer items-center gap-1 rounded-lg px-2.5 py-1 text-xs font-medium shadow-2xs transition-[transform,background-color] duration-120 active:scale-95 disabled:cursor-not-allowed disabled:opacity-50"
          onclick={handleInstall}
          disabled={installing || (isInstalled && !hasUpdate)}
          class:opacity-50={isInstalled && !hasUpdate}
          class:cursor-not-allowed={isInstalled && !hasUpdate}
        >
          {#if installing}
            <span>{hasUpdate ? "更新中..." : "安装中..."}</span>
          {:else if hasUpdate}
            <DownloadSimple class="h-3.5 w-3.5" />
            <span>更新</span>
          {:else if isInstalled}
            <Check class="h-3.5 w-3.5" />
            <span>已安装</span>
          {:else}
            <DownloadSimple class="h-3.5 w-3.5" />
            <span>安装</span>
          {/if}
        </button>
      {/if}
    </div>
  </div>
</div>
