<script lang="ts">
  import { Popover as PopoverPrimitive } from "bits-ui";

  let {
    ref = $bindable(null),
    children,
    ...restProps
  }: PopoverPrimitive.TriggerProps = $props();
</script>

<PopoverPrimitive.Trigger bind:ref {...restProps}>
  {@render children?.()}
</PopoverPrimitive.Trigger>
