<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";
  import { badgeVariants, type BadgeVariant } from "./index";

  interface Props extends HTMLAttributes<HTMLDivElement> {
    variant?: BadgeVariant;
  }

  let {
    class: className,
    variant = "default",
    children,
    ...restProps
  }: Props = $props();
</script>

<div class={cn(badgeVariants({ variant }), className)} {...restProps}>
  {@render children?.()}
</div>
