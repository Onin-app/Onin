<script lang="ts">
  import { X } from "phosphor-svelte";
  import FileTypeIcon from "./FileTypeIcon.svelte";

  interface Props {
    file: File;
    onRemove: () => void;
  }

  let { file, onRemove }: Props = $props();
</script>

<div
  class="group border-border/70 bg-muted/70 hover:bg-muted relative inline-flex h-[32px] max-w-[240px] items-center gap-1.5 rounded-lg border px-2 pr-7 text-xs shadow-2xs backdrop-blur-xs transition-[transform,background-color,border-color] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] select-none"
  role="group"
  aria-label="附件文件: {file.name}"
>
  <FileTypeIcon
    fileType={file.type}
    fileName={file.name}
    class="size-4.5 flex-shrink-0"
  />
  <span class="text-foreground/90 truncate font-mono text-xs font-medium">
    {file.name}
  </span>
  <button
    onclick={onRemove}
    class="text-muted-foreground hover:bg-destructive/15 hover:text-destructive absolute top-1/2 right-1 flex h-4.5 w-4.5 -translate-y-1/2 cursor-pointer items-center justify-center rounded-md transition-[transform,background-color,color] duration-120 ease-out active:scale-90"
    aria-label="移除文件 {file.name}"
  >
    <X class="size-2.5" weight="bold" />
  </button>
</div>
