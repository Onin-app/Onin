<script lang="ts">
  import { Select as SelectPrimitive } from "bits-ui";
  import { Check } from "phosphor-svelte";
  import { cn } from "$lib/utils";

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...restProps
  }: SelectPrimitive.ItemProps = $props();
</script>

<SelectPrimitive.Item
  bind:ref
  class={cn(
    "data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground relative flex w-full cursor-pointer items-center rounded-sm py-1.5 pr-8 pl-2 text-sm transition-[transform,background-color,color] duration-100 ease-out outline-none select-none active:scale-[0.98] data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
    className,
  )}
  {...restProps}
>
  {#snippet children(itemProps)}
    {@render children?.(itemProps)}
    {#if itemProps?.selected}
      <span
        class="absolute right-2 flex h-3.5 w-3.5 items-center justify-center"
      >
        <Check class="h-4 w-4" />
      </span>
    {/if}
  {/snippet}
</SelectPrimitive.Item>
