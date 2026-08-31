<script lang="ts">
  import { Select as SelectPrimitive } from "bits-ui";
  import { CaretUpDown } from "phosphor-svelte";
  import { cn } from "$lib/utils";

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...restProps
  }: SelectPrimitive.TriggerProps = $props();
</script>

<SelectPrimitive.Trigger
  bind:ref
  class={cn(
    "border-input ring-offset-background placeholder:text-muted-foreground focus:ring-ring flex h-9 w-full cursor-pointer items-center justify-between rounded-md border bg-transparent px-3 py-2 text-sm whitespace-nowrap shadow-xs focus:ring-1 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1",
    className,
  )}
  {...restProps}
>
  {@render children?.()}
  <CaretUpDown class="ml-2 size-4 shrink-0 opacity-50" />
</SelectPrimitive.Trigger>
