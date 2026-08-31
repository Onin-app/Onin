<script lang="ts">
  import { Tooltip as TooltipPrimitive } from "bits-ui";

  let {
    ref = $bindable(null),
    children,
    ...restProps
  }: TooltipPrimitive.TriggerProps = $props();
</script>

<TooltipPrimitive.Trigger bind:ref {...restProps}>
  {@render children?.()}
</TooltipPrimitive.Trigger>
