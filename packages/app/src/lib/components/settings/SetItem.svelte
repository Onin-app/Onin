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
  class="border-border/40 flex min-h-[44px] items-center justify-between gap-4 border-b py-2.5 last:border-0"
>
  <div class="flex flex-col gap-0.5">
    <span class="text-foreground text-sm font-medium">
      {title}
    </span>
    {#if description}
      <span class="text-muted-foreground text-xs">
        {description}
      </span>
    {/if}
  </div>
  <div class="flex items-center">
    {@render content()}
  </div>
</div>
