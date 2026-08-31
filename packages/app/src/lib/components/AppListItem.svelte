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
  class="group flex w-full cursor-pointer items-center justify-between gap-3 rounded-xl px-3.5 py-2.5 text-left transition-[transform,background-color,box-shadow] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none select-none active:scale-[0.985] {isSelected
    ? 'bg-accent text-accent-foreground shadow-active-item ring-border/40 ring-1'
    : 'text-foreground hover:bg-accent/40'}"
  onclick={onClick}
  onmouseenter={onHover}
>
  <!-- 左侧：图标 + 主内容（标题与描述） -->
  <div class="flex min-w-0 flex-1 items-center gap-3">
    <div class="relative h-8 w-8 flex-shrink-0">
      {#if app.icon}
        {#if app.icon_type === "Base64"}
          <img
            src={app.icon.startsWith("data:")
              ? app.icon
              : `data:image/png;base64,${app.icon}`}
            class="inline-block h-8 w-8 rounded-lg object-contain shadow-xs transition-transform duration-140 group-hover:scale-[1.02]"
            alt=""
          />
        {:else if app.icon_type === "Url"}
          <img
            src={app.icon}
            class="inline-block h-8 w-8 rounded-lg object-contain shadow-xs transition-transform duration-140 group-hover:scale-[1.02]"
            alt=""
          />
        {:else}
          <div
            class="bg-muted text-muted-foreground border-border/40 flex h-8 w-8 items-center justify-center rounded-lg border shadow-xs transition-transform duration-140 group-hover:scale-[1.02]"
          >
            <PhosphorIcon icon={app.icon} class="h-4.5 w-4.5" />
          </div>
        {/if}
      {:else if app.source === "Application"}
        <div
          class="bg-primary/10 text-primary border-primary/20 flex h-8 w-8 items-center justify-center rounded-lg border shadow-xs transition-transform duration-140 group-hover:scale-[1.02]"
        >
          <PhosphorIcon icon="cube" class="h-4.5 w-4.5" />
        </div>
      {/if}

      {#if triggerMode}
        <span
          class="border-border bg-background/95 text-muted-foreground absolute -right-1 -bottom-1 flex h-3.5 w-3.5 items-center justify-center rounded-full border shadow-xs backdrop-blur-xs"
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

    <!-- 文本信息 -->
    <div class="min-w-0 flex-1">
      <!-- 第一行：名称 + 别名 -->
      <div class="flex items-center gap-2">
        <span
          class="text-foreground truncate text-sm font-medium tracking-tight"
          >{app.name}</span
        >

        <!-- 别名标签（低对比度，轻量克制） -->
        {#if displayAliases.length > 0}
          <div class="flex min-w-0 items-center gap-1 overflow-hidden">
            {#each displayAliases as alias}
              <span
                class="bg-muted/50 text-muted-foreground/70 py-0.2 flex-shrink-0 rounded px-1.5 font-mono text-[10px]"
              >
                {alias}
              </span>
            {/each}
            {#if app.keywords.filter((kw) => !kw.disabled && kw.name.toLowerCase() !== app.name.toLowerCase()).length > 3}
              <span class="text-muted-foreground/50 text-[10px]">
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

      <!-- 第二行：描述信息（弱化颜色，减小视觉噪音） -->
      {#if app.description}
        <span
          class="text-muted-foreground/60 mt-0.5 block truncate text-[11.5px] leading-tight"
        >
          {app.description}
        </span>
      {:else if app.source !== "Command" && app.path}
        <span
          class="text-muted-foreground/45 mt-0.5 block truncate font-mono text-[10.5px] leading-tight"
        >
          {app.path}
        </span>
      {/if}
    </div>
  </div>

  <!-- 右侧：来源标签（垂直居中对齐，优雅整齐） -->
  <div class="flex flex-shrink-0 items-center pl-2">
    <span
      class="font-mono text-[11px] transition-colors duration-120 {isSelected
        ? 'text-muted-foreground/90 font-medium'
        : 'text-muted-foreground/50 group-hover:text-muted-foreground/80'}"
    >
      {app.source_display || (app.source === "Internal" ? "内置" : app.source)}
    </span>
  </div>
</button>
