<script lang="ts">
  import {
    ScrollArea as ScrollAreaPrimitive,
    type WithoutChild,
  } from "bits-ui";
  import { cn } from "$lib/utils";

  type Props = WithoutChild<ScrollAreaPrimitive.RootProps> & {
    orientation?: "vertical" | "horizontal" | "both";
    viewportClass?: string;
    verticalScrollbarClass?: string;
    horizontalScrollbarClass?: string;
    thumbClass?: string;
  };

  let {
    ref = $bindable(null),
    orientation = "vertical",
    viewportClass = "",
    verticalScrollbarClass: verticalScrollbarClassOverride = "",
    horizontalScrollbarClass: horizontalScrollbarClassOverride = "",
    thumbClass: thumbClassOverride = "",
    class: className,
    type = "hover",
    children,
    ...restProps
  }: Props = $props();

  const defaultVerticalScrollbarClass =
    "bg-muted/30 hover:bg-muted/80 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none rounded-full border-l border-l-transparent p-px transition-[width,background-color,opacity] duration-150 ease-out select-none hover:w-2.5 z-40";

  const defaultHorizontalScrollbarClass =
    "bg-muted/30 hover:bg-muted/80 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex h-1.5 touch-none rounded-full border-t border-t-transparent p-px transition-[height,background-color,opacity] duration-150 ease-out select-none hover:h-2.5 z-40";

  const defaultThumbClass =
    "bg-muted-foreground/50 hover:bg-muted-foreground/80 flex-1 rounded-full transition-colors duration-120";
</script>

{#snippet Scrollbar(orientation: "vertical" | "horizontal")}
  <ScrollAreaPrimitive.Scrollbar
    {orientation}
    class={cn(
      orientation === "vertical"
        ? verticalScrollbarClassOverride || defaultVerticalScrollbarClass
        : horizontalScrollbarClassOverride || defaultHorizontalScrollbarClass,
    )}
  >
    <ScrollAreaPrimitive.Thumb
      class={cn(thumbClassOverride || defaultThumbClass)}
    />
  </ScrollAreaPrimitive.Scrollbar>
{/snippet}

<ScrollAreaPrimitive.Root
  bind:ref
  class={cn("relative overflow-hidden", className)}
  {type}
  {...restProps}
>
  <ScrollAreaPrimitive.Viewport
    class={cn("h-full w-full rounded-[inherit]", viewportClass)}
  >
    {@render children?.()}
  </ScrollAreaPrimitive.Viewport>

  {#if orientation === "vertical" || orientation === "both"}
    {@render Scrollbar("vertical")}
  {/if}

  {#if orientation === "horizontal" || orientation === "both"}
    {@render Scrollbar("horizontal")}
  {/if}

  <ScrollAreaPrimitive.Corner />
</ScrollAreaPrimitive.Root>
