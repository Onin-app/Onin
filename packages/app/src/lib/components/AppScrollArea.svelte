<script lang="ts">
  import { ScrollArea, type WithoutChild } from "bits-ui";

  type Props = WithoutChild<ScrollArea.RootProps> & {
    orientation?: "vertical" | "horizontal" | "both";
    viewportClass?: string;
  };

  let {
    ref = $bindable(null),
    orientation = "vertical",
    viewportClass = "",
    class: rootClass = "",
    type = "hover",
    children,
    ...restProps
  }: Props = $props();

  const verticalScrollbarClass =
    "bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none rounded-full border-l border-l-transparent p-px transition-all duration-200 select-none hover:w-3";

  const horizontalScrollbarClass =
    "bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex h-1.5 touch-none rounded-full border-t border-t-transparent p-px transition-all duration-200 select-none hover:h-3";

  const thumbClass = "bg-muted-foreground flex-1 rounded-full";
</script>

{#snippet Scrollbar(orientation: "vertical" | "horizontal")}
  <ScrollArea.Scrollbar
    {orientation}
    class={orientation === "vertical"
      ? verticalScrollbarClass
      : horizontalScrollbarClass}
  >
    <ScrollArea.Thumb class={thumbClass} />
  </ScrollArea.Scrollbar>
{/snippet}

<ScrollArea.Root
  bind:ref
  class={rootClass}
  {type}
  {...restProps}
>
  <ScrollArea.Viewport class={viewportClass}>
    {@render children?.()}
  </ScrollArea.Viewport>

  {#if orientation === "vertical" || orientation === "both"}
    {@render Scrollbar("vertical")}
  {/if}

  {#if orientation === "horizontal" || orientation === "both"}
    {@render Scrollbar("horizontal")}
  {/if}

  <ScrollArea.Corner />
</ScrollArea.Root>
