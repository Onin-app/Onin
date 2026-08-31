<script lang="ts">
  import { Select as SelectPrimitive } from "bits-ui";

  let {
    open = $bindable(false),
    value = $bindable(),
    type = "single",
    children,
    ...restProps
  }: any = $props();
</script>

{#if type === "multiple"}
  <SelectPrimitive.Root bind:open bind:value type="multiple" {...restProps}>
    {@render children?.()}
  </SelectPrimitive.Root>
{:else}
  <SelectPrimitive.Root bind:open bind:value type="single" {...restProps}>
    {@render children?.()}
  </SelectPrimitive.Root>
{/if}
