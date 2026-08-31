<script lang="ts">
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import { buttonVariants } from "$lib/components/ui/button";

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...restProps
  }: AlertDialogPrimitive.CancelProps = $props();
</script>

<AlertDialogPrimitive.Cancel
  bind:ref
  class={cn(buttonVariants({ variant: "outline" }), "mt-2 sm:mt-0", className)}
  {...restProps}
>
  {@render children?.()}
</AlertDialogPrimitive.Cancel>
