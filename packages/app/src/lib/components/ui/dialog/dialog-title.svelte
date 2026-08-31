<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...restProps
  }: DialogPrimitive.TitleProps = $props();
</script>

<DialogPrimitive.Title
  bind:ref
  class={cn("text-lg leading-none font-semibold tracking-tight", className)}
  {...restProps}
>
  {@render children?.()}
</DialogPrimitive.Title>
