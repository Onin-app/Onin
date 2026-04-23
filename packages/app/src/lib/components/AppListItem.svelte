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
      .filter((kw) => !kw.disabled && kw.name !== app.name)
      .slice(0, 3)
      .map((kw) => kw.name),
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
  <div class="relative mr-2 h-8 w-8 flex-shrink-0">
    {#if app.icon}
      {#if app.icon_type === "Base64"}
        <img
          src={app.icon.startsWith("data:")
            ? app.icon
            : `data:image/png;base64,${app.icon}`}
          class="inline-block h-8 w-8"
          alt=""
        />
      {:else if app.icon_type === "Url"}
        <img src={app.icon} class="inline-block h-8 w-8" alt="" />
      {:else}
        <!-- 所有其他情况使用 Phosphor 图标 -->
        <div
          class="flex h-8 w-8 items-center justify-center rounded-md bg-gray-200 dark:bg-gray-700"
        >
          <PhosphorIcon icon={app.icon} class="h-6 w-6" />
        </div>
      {/if}
    {:else if app.source === "Application"}
      <!-- 应用程序默认图标 -->
      <div
        class="flex h-8 w-8 items-center justify-center rounded-md bg-blue-100 dark:bg-blue-900"
      >
        <PhosphorIcon
          icon="cube"
          class="h-5 w-5 text-blue-600 dark:text-blue-400"
        />
      </div>
    {/if}

    {#if triggerMode}
      <span
        class="absolute -right-1 -bottom-1 flex h-3.5 w-3.5 items-center justify-center rounded-full border border-neutral-300 bg-white text-neutral-500 shadow-sm dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
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
    {/if}
  </div>
  <div class="relative min-w-0 flex-1">
    <!-- 来源标签（绝对定位固定右上角） -->
    <span
      class="absolute top-0 right-0 rounded-md bg-neutral-200 px-1.5 py-0.5 text-xs text-neutral-700 dark:bg-neutral-700 dark:text-neutral-300"
    >
      {app.source_display || app.source}
    </span>

    <!-- 第一行：名称 + 别名 -->
    <div class="flex items-center gap-2 overflow-hidden pr-24">
      <span class="truncate">{app.name}</span>

      <!-- 别名标签 -->
      {#if displayAliases.length > 0}
        <div class="flex min-w-0 items-center gap-1 overflow-hidden">
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
    </div>

    <!-- 第二行：描述信息 -->
    {#if app.description}
      <span
        class="mt-0.5 block truncate pr-24 text-sm text-neutral-500 dark:text-neutral-400"
      >
        {app.description}
      </span>
    {:else if app.source !== "Command" && app.path}
      <span
        class="mt-0.5 block truncate pr-24 text-xs text-neutral-400 dark:text-neutral-500"
      >
        {app.path}
      </span>
    {/if}
  </div>
</button>
