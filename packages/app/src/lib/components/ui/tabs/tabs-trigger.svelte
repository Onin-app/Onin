<script lang="ts">
  import { Tabs as TabsPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...restProps
  }: TabsPrimitive.TriggerProps = $props();
</script>

<TabsPrimitive.Trigger
  bind:ref
  class={cn(
    "ring-offset-background focus-visible:ring-ring data-[state=active]:bg-background data-[state=active]:text-foreground inline-flex cursor-pointer items-center justify-center rounded-md px-3 py-1 text-xs font-medium whitespace-nowrap transition-[background-color,color,box-shadow,transform] duration-120 ease-out select-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.97] disabled:pointer-events-none disabled:opacity-50 data-[state=active]:shadow-xs",
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</TabsPrimitive.Trigger>
