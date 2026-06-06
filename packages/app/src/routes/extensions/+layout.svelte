<script lang="ts">
  /**
   * Extensions Layout
   *
   * Extension 页面共享布局
   * 处理主题和 ESC 返回逻辑
   */
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { Theme } from "$lib/type";
  import { theme, getTheme } from "$lib/utils/theme";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { page } from "$app/state";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();

  let currentTheme = $state<Theme>(Theme.DARK);

  // 返回主窗口
  const handleBack = () => {
    goto("/");
  };

  // ESC 处理（增加防重入锁，防止物理按键 DOM 事件与后端事件极短时间内双重触发）
  let isEscaping = false;
  const handleEsc = () => {
    if (isEscaping) return;
    isEscaping = true;
    handleBack();
    setTimeout(() => {
      isEscaping = false;
    }, 150);
  };

  // Theme subscription
  const unsubscribeTheme = theme.subscribe((value) => {
    currentTheme = value;
  });

  let removeWindowEscapeListener: (() => void) | null = null;

  onMount(() => {
    escapeHandler.set(handleEsc);

    const handleWindowEscape = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || event.defaultPrevented) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      handleEsc();
    };

    window.addEventListener("keydown", handleWindowEscape, true);
    removeWindowEscapeListener = () => {
      window.removeEventListener("keydown", handleWindowEscape, true);
    };
  });

  onDestroy(() => {
    unsubscribeTheme?.();
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(null);
    }
    removeWindowEscapeListener?.();
  });

  const isTranslator = $derived(page.route.id?.includes("translator"));
</script>

{#if isTranslator}
  {@render children()}
{:else}
  <div class="h-[100vh] w-full bg-transparent p-1">
    <main
      class="h-full w-full overflow-hidden rounded-xl bg-neutral-100 p-3 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
      data-tauri-drag-region
    >
      {@render children()}
    </main>
  </div>
{/if}
