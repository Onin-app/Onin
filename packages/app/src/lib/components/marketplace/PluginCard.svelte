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
        if (isValidPluginVersion(detail.version)) currentVersion = detail.version;
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
      alert(`安装失败: ${error}`);
    } finally {
      installing = false;
    }
  }
</script>

<div
  class="group flex cursor-pointer flex-col rounded-lg border border-neutral-200 bg-white p-3 text-left transition-all hover:border-neutral-300 hover:shadow-sm dark:border-neutral-700 dark:bg-neutral-900 dark:hover:border-neutral-600"
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
      class="flex h-16 w-16 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-neutral-100 to-neutral-200 dark:from-neutral-800 dark:to-neutral-700"
    >
      {#if plugin.icon && !imageError}
        <img
          src={plugin.icon}
          alt={plugin.name}
          class="h-12 w-12 rounded object-contain"
          onerror={handleImageError}
          crossorigin="anonymous"
        />
      {:else}
        <div class="text-2xl">🧩</div>
      {/if}
    </div>

    <!-- 右侧信息 -->
    <div class="flex min-w-0 flex-1 flex-col gap-1">
      <!-- 标题、版本和 GitHub 链接 -->
      <div class="flex items-start justify-between gap-2">
        <div class="flex min-w-0 items-baseline gap-2">
          <h3 class="truncate text-base leading-tight font-semibold">
            {plugin.name}
          </h3>
          {#if isValidPluginVersion(plugin.version)}
            <span class="shrink-0 text-xs text-neutral-400"
              >{formatPluginVersion(plugin.version)}</span
            >
          {/if}
        </div>
        <a
          href={plugin.repository}
          target="_blank"
          rel="noopener noreferrer"
          class="shrink-0 rounded p-1 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-neutral-100 dark:hover:bg-neutral-800"
          onclick={(e) => e.stopPropagation()}
          aria-label="查看 GitHub"
        >
          <GithubLogo class="h-4 w-4" />
        </a>
      </div>

      <!-- 描述 -->
      <p class="line-clamp-1 text-sm text-neutral-500 dark:text-neutral-400">
        {plugin.description}
      </p>

      <!-- 作者和 ID -->
      <div
        class="flex items-center justify-between gap-2 text-xs text-neutral-400"
      >
        <span class="truncate">{plugin.author}</span>
        <span class="shrink-0 text-neutral-300 dark:text-neutral-600"
          >ID: {plugin.id}</span
        >
      </div>
    </div>
  </div>

  <!-- 底部：统计、分类和安装按钮 -->
  <div
    class="flex items-center justify-between border-t border-neutral-200 pt-2 dark:border-neutral-700"
  >
    <!-- 左侧：统计或分类 -->
    {#if showStats}
      <!-- 统计信息（市场列表） -->
      <div class="flex items-center gap-3 text-xs text-neutral-500">
        <div class="flex items-center gap-1">
          <Star class="h-3.5 w-3.5" />
          <span>{formatNumber(plugin.stars)}</span>
        </div>
        <div class="flex items-center gap-1">
          <Download class="h-3.5 w-3.5" />
          <span>{formatNumber(plugin.downloads)}</span>
        </div>
      </div>
    {:else}
      <!-- 分类标签（已安装列表） -->
      <span
        class="rounded bg-neutral-100 px-2 py-0.5 text-xs text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
      >
        {plugin.category}
      </span>
    {/if}

    <!-- 右侧：分类和安装按钮 -->
    <div class="flex items-center gap-2">
      <!-- 分类标签（仅在市场列表显示） -->
      {#if showStats}
        <span
          class="rounded bg-neutral-100 px-2 py-0.5 text-xs text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
        >
          {plugin.category}
        </span>
      {/if}

      <!-- 安装按钮 -->
      {#if plugin.id}
        <button
          class="flex items-center gap-1 rounded bg-blue-500 px-3 py-1 text-xs text-white transition-colors hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
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
