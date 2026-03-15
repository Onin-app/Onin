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

  // ESC 处理
  const handleEsc = () => {
    handleBack();
  };

  // Theme subscription
  const unsubscribeTheme = theme.subscribe((value) => {
    currentTheme = value;
  });

  onMount(() => {
    escapeHandler.set(handleEsc);
  });

  onDestroy(() => {
    unsubscribeTheme?.();
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(null);
    }
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
