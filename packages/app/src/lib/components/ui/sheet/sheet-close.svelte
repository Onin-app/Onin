<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";

  let {
    ref = $bindable(null),
    children,
    ...restProps
  }: DialogPrimitive.CloseProps = $props();
</script>

<DialogPrimitive.Close bind:ref {...restProps}>
  {@render children?.()}
</DialogPrimitive.Close>
