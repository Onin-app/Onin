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
  }

  let {
    title,
    description,
    icon = "calculator",
    isSelected = false,
    triggerMode,
    onClick,
  }: Props = $props();

  const triggerModeValue = $derived(
    triggerMode === "matched"
      ? "matched"
      : triggerMode === "preview"
        ? "preview"
        : "function",
  );
</script>

<button
  role="option"
  aria-selected={isSelected}
  class="flex w-full overflow-hidden rounded p-2 text-left text-2xl transition-all duration-200 {!isSelected
    ? 'hover:bg-neutral-200 dark:hover:bg-neutral-700'
    : ''} {isSelected ? 'bg-neutral-300 dark:bg-neutral-600' : ''}"
  onclick={onClick}
>
  <!-- 图标：与 AppListItem 一致的尺寸和样式 -->
  <div class="relative mr-2 h-8 w-8 flex-shrink-0">
    <div
      class="flex h-8 w-8 items-center justify-center rounded-md bg-blue-100 dark:bg-blue-900"
    >
      <PhosphorIcon {icon} class="h-5 w-5 text-blue-600 dark:text-blue-400" />
    </div>
    <span
      class="absolute -right-1 -bottom-1 flex h-3.5 w-3.5 items-center justify-center rounded-full border border-neutral-300 bg-white text-neutral-500 shadow-sm dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
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
          <path d="M2.5 5.5h11" fill="none" stroke="currentColor" stroke-width="1.2" />
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

  <div class="relative min-w-0 flex-1">
    <!-- 来源标签 -->
    <span
      class="absolute top-0 right-0 rounded-md bg-neutral-200 px-1.5 py-0.5 text-xs text-neutral-700 dark:bg-neutral-700 dark:text-neutral-300"
    >
      Extension
    </span>

    <!-- 第一行：计算结果 -->
    <div class="flex items-center gap-2 overflow-hidden pr-24">
      <span class="truncate font-medium text-blue-600 dark:text-blue-400"
        >{title}</span
      >
    </div>

    <!-- 第二行：描述信息 -->
    <span
      class="mt-0.5 block truncate pr-24 text-sm text-neutral-500 dark:text-neutral-400"
    >
      {description}
    </span>
  </div>
</button>
