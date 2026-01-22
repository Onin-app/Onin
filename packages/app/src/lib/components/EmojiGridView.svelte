<script lang="ts">
  /**
   * EmojiGridView Component
   *
   * 显示 emoji Grid 视图，按分类分组
   * 支持键盘导航和点击选择
   */
  import { ScrollArea } from "bits-ui";
  import type { EmojiGridData, EmojiItem } from "$lib/composables/useExtensionManager.svelte";

  interface Props {
    data: EmojiGridData;
    onSelect: (emoji: EmojiItem) => void;
  }

  let { data, onSelect }: Props = $props();

  // 当前选中的 emoji 索引
  let selectedGroupIndex = $state(0);
  let selectedEmojiIndex = $state(0);

  // 计算所有 emoji 的扁平列表（用于键盘导航）
  const flatEmojis = $derived.by(() => {
    const result: { emoji: EmojiItem; groupIndex: number; emojiIndex: number }[] = [];
    data.groups.forEach((group, groupIndex) => {
      group.emojis.forEach((emoji, emojiIndex) => {
        result.push({ emoji, groupIndex, emojiIndex });
      });
    });
    return result;
  });

  // 当前选中的扁平索引
  let flatSelectedIndex = $state(0);

  // 处理键盘导航
  function handleKeyDown(e: KeyboardEvent) {
    const gridColumns = 8; // 每行显示的 emoji 数量

    switch (e.key) {
      case "ArrowRight":
        e.preventDefault();
        if (flatSelectedIndex < flatEmojis.length - 1) {
          flatSelectedIndex++;
        }
        break;
      case "ArrowLeft":
        e.preventDefault();
        if (flatSelectedIndex > 0) {
          flatSelectedIndex--;
        }
        break;
      case "ArrowDown":
        e.preventDefault();
        if (flatSelectedIndex + gridColumns < flatEmojis.length) {
          flatSelectedIndex += gridColumns;
        }
        break;
      case "ArrowUp":
        e.preventDefault();
        if (flatSelectedIndex - gridColumns >= 0) {
          flatSelectedIndex -= gridColumns;
        }
        break;
      case "Enter":
        e.preventDefault();
        if (flatEmojis[flatSelectedIndex]) {
          onSelect(flatEmojis[flatSelectedIndex].emoji);
        }
        break;
    }
  }

  // 点击选择 emoji
  function handleClick(emoji: EmojiItem, index: number) {
    flatSelectedIndex = index;
    onSelect(emoji);
  }

  // 获取 emoji 的扁平索引
  function getFlatIndex(groupIndex: number, emojiIndex: number): number {
    let index = 0;
    for (let i = 0; i < groupIndex; i++) {
      index += data.groups[i].emojis.length;
    }
    return index + emojiIndex;
  }
</script>

<div
  class="h-full w-full"
  role="grid"
  tabindex="0"
  onkeydown={handleKeyDown}
>
  <ScrollArea.Root class="h-full w-full rounded-[10px] border" type="hover">
    <ScrollArea.Viewport class="h-full w-full overflow-x-hidden">
      <div class="space-y-5 p-3">
        {#each data.groups as group, groupIndex}
          <div>
            <!-- 分类标题 -->
            <div class="mb-3 flex items-center gap-2">
              <span class="text-sm font-medium text-neutral-600 dark:text-neutral-400">
                {group.name}
              </span>
              <span class="rounded-full bg-neutral-200 px-2 py-0.5 text-xs text-neutral-500 dark:bg-neutral-700 dark:text-neutral-400">
                {group.emojis.length}
              </span>
            </div>

            <!-- Emoji Grid -->
            <div class="grid grid-cols-8 gap-1">
              {#each group.emojis as emoji, emojiIndex}
                {@const flatIndex = getFlatIndex(groupIndex, emojiIndex)}
                <button
                  class="flex h-10 w-10 cursor-pointer items-center justify-center rounded-lg text-2xl transition-all duration-150
                    {flatSelectedIndex === flatIndex
                      ? 'bg-blue-500/20 ring-2 ring-blue-500 scale-110'
                      : 'hover:bg-neutral-200 hover:scale-105 dark:hover:bg-neutral-700'}"
                  onclick={() => handleClick(emoji, flatIndex)}
                  title={emoji.name}
                >
                  {emoji.emoji}
                </button>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </ScrollArea.Viewport>
    <ScrollArea.Scrollbar
      orientation="vertical"
      class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-2 touch-none select-none rounded-full border-l border-l-transparent p-px transition-all duration-200 hover:w-3"
    >
      <ScrollArea.Thumb class="bg-neutral-300 dark:bg-neutral-600 flex-1 rounded-full" />
    </ScrollArea.Scrollbar>
    <ScrollArea.Corner />
  </ScrollArea.Root>
</div>
