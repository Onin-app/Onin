<script lang="ts">
  import { GithubLogo, Download, Star } from "phosphor-svelte";
  import type { MarketplacePlugin } from "$lib/types/marketplace";

  interface Props {
    plugin: MarketplacePlugin;
    onclick?: () => void;
  }

  let { plugin, onclick }: Props = $props();
  let imageError = $state(false);

  function handleImageError() {
    imageError = true;
  }

  function formatNumber(num: number): string {
    if (num >= 1000) {
      return (num / 1000).toFixed(1) + "k";
    }
    return num.toString();
  }
</script>

<button
  class="group flex flex-col rounded-lg border border-neutral-200 bg-white p-3 text-left transition-all hover:border-neutral-300 hover:shadow-sm dark:border-neutral-700 dark:bg-neutral-900 dark:hover:border-neutral-600"
  {onclick}
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
        />
      {:else}
        <div class="text-2xl">🧩</div>
      {/if}
    </div>

    <!-- 右侧信息 -->
    <div class="flex min-w-0 flex-1 flex-col gap-1">
      <!-- 标题和 GitHub 链接 -->
      <div class="flex items-start justify-between gap-2">
        <h3 class="truncate text-base leading-tight font-semibold">
          {plugin.name}
        </h3>
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
      <p class="line-clamp-2 text-sm text-neutral-500 dark:text-neutral-400">
        {plugin.description}
      </p>

      <!-- 作者和 ID -->
      <div class="flex items-center justify-between text-xs text-neutral-400">
        <span class="truncate">{plugin.author}</span>
        <span class="ml-2 shrink-0">ID: {plugin.id}</span>
      </div>
    </div>
  </div>

  <!-- 底部：统计和分类 -->
  <div
    class="flex items-center justify-between border-t border-neutral-200 pt-2 dark:border-neutral-700"
  >
    <!-- 统计 -->
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

    <!-- 分类标签 -->
    <span
      class="rounded bg-neutral-100 px-2 py-0.5 text-xs text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
    >
      {plugin.category}
    </span>
  </div>
</button>
