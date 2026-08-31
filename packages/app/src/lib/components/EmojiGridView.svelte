<script lang="ts">
  /**
   * EmojiGridView Component
   *
   * 显示 emoji Grid 视图，按分类分组
   * 支持键盘导航和点击选择
   */
  import { onMount } from "svelte";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import type {
    EmojiGridData,
    EmojiItem,
  } from "$lib/composables/useExtensionManager.svelte";

  interface Props {
    data: EmojiGridData;
    onSelect: (emoji: EmojiItem) => void;
  }

  let { data, onSelect }: Props = $props();

  // 当前选中的 emoji 索引
  let selectedGroupIndex = $state(0);
  let selectedEmojiIndex = $state(0);

  // 动态计算的列数（根据容器宽度）
  let gridColumns = $state(8);
  let gridContainerRefs: (HTMLDivElement | null)[] = [];

  // 计算列数的函数
  function updateGridColumns() {
    const gridContainerRef = gridContainerRefs[0];
    if (gridContainerRef) {
      const containerWidth = gridContainerRef.clientWidth;
      const itemWidth = 44; // emoji button width (increased)
      const gap = 8; // gap-2 = 8px (increased)
      // 计算能容纳多少列
      const cols = Math.floor((containerWidth + gap) / (itemWidth + gap));
      gridColumns = Math.max(1, cols);
    }
  }

  // 监听容器大小变化
  onMount(() => {
    // 延迟一下确保 DOM 已经渲染
    setTimeout(() => {
      updateGridColumns();
    }, 0);

    const gridContainerRef = gridContainerRefs[0];
    if (gridContainerRef) {
      const resizeObserver = new ResizeObserver(() => {
        updateGridColumns();
      });
      resizeObserver.observe(gridContainerRef);

      return () => {
        resizeObserver.disconnect();
      };
    }
  });

  // 计算所有 emoji 的扁平列表（用于键盘导航）
  const flatEmojis = $derived.by(() => {
    const result: {
      emoji: EmojiItem;
      groupIndex: number;
      emojiIndex: number;
    }[] = [];
    data.groups.forEach((group, groupIndex) => {
      group.emojis.forEach((emoji, emojiIndex) => {
        result.push({ emoji, groupIndex, emojiIndex });
      });
    });
    return result;
  });

  // 当前选中的扁平索引
  let flatSelectedIndex = $state(0);

  // 用于存储 emoji button 的引用
  let buttonRefs: (HTMLButtonElement | null)[] = [];

  // 当选中索引变化时，滚动到可视区域
  $effect(() => {
    const button = buttonRefs[flatSelectedIndex];
    if (button) {
      button.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
        inline: "nearest",
      });
    }
  });

  // 获取当前位置信息（组索引、组内索引、行、列）
  function getPositionInfo(flatIndex: number) {
    const item = flatEmojis[flatIndex];
    if (!item) return null;

    const { groupIndex, emojiIndex } = item;
    const row = Math.floor(emojiIndex / gridColumns);
    const col = emojiIndex % gridColumns;

    return { groupIndex, emojiIndex, row, col };
  }

  // 获取某个组在某行某列的扁平索引
  function getFlatIndexForGroupRowCol(
    groupIndex: number,
    row: number,
    col: number,
  ): number | null {
    const emojiIndex = row * gridColumns + col;

    // 检查该位置是否存在
    if (groupIndex < 0 || groupIndex >= data.groups.length) return null;
    const group = data.groups[groupIndex];
    if (emojiIndex < 0 || emojiIndex >= group.emojis.length) return null;

    // 计算扁平索引
    let flatIdx = 0;
    for (let i = 0; i < groupIndex; i++) {
      flatIdx += data.groups[i].emojis.length;
    }
    flatIdx += emojiIndex;

    return flatIdx;
  }

  // 处理键盘导航 - exported for parent to call
  export function handleKeyDown(e: KeyboardEvent) {
    const pos = getPositionInfo(flatSelectedIndex);
    if (!pos) return;

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
        {
          // 先尝试在同组内向下一行
          let nextIndex = getFlatIndexForGroupRowCol(
            pos.groupIndex,
            pos.row + 1,
            pos.col,
          );
          if (nextIndex === null) {
            // 没有下一行，尝试找同一行更靠左的位置
            for (let c = pos.col - 1; c >= 0; c--) {
              nextIndex = getFlatIndexForGroupRowCol(
                pos.groupIndex,
                pos.row + 1,
                c,
              );
              if (nextIndex !== null) break;
            }
          }
          if (nextIndex === null) {
            // 同组内没有下一行，跳到下一组的第一行同列位置
            for (let g = pos.groupIndex + 1; g < data.groups.length; g++) {
              nextIndex = getFlatIndexForGroupRowCol(g, 0, pos.col);
              if (nextIndex !== null) break;
              // 如果该列不存在，尝试最后一个
              const lastColInGroup = Math.min(
                pos.col,
                data.groups[g].emojis.length - 1,
              );
              nextIndex = getFlatIndexForGroupRowCol(g, 0, lastColInGroup);
              if (nextIndex !== null) break;
            }
          }
          if (nextIndex !== null) {
            flatSelectedIndex = nextIndex;
          }
        }
        break;
      case "ArrowUp":
        e.preventDefault();
        {
          // 先尝试在同组内向上一行
          let prevIndex = getFlatIndexForGroupRowCol(
            pos.groupIndex,
            pos.row - 1,
            pos.col,
          );
          if (prevIndex === null && pos.row > 0) {
            // 上一行存在但该列不存在，找靠左的
            for (let c = pos.col - 1; c >= 0; c--) {
              prevIndex = getFlatIndexForGroupRowCol(
                pos.groupIndex,
                pos.row - 1,
                c,
              );
              if (prevIndex !== null) break;
            }
          }
          if (prevIndex === null) {
            // 同组内没有上一行，跳到上一组的最后一行同列位置
            for (let g = pos.groupIndex - 1; g >= 0; g--) {
              const group = data.groups[g];
              const lastRow = Math.floor(
                (group.emojis.length - 1) / gridColumns,
              );
              prevIndex = getFlatIndexForGroupRowCol(g, lastRow, pos.col);
              if (prevIndex !== null) break;
              // 如果该列不存在，取最后一行最后一个
              prevIndex = getFlatIndexForGroupRowCol(
                g,
                lastRow,
                (group.emojis.length - 1) % gridColumns,
              );
              if (prevIndex !== null) break;
            }
          }
          if (prevIndex !== null) {
            flatSelectedIndex = prevIndex;
          }
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

<div class="h-full w-full" role="grid" tabindex="0" onkeydown={handleKeyDown}>
  <ScrollArea
    class="border-border/40 bg-card/40 h-full w-full rounded-2xl border shadow-2xs"
    viewportClass="h-full w-full overflow-x-hidden"
  >
    <div class="space-y-6 p-4">
      {#each data.groups as group, groupIndex}
        <div>
          <!-- 分类标题 -->
          <div class="mb-3 flex items-center gap-2">
            <span class="text-foreground text-xs font-semibold tracking-tight">
              {group.name}
            </span>
            <span
              class="bg-muted text-muted-foreground rounded-full px-2 py-0.5 font-mono text-[10px]"
            >
              {group.emojis.length}
            </span>
          </div>

          <!-- Emoji Grid -->
          <div
            bind:this={gridContainerRefs[groupIndex]}
            class="grid gap-2"
            style="grid-template-columns: repeat(auto-fill, minmax(44px, 1fr));"
          >
            {#each group.emojis as emoji, emojiIndex}
              {@const flatIndex = getFlatIndex(groupIndex, emojiIndex)}
              <button
                bind:this={buttonRefs[flatIndex]}
                class="flex h-11 w-11 cursor-pointer items-center justify-center rounded-xl text-2xl transition-[transform,background-color,box-shadow] duration-120 ease-[cubic-bezier(0.23,1,0.32,1)] active:scale-90
                    {flatSelectedIndex === flatIndex
                  ? 'bg-primary/15 ring-primary/60 scale-110 shadow-xs ring-2'
                  : 'hover:bg-muted/80 hover:scale-105'}"
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
  </ScrollArea>
</div>
