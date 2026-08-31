<script lang="ts">
  import { Switch as SwitchPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";

  let {
    ref = $bindable(null),
    checked = $bindable(false),
    class: className,
    disabled = false,
    ...restProps
  }: SwitchPrimitive.RootProps = $props();
</script>

<SwitchPrimitive.Root
  bind:ref
  bind:checked
  {disabled}
  class={cn(
    "group peer focus-visible:ring-ring focus-visible:ring-offset-background data-[state=checked]:bg-primary data-[state=unchecked]:bg-input inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent shadow-xs transition-[transform,background-color] duration-140 select-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.97] disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
  {...restProps}
>
  <SwitchPrimitive.Thumb
    class="bg-background pointer-events-none block h-4 w-4 rounded-full shadow-lg ring-0 transition-[transform,width] duration-200 ease-[cubic-bezier(0.32,0.72,0,1)] group-active:w-4.5 data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0"
  />
</SwitchPrimitive.Root>
