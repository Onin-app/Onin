<script lang="ts">
  import { Select as SelectPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import SelectPortal from "./select-portal.svelte";

  let {
    ref = $bindable(null),
    class: className,
    sideOffset = 4,
    portalProps,
    children,
    ...restProps
  }: SelectPrimitive.ContentProps & {
    portalProps?: SelectPrimitive.PortalProps;
  } = $props();
</script>

<SelectPortal {...portalProps}>
  <SelectPrimitive.Content
    bind:ref
    {sideOffset}
    class={cn(
      "bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:data-[state=open]:slide-in-from-top-2 data-[side=bottom]:data-[state=closed]:slide-out-to-top-2 data-[side=top]:data-[state=open]:slide-in-from-bottom-2 data-[side=top]:data-[state=closed]:slide-out-to-bottom-2 data-[side=left]:data-[state=open]:slide-in-from-right-2 data-[side=left]:data-[state=closed]:slide-out-to-right-2 data-[side=right]:data-[state=open]:slide-in-from-left-2 data-[side=right]:data-[state=closed]:slide-out-to-left-2 relative z-50 max-h-96 min-w-[8rem] origin-[var(--bits-floating-transform-origin,center)] overflow-hidden rounded-md border shadow-md duration-140 ease-[cubic-bezier(0.23,1,0.32,1)]",
      className,
    )}
    {...restProps}
  >
    <SelectPrimitive.Viewport class="p-1">
      {@render children?.()}
    </SelectPrimitive.Viewport>
  </SelectPrimitive.Content>
</SelectPortal>
