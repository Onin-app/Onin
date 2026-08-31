<script lang="ts">
  /**
   * Extension Result Item Component
   *
   * 显示 Extension 计算结果的列表项
   * 样式与 AppListItem 保持一致
   */
  import PhosphorIcon from "./PhosphorIcon.svelte";

  interface Props {
    title: string;
    description: string;
    icon?: string;
    isSelected?: boolean;
    triggerMode?: "matched" | "preview";
    onClick: () => void;
    onHover?: (e: MouseEvent) => void;
  }

  let {
    title,
    description,
    icon = "calculator",
    isSelected = false,
    triggerMode,
    onClick,
    onHover,
  }: Props = $props();

  const triggerModeValue = $derived(
    triggerMode === "matched"
      ? "matched"
      : triggerMode === "preview"
        ? "preview"
        : "function",
  );

  const colorSwatch = $derived(
    /^#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/.test(title) ? title : null,
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
  <!-- 左侧：图标 + 预览计算结果与描述 -->
  <div class="flex min-w-0 flex-1 items-center gap-3">
    <div class="relative h-8 w-8 flex-shrink-0">
      <div
        class="flex h-8 w-8 items-center justify-center rounded-lg border border-blue-500/20 bg-blue-500/10 text-blue-600 shadow-xs transition-transform duration-140 group-hover:scale-[1.02] dark:text-blue-400"
      >
        <PhosphorIcon {icon} class="h-4.5 w-4.5" />
      </div>
      <span
        class="border-border bg-background/95 text-muted-foreground absolute -right-1 -bottom-1 flex h-3.5 w-3.5 items-center justify-center rounded-full border shadow-xs backdrop-blur-xs"
      >
        {#if triggerModeValue === "function"}
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
        {:else if triggerModeValue === "matched"}
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
        {:else if triggerModeValue === "preview"}
          <svg viewBox="0 0 16 16" class="h-2.5 w-2.5" aria-hidden="true">
            <path
              d="M2.2 8s2.1-3 5.8-3 5.8 3 5.8 3-2.1 3-5.8 3-5.8-3-5.8-3Z"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
              stroke-linejoin="round"
            />
            <circle cx="8" cy="8" r="1.6" fill="currentColor" />
          </svg>
        {/if}
      </span>
    </div>

    <!-- 文本信息 -->
    <div class="min-w-0 flex-1">
      <!-- 第一行：计算结果 / 预览内容 -->
      <div class="flex items-center gap-2">
        {#if colorSwatch}
          <span
            class="border-border/80 h-3.5 w-3.5 shrink-0 rounded-md border shadow-xs"
            style={`background-color: ${colorSwatch}`}
            aria-hidden="true"
          ></span>
        {/if}
        <span
          class="truncate text-sm font-medium tracking-tight text-blue-600 dark:text-blue-400"
          >{title}</span
        >
      </div>

      <!-- 第二行：描述信息 -->
      <span
        class="text-muted-foreground/60 mt-0.5 block truncate font-mono text-[11.5px] leading-tight"
      >
        {description}
      </span>
    </div>
  </div>

  <!-- 右侧：来源标签 -->
  <div class="flex flex-shrink-0 items-center pl-2">
    <span
      class="font-mono text-[11px] transition-colors duration-120 {isSelected
        ? 'text-muted-foreground/90 font-medium'
        : 'text-muted-foreground/50 group-hover:text-muted-foreground/80'}"
    >
      扩展
    </span>
  </div>
</button>
