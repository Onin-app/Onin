<script lang="ts">
  import type { Snippet } from "svelte";

  interface SetItemProps {
    title: string;
    description?: string;
    content: Snippet;
  }

  const { title, description, content }: SetItemProps = $props();
</script>

<div
  class="flex min-h-[44px] items-center justify-between gap-4 border-b border-neutral-100 py-2.5 last:border-0 dark:border-neutral-800"
>
  <div class="flex flex-col gap-1">
    <span class="text-sm font-medium text-neutral-900 dark:text-neutral-100">
      {title}
    </span>
    {#if description}
      <span class="text-xs text-neutral-500 dark:text-neutral-400">
        {description}
      </span>
    {/if}
  </div>
  <div class="flex items-center">
    {@render content()}
  </div>
</div>
