<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { X } from "phosphor-svelte";
  import { cn } from "$lib/utils";
  import DialogOverlay from "./dialog-overlay.svelte";
  import DialogPortal from "./dialog-portal.svelte";

  let {
    ref = $bindable(null),
    class: className,
    children,
    portalProps,
    ...restProps
  }: DialogPrimitive.ContentProps & {
    portalProps?: DialogPrimitive.PortalProps;
  } = $props();
</script>

<DialogPortal {...portalProps}>
  <DialogOverlay />
  <DialogPrimitive.Content
    bind:ref
    class={cn(
      "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 bg-background fixed top-[50%] left-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 rounded-xl border p-6 shadow-lg duration-180 ease-[cubic-bezier(0.23,1,0.32,1)]",
      className,
    )}
    {...restProps}
  >
    {@render children?.()}
    <DialogPrimitive.Close
      class="ring-offset-background focus:ring-ring data-[state=open]:bg-accent data-[state=open]:text-muted-foreground absolute top-4 right-4 rounded-sm opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-none disabled:pointer-events-none"
    >
      <X class="h-4 w-4" />
      <span class="sr-only">关闭</span>
    </DialogPrimitive.Close>
  </DialogPrimitive.Content>
</DialogPortal>
