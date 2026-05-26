<script lang="ts">
  import { Eye, EyeSlash } from "phosphor-svelte";
  import type { HTMLInputAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLInputAttributes, "value"> {
    value: string | null;
  }

  let {
    value = $bindable(),
    class: className = "",
    placeholder = "",
    ...restProps
  }: Props = $props();

  let showPassword = $state(false);
</script>

<div class="relative flex items-center">
  <input
    type={showPassword ? "text" : "password"}
    bind:value
    {placeholder}
    {...restProps}
    class="border border-neutral-200 bg-white text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100 {className} pr-10 pl-3"
  />
  <button
    type="button"
    onclick={() => (showPassword = !showPassword)}
    class="absolute right-3 flex cursor-pointer items-center justify-center text-neutral-400 hover:text-neutral-600 focus:outline-hidden dark:hover:text-neutral-200"
    aria-label={showPassword ? "隐藏密码" : "显示密码"}
  >
    {#if showPassword}
      <EyeSlash class="h-4 w-4" />
    {:else}
      <Eye class="h-4 w-4" />
    {/if}
  </button>
</div>
