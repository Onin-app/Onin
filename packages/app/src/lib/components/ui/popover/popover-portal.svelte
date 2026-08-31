<script lang="ts">
  import { Popover as PopoverPrimitive } from "bits-ui";

  let { children, ...restProps }: PopoverPrimitive.PortalProps = $props();
</script>

<PopoverPrimitive.Portal {...restProps}>
  {@render children?.()}
</PopoverPrimitive.Portal>
