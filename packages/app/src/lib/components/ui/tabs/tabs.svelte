<script lang="ts">
  import { Tabs as TabsPrimitive } from "bits-ui";

  let {
    ref = $bindable(null),
    value = $bindable(),
    children,
    ...restProps
  }: TabsPrimitive.RootProps = $props();
</script>

<TabsPrimitive.Root bind:ref bind:value {...restProps}>
  {@render children?.()}
</TabsPrimitive.Root>
