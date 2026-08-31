<script lang="ts">
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui";

  let { children, ...restProps }: AlertDialogPrimitive.PortalProps = $props();
</script>

<AlertDialogPrimitive.Portal {...restProps}>
  {@render children?.()}
</AlertDialogPrimitive.Portal>
