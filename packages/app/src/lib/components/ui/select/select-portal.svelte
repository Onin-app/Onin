<script lang="ts">
  import { Select as SelectPrimitive } from "bits-ui";

  let { children, ...restProps }: SelectPrimitive.PortalProps = $props();
</script>

<SelectPrimitive.Portal {...restProps}>
  {@render children?.()}
</SelectPrimitive.Portal>
