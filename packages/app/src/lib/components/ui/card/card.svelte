<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";

  let {
    class: className,
    children,
    ...restProps
  }: HTMLAttributes<HTMLDivElement> = $props();
</script>

<div
  class={cn(
    "bg-card text-card-foreground rounded-xl border shadow-xs",
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</div>
