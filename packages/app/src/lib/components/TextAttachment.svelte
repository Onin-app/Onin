<script lang="ts">
  import { PencilSimple, X } from "phosphor-svelte";

  interface Props {
    text: string;
    onEdit: () => void;
    onRemove: () => void;
  }

  let { text, onEdit, onRemove }: Props = $props();

  // 截取文本预览（最多显示50个字符）
  const preview = $derived(
    text.length > 50 ? text.substring(0, 50) + "..." : text,
  );
</script>

<div
  class="group border-border/70 bg-muted/70 hover:bg-muted relative inline-flex h-[32px] max-w-[280px] items-center gap-1.5 rounded-lg border px-2.5 pr-7 text-xs shadow-2xs backdrop-blur-xs transition-[transform,background-color,border-color] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] select-none"
  role="group"
  aria-label="附件文本"
>
  <button
    class="flex min-w-0 flex-1 cursor-pointer items-center gap-1.5 overflow-hidden text-left outline-none"
    onclick={onEdit}
    aria-label="编辑文本"
  >
    <PencilSimple class="text-muted-foreground size-3.5 shrink-0" />
    <span class="text-foreground/90 truncate font-mono text-xs font-medium">
      {preview}
    </span>
  </button>
  <button
    onclick={onRemove}
    class="text-muted-foreground hover:bg-destructive/15 hover:text-destructive absolute top-1/2 right-1 flex h-4.5 w-4.5 -translate-y-1/2 cursor-pointer items-center justify-center rounded-md transition-[transform,background-color,color] duration-120 ease-out active:scale-90"
    aria-label="移除文本附件"
  >
    <X class="size-2.5" weight="bold" />
  </button>
</div>
