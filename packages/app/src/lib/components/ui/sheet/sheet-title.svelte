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
  class={cn("text-foreground text-base font-semibold", className)}
  {...restProps}
>
  {@render children?.()}
</DialogPrimitive.Title>
