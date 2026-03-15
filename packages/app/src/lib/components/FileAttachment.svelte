<script lang="ts">
  import { X } from "phosphor-svelte";
  import FileTypeIcon from "./FileTypeIcon.svelte";

  interface Props {
    file: File;
    onRemove: () => void;
  }

  let { file, onRemove }: Props = $props();
  let isHovered = $state(false);
</script>

<div
  class="group relative inline-flex h-[34px] items-center gap-1.5 rounded-md border border-neutral-300 bg-neutral-50 px-2 pr-8 dark:border-neutral-600 dark:bg-neutral-700"
  onmouseenter={() => (isHovered = true)}
  onmouseleave={() => (isHovered = false)}
  role="button"
  tabindex="0"
>
  <FileTypeIcon
    fileType={file.type}
    fileName={file.name}
    class="size-6 flex-shrink-0"
  />
  <span
    class="max-w-[150px] truncate text-sm font-medium text-neutral-700 dark:text-neutral-200"
  >
    {file.name}
  </span>
  {#if isHovered}
    <button
      onclick={onRemove}
      class="absolute top-1/2 right-1 flex h-5 w-5 -translate-y-1/2 items-center justify-center rounded bg-red-500 text-white hover:bg-red-600 dark:bg-red-600 dark:hover:bg-red-700"
      aria-label="移除文件 {file.name}"
    >
      <X class="size-3" weight="bold" />
    </button>
  {/if}
</div>
