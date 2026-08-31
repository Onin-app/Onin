<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { X } from "phosphor-svelte";
  import { cn } from "$lib/utils";
  import { sheetVariants, type SheetSide } from "./index";
  import SheetOverlay from "./sheet-overlay.svelte";
  import SheetPortal from "./sheet-portal.svelte";

  let {
    ref = $bindable(null),
    class: className,
    side = "right",
    portalProps,
    children,
    ...restProps
  }: DialogPrimitive.ContentProps & {
    side?: SheetSide;
    portalProps?: DialogPrimitive.PortalProps;
  } = $props();
</script>

<SheetPortal {...portalProps}>
  <SheetOverlay />
  <DialogPrimitive.Content
    bind:ref
    class={cn(sheetVariants({ side }), className)}
    {...restProps}
  >
    {@render children?.()}
    <DialogPrimitive.Close
      class="ring-offset-background focus:ring-ring data-[state=open]:bg-secondary absolute top-4 right-4 rounded-sm opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-none disabled:pointer-events-none"
    >
      <X class="h-4 w-4" />
      <span class="sr-only">关闭</span>
    </DialogPrimitive.Close>
  </DialogPrimitive.Content>
</SheetPortal>
