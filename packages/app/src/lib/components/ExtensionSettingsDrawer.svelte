<script lang="ts">
  import { X } from "phosphor-svelte";
  import { fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import FileSearchSettings from "$lib/components/settings/FileSearchSettings.svelte";
  import OcrSettings from "$lib/components/settings/OcrSettings.svelte";

  interface Props {
    open: boolean;
    extensionId: string;
    extensionName: string;
  }

  let { open = $bindable(false), extensionId, extensionName }: Props = $props();

  function closeDrawer() {
    open = false;
  }

  // 用 clip-path 从右向左揭开：右边缘始终固定，右侧圆角全程可见
  // intro: (1-t) 从 1→0，左侧裁剪从 100%→0%，内容从右侧揭开
  // outro: (1-t) 从 0→1，左侧裁剪从 0%→100%，内容向右收起
  function slidePanel(
    _node: Element,
    {
      duration = 250,
      easing = cubicOut,
    }: { duration?: number; easing?: (t: number) => number } = {},
  ) {
    return {
      duration,
      easing,
      css: (t: number) => `clip-path: inset(0 0 0 ${(1 - t) * 100}%)`,
    };
  }
</script>

{#if open}
  <!-- 遮罩层：fixed 覆盖全视口，确保左侧内容被遮住，消除白边 -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    transition:fade={{ duration: 150 }}
    class="fixed inset-0 z-50 rounded-xl bg-neutral-950/40 backdrop-blur-[1px]"
    onclick={closeDrawer}
  ></div>

  <!-- 内容面板：absolute 定位受 OCR 根容器 overflow-hidden + rounded-xl 裁剪，右侧圆角自动正确 -->
  <div
    transition:slidePanel={{ duration: 250, easing: cubicOut }}
    class="fixed top-0 right-0 bottom-0 z-50 flex w-full max-w-[440px] flex-col overflow-hidden rounded-l-2xl rounded-tr-xl rounded-br-xl bg-white shadow-2xl dark:bg-neutral-900"
  >
    <!-- 头部栏 -->
    <div
      class="flex items-center justify-between rounded-tl-2xl border-b border-neutral-200 bg-white px-4 py-3.5 dark:border-neutral-800 dark:bg-neutral-950"
    >
      <h3 class="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
        {extensionName || "扩展"}设置
      </h3>
      <button
        class="rounded-lg p-1.5 text-neutral-500 transition-colors hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800"
        onclick={closeDrawer}
        aria-label="关闭"
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <!-- 内容区滚动区域 -->
    <div class="flex-1 overflow-hidden rounded-bl-2xl">
      <AppScrollArea class="h-full w-full" viewportClass="h-full w-full">
        <div class="p-5 pr-6 pb-12">
          {#if extensionId === "file_search"}
            <FileSearchSettings />
          {:else if extensionId === "ocr"}
            <OcrSettings />
          {/if}
        </div>
      </AppScrollArea>
    </div>
  </div>
{/if}
