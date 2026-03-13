<script lang="ts">
  /**
   * PluginCard Component
   *
   * 单个插件卡片组件
   * 显示插件信息和操作按钮
   */
  import { Button, Switch, Popover } from "bits-ui";
  import {
    PuzzlePiece,
    Gear,
    Trash,
    GithubLogo,
    WarningCircle,
  } from "phosphor-svelte";
  import { getPluginIconUrl, type PluginIconInfo } from "$lib/utils/pluginIcon";
  import type { PluginManifest } from "$lib/composables/usePluginList.svelte";

  interface Props {
    plugin: PluginManifest;
    imageErrors: Set<string>;
    onExecute: (pluginId: string) => void;
    onToggle: (pluginId: string, enabled: boolean) => void;
    onSettings: (plugin: PluginManifest) => void;
    onUninstall: (pluginId: string) => void;
    onViewDetail: (pluginId: string) => void;
    onImageError: (pluginId: string) => void;
  }

  let {
    plugin,
    imageErrors,
    onExecute,
    onToggle,
    onSettings,
    onUninstall,
    onViewDetail,
    onImageError,
  }: Props = $props();
</script>

<div
  class="group flex cursor-pointer flex-col rounded-lg border border-neutral-200 bg-white p-3 transition-all hover:border-neutral-300 hover:shadow-sm dark:border-neutral-700 dark:bg-neutral-900 dark:hover:border-neutral-600"
  onclick={() => onViewDetail(plugin.id)}
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onViewDetail(plugin.id);
    }
  }}
>
  <!-- 顶部：图标和信息 -->
  <div class="mb-2 flex items-start gap-3">
    <!-- 左侧图标 -->
    <button
      class="relative flex h-16 w-16 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-neutral-100 to-neutral-200 transition-transform hover:scale-105 dark:from-neutral-800 dark:to-neutral-700"
      onclick={(e: MouseEvent) => {
        e.stopPropagation();
        onExecute(plugin.id);
      }}
    >
      {#await getPluginIconUrl(plugin)}
        <PuzzlePiece class="h-8 w-8 animate-pulse" />
      {:then iconUrl}
        {#if iconUrl && !imageErrors.has(plugin.id)}
          <img
            src={iconUrl}
            alt={plugin.name}
            class="h-12 w-12 rounded object-contain"
            onerror={() => {
              console.error(
                "Failed to load icon:",
                plugin.icon,
                "URL:",
                iconUrl,
              );
              onImageError(plugin.id);
            }}
          />
        {:else}
          <PuzzlePiece class="h-8 w-8" />
        {/if}
      {:catch}
        <PuzzlePiece class="h-8 w-8" />
      {/await}

      <!-- 来源标识 -->
      {#if plugin.install_source === "local"}
        <span
          class="absolute -top-1 -right-1 rounded bg-orange-500 px-1.5 py-0.5 text-[10px] font-medium text-white shadow-sm"
        >
          本地
        </span>
      {/if}
    </button>

    <!-- 右侧信息 -->
    <div class="flex min-w-0 flex-1 flex-col">
      <div class="mb-1 flex items-start justify-between gap-2">
        <div class="flex min-w-0 items-baseline gap-2">
          <h3 class="truncate text-base leading-tight font-semibold">
            {plugin.name}
          </h3>
          {#if plugin.version}
            <span class="shrink-0 text-xs text-neutral-400"
              >v{plugin.version}</span
            >
          {/if}
        </div>
        <Button.Root
          class="shrink-0 rounded p-1 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-neutral-100 dark:hover:bg-neutral-800"
          onclick={(e: MouseEvent) => {
            e.stopPropagation();
          }}
          aria-label="查看 GitHub"
        >
          <GithubLogo class="h-4 w-4" />
        </Button.Root>
      </div>
      <p class="line-clamp-2 text-sm text-neutral-500 dark:text-neutral-400">
        {plugin.description}
      </p>
    </div>
  </div>

  <!-- 作者和 ID -->
  <div
    class="mb-2 flex items-center justify-between gap-2 text-xs text-neutral-400"
  >
    {#if plugin.author}
      <span class="truncate">{plugin.author}</span>
    {/if}
    <span class="shrink-0 text-neutral-300 dark:text-neutral-600"
      >ID: {plugin.id}</span
    >
  </div>

  <!-- 底部：操作按钮 -->
  <div
    class="flex items-center justify-between border-t border-neutral-200 pt-2 dark:border-neutral-700"
  >
    <div></div>

    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="flex items-center gap-1"
      onclick={(e: MouseEvent) => e.stopPropagation()}
    >
      <!-- 启用/禁用开关 -->
      <Switch.Root
        checked={plugin.enabled !== false}
        onCheckedChange={(checked) => {
          onToggle(plugin.dir_name || plugin.id, checked);
        }}
        class="focus-visible:ring-foreground focus-visible:ring-offset-background data-[state=checked]:bg-foreground data-[state=unchecked]:bg-dark-10 data-[state=unchecked]:shadow-mini-inset dark:data-[state=checked]:bg-foreground peer inline-flex h-[20px] min-h-[20px] w-[36px] shrink-0 cursor-pointer items-center rounded-full px-[2px] transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50"
      >
        <Switch.Thumb
          class="bg-background data-[state=unchecked]:shadow-mini dark:border-background/30 dark:bg-foreground dark:shadow-popover pointer-events-none block size-[16px] shrink-0 rounded-full transition-transform data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0 dark:border dark:data-[state=unchecked]:border"
        />
      </Switch.Root>

      <!-- 设置按钮 -->
      {#if plugin.settings && plugin.settings.fields.length > 0}
        <Button.Root
          class="rounded px-2 py-1 text-xs transition-colors hover:bg-neutral-100 dark:hover:bg-neutral-800"
          onclick={(e: MouseEvent) => {
            e.stopPropagation();
            onSettings(plugin);
          }}
          aria-label="插件设置"
        >
          <Gear class="h-4 w-4" />
        </Button.Root>
      {/if}

      <!-- 卸载按钮 -->
      <Popover.Root>
        <Popover.Trigger>
          <Button.Root
            class="rounded px-2 py-1 text-xs transition-colors hover:bg-red-100 hover:text-red-600 dark:hover:bg-red-900/20 dark:hover:text-red-400"
            aria-label="卸载插件"
          >
            <Trash class="h-4 w-4" />
          </Button.Root>
        </Popover.Trigger>
        <Popover.Portal>
          <Popover.Content
            class="border-dark-10 bg-background shadow-popover data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-30 w-full max-w-[328px] origin-(--bits-popover-content-transform-origin) rounded-[12px] border p-4"
            sideOffset={8}
          >
            <div class="mb-2 flex items-center">
              <WarningCircle size={20} class="mr-2" />
              确认卸载插件{plugin.name}？
            </div>
            <div class="flex justify-end">
              <Button.Root
                class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex items-center justify-center px-3 py-1 text-[12px] font-semibold active:scale-[0.98] active:transition-all"
                onclick={(e: MouseEvent) => {
                  e.stopPropagation();
                  onUninstall(plugin.id);
                }}
              >
                确认
              </Button.Root>
            </div>
          </Popover.Content>
        </Popover.Portal>
      </Popover.Root>
    </div>
  </div>
</div>
