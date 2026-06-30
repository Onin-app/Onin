<script lang="ts">
  /**
   * ExtensionHeader Component
   *
   * Extension 页面通用 header 组件
   * 包含返回按钮、搜索输入框和可选的右侧插槽
   */
  import { ArrowLeft, Gear } from "phosphor-svelte";
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";
  import {
    requestInputFocusWithRetry,
    focusExtensionInputTrigger,
  } from "$lib/stores/focusInput";
  import ExtensionSettingsDrawer from "./ExtensionSettingsDrawer.svelte";

  interface Props {
    icon?: string;
    title?: string;
    placeholder?: string;
    value?: string;
    showSearch?: boolean;
    disabled?: boolean;
    onInput?: (value: string) => void;
    onBack?: () => void;
    onKeyDown?: (e: KeyboardEvent) => void;
    right?: Snippet;
    extensionId?: string;
  }

  let {
    icon,
    title,
    placeholder = "搜索...",
    value = $bindable(""),
    showSearch = true,
    disabled = false,
    onInput,
    onBack,
    onKeyDown,
    right,
    extensionId,
  }: Props = $props();

  let settingsOpen = $state(false);

  let inputElement: HTMLInputElement = $state()!;

  export function focus() {
    requestInputFocusWithRetry();
  }

  let initialTrigger = $state<number | null>(null);

  $effect(() => {
    const triggerVal = $focusExtensionInputTrigger;
    if (initialTrigger === null) {
      initialTrigger = triggerVal;
      // 首次初始化挂载时（例如普通内部路由跳转）：只执行单次的原生 DOM 聚焦，不启动定时器重试，彻底杜绝与快捷键冲突
      if (typeof document !== "undefined") {
        inputElement?.focus();
      }
    } else if (triggerVal > initialTrigger) {
      // 只有当全局信号发生实际递增（快捷键唤起）时，才独占启动带有重试的聚焦引擎
      focus();
    }
  });

  const handleInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    value = target.value;
    onInput?.(target.value);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    // Forward navigation keys and strictly whitelisted custom shortcuts (Ctrl+C / Cmd+C) to parent
    if (
      [
        "ArrowUp",
        "ArrowDown",
        "ArrowLeft",
        "ArrowRight",
        "Enter",
        "Backspace",
      ].includes(e.key) ||
      // 仅限特定快捷键白名单：复制操作 Ctrl+C 或 Cmd+C
      (e.key.toLowerCase() === "c" && (e.ctrlKey || e.metaKey))
    ) {
      onKeyDown?.(e);
    }
  };
</script>

<div class="flex items-center gap-2 pb-2" role="banner">
  <!-- Back Button -->
  <button
    class="flex h-10 w-10 flex-shrink-0 cursor-pointer items-center justify-center rounded-lg text-neutral-600 transition-colors hover:bg-neutral-200 dark:text-neutral-400 dark:hover:bg-neutral-700"
    onclick={onBack}
    aria-label="返回"
  >
    <ArrowLeft class="size-5" weight="bold" />
  </button>

  <!-- Search Input or Title -->
  {#if showSearch}
    <div
      class="flex w-full flex-row items-center gap-2 rounded-lg border border-neutral-300 bg-white px-2 py-2 transition-all duration-200 dark:border-neutral-600 dark:bg-neutral-800 {disabled
        ? 'cursor-not-allowed bg-neutral-50 opacity-65 dark:bg-neutral-900/50'
        : ''}"
    >
      <input
        bind:this={inputElement}
        id="extension-search-input"
        class="h-[34px] min-w-0 flex-1 bg-transparent text-2xl focus:ring-0 focus:outline-none active:ring-0 active:outline-none {disabled
          ? 'cursor-not-allowed text-neutral-400 dark:text-neutral-500'
          : ''}"
        type="text"
        {disabled}
        {placeholder}
        {value}
        oninput={handleInput}
        onkeydown={handleKeyDown}
      />
    </div>
  {:else}
    <div class="flex-1 pl-1.5">
      <h2
        class="text-sm font-semibold tracking-wide text-neutral-800 uppercase dark:text-neutral-100"
      >
        {title || "扩展"}
      </h2>
    </div>
  {/if}

  <!-- Right Slot & Settings Button -->
  <div class="flex flex-shrink-0 items-center gap-2">
    {#if right}
      {@render right()}
    {/if}
    {#if extensionId}
      {#if right}
        <span class="h-4 w-[1px] bg-neutral-200 dark:bg-neutral-800"></span>
      {/if}
      <button
        class="flex h-9 w-9 cursor-pointer items-center justify-center rounded-lg text-neutral-600 transition-all hover:bg-neutral-200 active:scale-95 dark:text-neutral-400 dark:hover:bg-neutral-700"
        onclick={() => (settingsOpen = true)}
        aria-label="扩展设置"
      >
        <Gear class="size-5" />
      </button>
    {/if}
  </div>
</div>

{#if extensionId}
  <ExtensionSettingsDrawer
    bind:open={settingsOpen}
    {extensionId}
    extensionName={title || ""}
  />
{/if}
