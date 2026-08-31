<script lang="ts">
  import { Tooltip as TooltipPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";

  let {
    ref = $bindable(null),
    class: className,
    sideOffset = 4,
    children,
    ...restProps
  }: TooltipPrimitive.ContentProps = $props();
</script>

<TooltipPrimitive.Portal>
  <TooltipPrimitive.Content
    bind:ref
    {sideOffset}
    class={cn(
      "bg-primary text-primary-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:data-[state=open]:slide-in-from-top-1.5 data-[side=bottom]:data-[state=closed]:slide-out-to-top-1.5 data-[side=top]:data-[state=open]:slide-in-from-bottom-1.5 data-[side=top]:data-[state=closed]:slide-out-to-bottom-1.5 data-[side=left]:data-[state=open]:slide-in-from-right-1.5 data-[side=left]:data-[state=closed]:slide-out-to-right-1.5 data-[side=right]:data-[state=open]:slide-in-from-left-1.5 data-[side=right]:data-[state=closed]:slide-out-to-left-1.5 pointer-events-none z-50 origin-[var(--bits-floating-transform-origin,center)] overflow-hidden rounded-md px-2.5 py-1 text-xs shadow-xs ease-[cubic-bezier(0.23,1,0.32,1)] select-none data-[state=closed]:duration-100 data-[state=open]:duration-140",
      className,
    )}
    {...restProps}
  >
    {@render children?.()}
  </TooltipPrimitive.Content>
</TooltipPrimitive.Portal>
