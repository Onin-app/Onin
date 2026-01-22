<script lang="ts">
  /**
   * Emoji Extension Page
   *
   * 独立的 emoji 选择器页面
   * 使用 ExtensionHeader 组件和 EmojiGridView 组件
   */
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";
  import EmojiGridView from "$lib/components/EmojiGridView.svelte";
  import type { EmojiGridData, EmojiItem } from "$lib/composables/useExtensionManager.svelte";

  let searchQuery = $state("");
  let emojiData = $state<EmojiGridData | null>(null);
  let headerRef: ExtensionHeader;

  // 从 URL 参数获取初始搜索值
  const initialQuery = $derived($page.url.searchParams.get("q") || "");

  // 获取 emoji 数据
  async function fetchEmojiData(query: string) {
    try {
      const data = await invoke<EmojiGridData | null>("get_emoji_data", {
        searchQuery: query,
      });

      if (data) {
        emojiData = data;
      }
    } catch (error) {
      console.error("[Emoji] Failed to fetch data:", error);
    }
  }

  // 处理搜索输入
  const handleSearch = (value: string) => {
    searchQuery = value;
    fetchEmojiData(value);
  };

  // 返回主窗口
  const handleBack = () => {
    goto("/");
  };

  // 处理 emoji 选择
  const handleEmojiSelect = async (emoji: EmojiItem) => {
    try {
      await navigator.clipboard.writeText(emoji.emoji);
      console.log("[Emoji] Copied to clipboard:", emoji.emoji);
    } catch (e) {
      console.error("[Emoji] Failed to copy:", e);
    }

    // 关闭窗口
    invoke("close_main_window");
  };

  onMount(() => {
    // 使用 URL 参数初始化搜索
    searchQuery = initialQuery;
    fetchEmojiData(searchQuery);

    // 自动聚焦输入框
    headerRef?.focus();
  });
</script>

<div class="flex h-full w-full flex-col">
  <ExtensionHeader
    bind:this={headerRef}
    placeholder="Search Emoji & Symbols..."
    bind:value={searchQuery}
    onInput={handleSearch}
    onBack={handleBack}
  />

  <div class="flex-1 overflow-hidden">
    {#if emojiData}
      <EmojiGridView data={emojiData} onSelect={handleEmojiSelect} />
    {:else}
      <div class="flex h-full items-center justify-center text-neutral-500">
        加载中...
      </div>
    {/if}
  </div>
</div>
