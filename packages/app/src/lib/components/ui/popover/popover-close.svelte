<script lang="ts">
  import { Popover as PopoverPrimitive } from "bits-ui";

  let {
    ref = $bindable(null),
    children,
    ...restProps
  }: PopoverPrimitive.CloseProps = $props();
</script>

<PopoverPrimitive.Close bind:ref {...restProps}>
  {@render children?.()}
</PopoverPrimitive.Close>
