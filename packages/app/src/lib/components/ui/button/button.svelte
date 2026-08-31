<script lang="ts">
  import { Button as ButtonPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import { buttonVariants, type ButtonVariant, type ButtonSize } from "./index";
  import type {
    HTMLButtonAttributes,
    HTMLAnchorAttributes,
  } from "svelte/elements";
  import type { Snippet } from "svelte";

  type Props = (
    | (HTMLButtonAttributes & { href?: undefined })
    | (HTMLAnchorAttributes & { href: string })
  ) & {
    variant?: ButtonVariant;
    size?: ButtonSize;
    ref?: HTMLElement | null;
    children?: Snippet;
  };

  let {
    class: className,
    variant = "default",
    size = "default",
    ref = $bindable(null),
    children,
    type = "button",
    ...restProps
  }: Props = $props();
</script>

<ButtonPrimitive.Root
  bind:ref={ref as any}
  {type}
  class={cn(buttonVariants({ variant, size }), className)}
  {...restProps as any}
>
  {@render children?.()}
</ButtonPrimitive.Root>
