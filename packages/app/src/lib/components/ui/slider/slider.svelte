<script lang="ts">
  import { Slider as SliderPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";

  type Props = {
    ref?: HTMLElement | null;
    value?: number | number[];
    type?: "single" | "multiple";
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    class?: string;
    onValueChange?: (value: any) => void;
    onValueCommit?: (value: any) => void;
    [key: string]: any;
  };

  let {
    ref = $bindable(null),
    value = $bindable(),
    class: className,
    type = "single",
    ...restProps
  }: Props = $props();
</script>

{#if type === "single"}
  <SliderPrimitive.Root
    bind:ref
    bind:value={value as number}
    type="single"
    class={cn(
      "relative flex w-full touch-none items-center select-none",
      className,
    )}
    {...restProps}
  >
    {#snippet children({ thumbs })}
      <span
        class="bg-secondary relative h-1.5 w-full grow overflow-hidden rounded-full"
      >
        <SliderPrimitive.Range class="bg-primary absolute h-full" />
      </span>
      {#each thumbs as thumb}
        <SliderPrimitive.Thumb
          index={thumb}
          class="border-primary/50 bg-background focus-visible:ring-ring block h-4 w-4 rounded-full border shadow transition-colors focus-visible:ring-1 focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
        />
      {/each}
    {/snippet}
  </SliderPrimitive.Root>
{:else}
  <SliderPrimitive.Root
    bind:ref
    bind:value={value as number[]}
    type="multiple"
    class={cn(
      "relative flex w-full touch-none items-center select-none",
      className,
    )}
    {...restProps}
  >
    {#snippet children({ thumbs })}
      <span
        class="bg-secondary relative h-1.5 w-full grow overflow-hidden rounded-full"
      >
        <SliderPrimitive.Range class="bg-primary absolute h-full" />
      </span>
      {#each thumbs as thumb}
        <SliderPrimitive.Thumb
          index={thumb}
          class="border-primary/50 bg-background focus-visible:ring-ring block h-4 w-4 rounded-full border shadow transition-colors focus-visible:ring-1 focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
        />
      {/each}
    {/snippet}
  </SliderPrimitive.Root>
{/if}
