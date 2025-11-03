<script lang="ts">
  interface Props {
    text: string;
    onEdit: () => void;
    onRemove: () => void;
  }

  let { text, onEdit, onRemove }: Props = $props();

  // 截取文本预览（最多显示50个字符）
  const preview = $derived(
    text.length > 50 ? text.substring(0, 50) + "..." : text
  );
</script>

<button
  class="inline-flex h-[34px] cursor-pointer items-center gap-1.5 rounded-md border border-neutral-300 bg-neutral-50 px-3 transition-colors hover:bg-neutral-100 dark:border-neutral-600 dark:bg-neutral-700 dark:hover:bg-neutral-600"
  onclick={onEdit}
  aria-label="编辑文本"
>
  <span
    class="max-w-[200px] truncate text-sm font-medium text-neutral-700 dark:text-neutral-200"
  >
    {preview}
  </span>
</button>
