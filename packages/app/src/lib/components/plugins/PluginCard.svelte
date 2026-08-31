<script lang="ts">
  /**
   * PluginCard Component
   *
   * 单个插件卡片组件
   * 显示插件信息和操作按钮
   */
  import { Button } from "$lib/components/ui/button";
  import { Switch } from "$lib/components/ui/switch";
  import { Badge } from "$lib/components/ui/badge";
  import { Card } from "$lib/components/ui/card";
  import {
    Popover,
    PopoverTrigger,
    PopoverContent,
    PopoverClose,
  } from "$lib/components/ui/popover";
  import {
    PuzzlePiece,
    Gear,
    Trash,
    GithubLogo,
    WarningCircle,
  } from "phosphor-svelte";
  import { getPluginIconUrl, type PluginIconInfo } from "$lib/utils/pluginIcon";
  import {
    formatPluginVersion,
    isValidPluginVersion,
  } from "$lib/utils/pluginVersion";
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

<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
<Card
  class="group border-border/60 bg-card hover:border-border flex cursor-pointer flex-col rounded-2xl p-3.5 shadow-2xs transition-[border-color,box-shadow,transform] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] hover:shadow-xs active:scale-[0.99]"
  onclick={() => onViewDetail(plugin.id)}
  role="button"
  tabindex={0}
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
      class="bg-muted/60 text-muted-foreground border-border/50 relative flex h-13 w-13 shrink-0 cursor-pointer items-center justify-center rounded-xl border shadow-xs transition-transform duration-140 hover:scale-105 active:scale-95"
      onclick={(e: MouseEvent) => {
        e.stopPropagation();
        onExecute(plugin.id);
      }}
    >
      {#await getPluginIconUrl(plugin)}
        <PuzzlePiece class="h-6 w-6 animate-pulse" />
      {:then iconUrl}
        {#if iconUrl && !imageErrors.has(plugin.id)}
          <img
            src={iconUrl}
            alt={plugin.name}
            class="h-9 w-9 rounded-md object-contain"
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
          <PuzzlePiece class="h-6 w-6" />
        {/if}
      {:catch}
        <PuzzlePiece class="h-6 w-6" />
      {/await}

      <!-- 来源标识 -->
      {#if plugin.install_source === "local"}
        <Badge
          class="absolute -top-1 -right-1 rounded-md px-1 py-0 text-[9px] font-medium shadow-xs"
        >
          本地
        </Badge>
      {/if}
    </button>

    <!-- 右侧信息 -->
    <div class="flex min-w-0 flex-1 flex-col">
      <div class="mb-1 flex items-start justify-between gap-2">
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
        <Button
          variant="ghost"
          size="icon"
          class="text-muted-foreground hover:text-foreground h-6 w-6 shrink-0 opacity-0 transition-opacity duration-120 group-hover:opacity-100"
          onclick={(e: MouseEvent) => {
            e.stopPropagation();
          }}
          aria-label="查看 GitHub"
        >
          <GithubLogo class="h-3.5 w-3.5" />
        </Button>
      </div>
      <p class="text-muted-foreground/75 line-clamp-1 text-xs leading-normal">
        {plugin.description}
      </p>
    </div>
  </div>

  <!-- 作者和 ID -->
  <div
    class="text-muted-foreground/60 mb-2.5 flex items-center justify-between gap-2 text-xs"
  >
    {#if plugin.author}
      <span class="truncate text-[11.5px]">{plugin.author}</span>
    {/if}
    <span class="shrink-0 font-mono text-[10px]">ID: {plugin.id}</span>
  </div>

  <!-- 底部：操作按钮 -->
  <div
    class="border-border/40 flex items-center justify-between border-t pt-2.5"
  >
    <div></div>

    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="flex items-center gap-1.5"
      onclick={(e: MouseEvent) => e.stopPropagation()}
    >
      <!-- 启用/禁用开关 -->
      <Switch
        checked={plugin.enabled !== false}
        onCheckedChange={(checked) => {
          onToggle(plugin.dir_name || plugin.id, checked);
        }}
      />

      <!-- 设置按钮 -->
      {#if plugin.settings && plugin.settings.fields.length > 0}
        <Button
          variant="ghost"
          size="icon"
          class="text-muted-foreground hover:text-foreground h-7 w-7 rounded-lg transition-transform duration-120 active:scale-90"
          onclick={(e: MouseEvent) => {
            e.stopPropagation();
            onSettings(plugin);
          }}
          aria-label="插件设置"
        >
          <Gear class="h-3.5 w-3.5" />
        </Button>
      {/if}

      <!-- 卸载按钮 -->
      <Popover>
        <PopoverTrigger>
          <Button
            variant="ghost"
            size="icon"
            class="text-muted-foreground hover:text-destructive h-7 w-7 rounded-lg transition-transform duration-120 active:scale-90"
            aria-label="卸载插件"
          >
            <Trash class="h-3.5 w-3.5" />
          </Button>
        </PopoverTrigger>
        <PopoverContent class="w-64 rounded-2xl p-4 shadow-xl">
          <div class="mb-3 flex items-center gap-2 text-sm font-medium">
            <WarningCircle size={18} class="text-destructive shrink-0" />
            <span>确认卸载插件 {plugin.name}？</span>
          </div>
          <div class="flex justify-end gap-2">
            <PopoverClose>
              <Button
                variant="outline"
                size="sm"
                class="h-7 rounded-lg text-xs"
              >
                取消
              </Button>
            </PopoverClose>
            <Button
              variant="destructive"
              size="sm"
              class="h-7 rounded-lg text-xs"
              onclick={(e: MouseEvent) => {
                e.stopPropagation();
                onUninstall(plugin.id);
              }}
            >
              确认卸载
            </Button>
          </div>
        </PopoverContent>
      </Popover>
    </div>
  </div>
</Card>
