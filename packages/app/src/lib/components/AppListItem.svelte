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
  }

  let { app, isSelected, onClick }: Props = $props();

  // 获取需要显示的别名（排除与名称相同的关键词，最多显示3个）
  const displayAliases = $derived(
    app.keywords
      .filter((kw) => !kw.disabled && kw.name !== app.name)
      .slice(0, 3)
      .map((kw) => kw.name),
  );
</script>

<button
  role="option"
  aria-selected={isSelected}
  class="flex w-full rounded p-2 text-left text-2xl transition-all duration-200 {!isSelected
    ? 'hover:bg-neutral-200 dark:hover:bg-neutral-700'
    : ''} {isSelected ? 'bg-neutral-300 dark:bg-neutral-600' : ''}"
  onclick={onClick}
>
  {#if app.icon}
    {#if app.icon_type === "Base64"}
      <img
        src={app.icon.startsWith("data:")
          ? app.icon
          : `data:image/png;base64,${app.icon}`}
        class="mr-2 inline-block h-8 w-8 flex-shrink-0"
        alt=""
      />
    {:else if app.icon_type === "Url"}
      <img
        src={app.icon}
        class="mr-2 inline-block h-8 w-8 flex-shrink-0"
        alt=""
      />
    {:else}
      <!-- 所有其他情况使用 Phosphor 图标 -->
      <div
        class="mr-2 flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-md bg-gray-200 dark:bg-gray-700"
      >
        <PhosphorIcon icon={app.icon} class="h-6 w-6" />
      </div>
    {/if}
  {/if}
  <div class="flex min-w-0 flex-1 flex-col">
    <!-- 第一行：名称 + 别名 + 来源 -->
    <div class="flex items-center gap-2">
      <span class="flex-shrink-0">{app.name}</span>

      <!-- 别名标签 -->
      {#if displayAliases.length > 0}
        <div class="flex min-w-0 flex-1 items-center gap-1 overflow-hidden">
          {#each displayAliases as alias}
            <span
              class="flex-shrink-0 rounded bg-neutral-200/70 px-1.5 py-0.5 text-xs text-neutral-500 dark:bg-neutral-700/70 dark:text-neutral-400"
            >
              {alias}
            </span>
          {/each}
          {#if app.keywords.filter((kw) => !kw.disabled && kw.name !== app.name).length > 3}
            <span class="text-xs text-neutral-400 dark:text-neutral-500">
              +{app.keywords.filter(
                (kw) => !kw.disabled && kw.name !== app.name,
              ).length - 3}
            </span>
          {/if}
        </div>
      {/if}

      <!-- 来源标签 -->
      <span
        class="ml-auto flex-shrink-0 rounded-md bg-neutral-200 px-1.5 py-0.5 text-xs text-neutral-700 dark:bg-neutral-700 dark:text-neutral-300"
      >
        {app.source_display || app.source}
      </span>
    </div>

    <!-- 第二行：描述信息 -->
    {#if app.description}
      <span
        class="mt-0.5 truncate text-sm text-neutral-500 dark:text-neutral-400"
      >
        {app.description}
      </span>
    {:else if app.source !== "Command" && app.path}
      <span
        class="mt-0.5 truncate text-xs text-neutral-400 dark:text-neutral-500"
      >
        {app.path}
      </span>
    {/if}
  </div>
</button>
