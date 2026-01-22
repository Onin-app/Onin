<script lang="ts">
  /**
   * ExtensionHeader Component
   *
   * Extension 页面通用 header 组件
   * 包含返回按钮、搜索输入框和可选的右侧插槽
   */
  import { ArrowLeft } from "phosphor-svelte";
  import type { Snippet } from "svelte";

  interface Props {
    icon?: string;
    title?: string;
    placeholder?: string;
    value?: string;
    onInput?: (value: string) => void;
    onBack?: () => void;
    right?: Snippet;
  }

  let {
    icon,
    title,
    placeholder = "搜索...",
    value = $bindable(""),
    onInput,
    onBack,
    right,
  }: Props = $props();

  let inputElement: HTMLInputElement;

  export function focus() {
    inputElement?.focus();
  }

  const handleInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    value = target.value;
    onInput?.(target.value);
  };
</script>

<div
  class="flex items-center gap-2 pb-2"
  role="banner"
>
  <!-- Back Button -->
  <button
    class="flex h-10 w-10 flex-shrink-0 cursor-pointer items-center justify-center rounded-lg text-neutral-600 transition-colors hover:bg-neutral-200 dark:text-neutral-400 dark:hover:bg-neutral-700"
    onclick={onBack}
    aria-label="返回"
  >
    <ArrowLeft class="size-5" weight="bold" />
  </button>

  <!-- Search Input -->
  <div
    class="flex w-full flex-row items-center gap-2 rounded-lg border border-neutral-300 bg-white px-2 py-2 dark:border-neutral-600 dark:bg-neutral-800"
  >
    <input
      bind:this={inputElement}
      class="min-w-0 flex-1 h-[34px] bg-transparent text-2xl focus:outline-none focus:ring-0 active:outline-none active:ring-0"
      type="text"
      {placeholder}
      {value}
      oninput={handleInput}
    />
  </div>

  <!-- Right Slot -->
  <div class="flex-shrink-0">
    {#if right}
      {@render right()}
    {/if}
  </div>
</div>
