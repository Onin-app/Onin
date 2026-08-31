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
  class="border-border/40 flex min-h-[48px] items-center justify-between gap-6 border-b py-3 last:border-0"
>
  <div class="flex min-w-0 flex-1 flex-col gap-0.5">
    <span class="text-foreground text-sm font-medium tracking-tight">
      {title}
    </span>
    {#if description}
      <span class="text-muted-foreground/75 text-xs leading-normal">
        {description}
      </span>
    {/if}
  </div>
  <div class="flex shrink-0 items-center">
    {@render content()}
  </div>
</div>
