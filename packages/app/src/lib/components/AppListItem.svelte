<script lang="ts">
  /**
   * AppListItem Component
   *
   * 单个应用项的渲染组件
   * 遵循单一职责原则，只负责渲染单个应用项
   */
  import type { LaunchableItem } from "$lib/type";
  import PhosphorIcon from "./PhosphorIcon.svelte";

  interface Props {
    app: LaunchableItem;
    isSelected: boolean;
    onClick: () => void;
    onHover?: (e: MouseEvent) => void;
  }

  let { app, isSelected, onClick, onHover }: Props = $props();

  const triggerMode = $derived(
    app.trigger_mode === "matched"
      ? "matched"
      : app.trigger_mode === "preview"
        ? "preview"
        : app.source === "Extension"
          ? "function"
          : null,
  );

  // 获取需要显示的别名（排除与名称相同的关键词，最多显示3个）
  const displayAliases = $derived(
    app.keywords
      .filter(
        (kw) =>
          !kw.disabled && kw.name.toLowerCase() !== app.name.toLowerCase(),
      )
      .slice(0, 3)
      .map((kw) => kw.name),
  );
</script>

<button
  role="option"
  aria-selected={isSelected}
  class="group flex w-full cursor-pointer items-center rounded-xl px-3 py-2 text-left transition-colors select-none {isSelected
    ? 'bg-accent text-accent-foreground shadow-2xs'
    : 'text-foreground hover:bg-accent/40'}"
  onclick={onClick}
  onmouseenter={onHover}
>
  <div class="relative mr-3 h-8 w-8 flex-shrink-0">
    {#if app.icon}
      {#if app.icon_type === "Base64"}
        <img
          src={app.icon.startsWith("data:")
            ? app.icon
            : `data:image/png;base64,${app.icon}`}
          class="inline-block h-8 w-8 rounded-lg object-contain"
          alt=""
        />
      {:else if app.icon_type === "Url"}
        <img
          src={app.icon}
          class="inline-block h-8 w-8 rounded-lg object-contain"
          alt=""
        />
      {:else}
        <div
          class="bg-muted text-muted-foreground flex h-8 w-8 items-center justify-center rounded-lg"
        >
          <PhosphorIcon icon={app.icon} class="h-5 w-5" />
        </div>
      {/if}
    {:else if app.source === "Application"}
      <div
        class="bg-primary/10 text-primary flex h-8 w-8 items-center justify-center rounded-lg"
      >
        <PhosphorIcon icon="cube" class="h-5 w-5" />
      </div>
    {/if}

    {#if triggerMode}
      <span
        class="border-border bg-background text-muted-foreground absolute -right-1 -bottom-1 flex h-3.5 w-3.5 items-center justify-center rounded-full border shadow-xs"
      >
        {#if triggerMode === "function"}
          <svg viewBox="0 0 16 16" class="h-2.5 w-2.5" aria-hidden="true">
            <rect
              x="2.5"
              y="3"
              width="11"
              height="10"
              rx="2"
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
            />
            <path
              d="M2.5 5.5h11"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
            />
          </svg>
        {:else if triggerMode === "matched"}
          <svg viewBox="0 0 16 16" class="h-2.5 w-2.5" aria-hidden="true">
            <circle
              cx="8"
              cy="8"
              r="4.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
            />
            <circle cx="8" cy="8" r="1.6" fill="currentColor" />
          </svg>
        {:else if triggerMode === "preview"}
          <svg viewBox="0 0 16 16" class="h-2.5 w-2.5" aria-hidden="true">
            <path
              d="M2.2 8s2.1-3 5.8-3 5.8 3 5.8 3-2.1 3-5.8 3-5.8-3Z"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
              stroke-linejoin="round"
            />
            <circle cx="8" cy="8" r="1.6" fill="currentColor" />
          </svg>
        {/if}
      </span>
    {/if}
  </div>
  <div class="relative min-w-0 flex-1">
    <!-- 来源标签（右上角低对比度优雅辅助文字） -->
    <span
      class="text-muted-foreground/60 group-hover:text-muted-foreground/90 absolute top-0 right-0 font-mono text-[11px] font-normal transition-opacity"
    >
      {app.source_display || (app.source === "Internal" ? "内置" : app.source)}
    </span>

    <!-- 第一行：名称 + 别名 -->
    <div class="flex items-center gap-2 overflow-hidden pr-20">
      <span class="text-foreground truncate text-sm font-semibold"
        >{app.name}</span
      >

      <!-- 别名标签 -->
      {#if displayAliases.length > 0}
        <div class="flex min-w-0 items-center gap-1 overflow-hidden">
          {#each displayAliases as alias}
            <span
              class="bg-muted/70 py-0.2 text-muted-foreground flex-shrink-0 rounded px-1.5 text-[10px]"
            >
              {alias}
            </span>
          {/each}
          {#if app.keywords.filter((kw) => !kw.disabled && kw.name.toLowerCase() !== app.name.toLowerCase()).length > 3}
            <span class="text-muted-foreground/70 text-[10px]">
              +{app.keywords.filter(
                (kw) =>
                  !kw.disabled &&
                  kw.name.toLowerCase() !== app.name.toLowerCase(),
              ).length - 3}
            </span>
          {/if}
        </div>
      {/if}
    </div>

    <!-- 第二行：描述信息 -->
    {#if app.description}
      <span
        class="text-muted-foreground/80 mt-0.5 block truncate pr-20 text-xs"
      >
        {app.description}
      </span>
    {:else if app.source !== "Command" && app.path}
      <span
        class="text-muted-foreground/60 mt-0.5 block truncate pr-20 font-mono text-[11px]"
      >
        {app.path}
      </span>
    {/if}
  </div>
</button>
