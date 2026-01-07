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
        src={`data:image/png;base64,${app.icon}`}
        class="mr-2 inline-block h-8 w-8"
        alt=""
      />
    {:else}
      <!-- 所有其他情况使用 Phosphor 图标 -->
      <div
        class="mr-2 flex h-8 w-8 items-center justify-center rounded-md bg-gray-200 dark:bg-gray-700"
      >
        <PhosphorIcon icon={app.icon} class="h-6 w-6" />
      </div>
    {/if}
  {/if}
  <div class="flex flex-1 flex-col">
    <div class="flex items-center justify-between">
      <span>
        {app.name}
      </span>
      <span
        class="rounded-md bg-neutral-200 px-1.5 py-0.5 text-xs text-neutral-700 dark:bg-neutral-700 dark:text-neutral-300"
      >
        {app.source_display || app.source}
      </span>
    </div>
    {#if app.source !== "Command"}
      <span class="text-neutral-399 text-xs dark:text-neutral-500">
        {app.path}
      </span>
    {/if}
  </div>
</button>
