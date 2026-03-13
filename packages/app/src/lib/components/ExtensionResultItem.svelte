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
    onClick: () => void;
  }

  let {
    title,
    description,
    icon = "calculator",
    isSelected = false,
    onClick,
  }: Props = $props();
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
  <div
    class="mr-2 flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-md bg-blue-100 dark:bg-blue-900"
  >
    <PhosphorIcon {icon} class="h-5 w-5 text-blue-600 dark:text-blue-400" />
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
