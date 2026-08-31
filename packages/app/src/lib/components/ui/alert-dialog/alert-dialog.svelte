<script lang="ts">
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui";

  let {
    open = $bindable(false),
    children,
    ...restProps
  }: AlertDialogPrimitive.RootProps = $props();
</script>

<AlertDialogPrimitive.Root bind:open {...restProps}>
  {@render children?.()}
</AlertDialogPrimitive.Root>
