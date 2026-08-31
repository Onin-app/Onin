<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";

  interface Props extends HTMLInputAttributes {
    value?: any;
  }

  let {
    class: className,
    value = $bindable(),
    type = "text",
    ...restProps
  }: Props = $props();
</script>

<input
  {type}
  bind:value
  class={cn(
    "border-input placeholder:text-muted-foreground focus-visible:ring-ring focus-visible:border-ring flex h-9 w-full rounded-md border bg-transparent px-3 py-1 text-sm shadow-xs transition-[border-color,box-shadow] duration-120 ease-out file:border-0 file:bg-transparent file:text-sm file:font-medium focus-visible:ring-1 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
  {...restProps}
/>
