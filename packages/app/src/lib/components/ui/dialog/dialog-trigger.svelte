<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";

  let {
    ref = $bindable(null),
    children,
    ...restProps
  }: DialogPrimitive.TriggerProps = $props();
</script>

<DialogPrimitive.Trigger bind:ref {...restProps}>
  {@render children?.()}
</DialogPrimitive.Trigger>
