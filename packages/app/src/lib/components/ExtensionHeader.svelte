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
    focusExtensionInputTrigger,
    focusInputElement,
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
    focusInputElement(inputElement);
  }

  $effect(() => {
    $focusExtensionInputTrigger;
    focusInputElement(inputElement);
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
    class="text-muted-foreground hover:text-foreground hover:bg-muted/80 flex h-9 w-9 flex-shrink-0 cursor-pointer items-center justify-center rounded-xl transition-[transform,background-color] duration-120 active:scale-95"
    onclick={onBack}
    aria-label="返回"
  >
    <ArrowLeft class="size-4.5" weight="bold" />
  </button>

  <!-- Search Input or Title -->
  {#if showSearch}
    <div
      class="border-border/60 bg-background/80 focus-within:border-primary/60 focus-within:ring-primary/20 flex w-full flex-row items-center gap-2 rounded-xl border px-3 py-1.5 shadow-2xs transition-[border-color,box-shadow] duration-140 focus-within:ring-2 {disabled
        ? 'bg-muted/40 cursor-not-allowed opacity-65'
        : ''}"
    >
      <input
        bind:this={inputElement}
        id="extension-search-input"
        class="text-foreground placeholder:text-muted-foreground/50 h-[30px] min-w-0 flex-1 bg-transparent text-xl font-medium tracking-tight focus:outline-none {disabled
          ? 'text-muted-foreground cursor-not-allowed'
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
        class="text-foreground text-sm font-semibold tracking-tight uppercase"
      >
        {title || "扩展"}
      </h2>
    </div>
  {/if}

  <!-- Right Slot & Settings Button -->
  <div class="flex flex-shrink-0 items-center gap-1.5">
    {#if right}
      {@render right()}
    {/if}
    {#if extensionId}
      {#if right}
        <span class="border-border/40 h-4 w-[1px] border-r"></span>
      {/if}
      <button
        class="text-muted-foreground hover:text-foreground hover:bg-muted/80 flex h-9 w-9 cursor-pointer items-center justify-center rounded-xl transition-[transform,background-color] duration-120 active:scale-95"
        onclick={() => (settingsOpen = true)}
        aria-label="扩展设置"
      >
        <Gear class="size-4.5" />
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
