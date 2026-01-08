<script lang="ts">
  /**
   * CommandSidebar Component
   *
   * 指令设置侧边栏组件
   * 显示内置指令分类和插件指令列表
   */
  import { Button } from "bits-ui";
  import {
    Command as CommandIcon,
    RocketLaunch,
    File,
    Plugs,
    User,
    PuzzlePiece,
  } from "phosphor-svelte";

  // 分类图标映射
  const iconMap: Record<string, any> = {
    Command: CommandIcon,
    Application: RocketLaunch,
    FileCommand: File,
    Plugin: Plugs,
    Custom: User,
  };

  // 分类接口
  interface Category {
    id: string;
    name: string;
  }

  // Props 接口
  interface Props {
    categories: Category[];
    activeCategory: Category | null;
    pluginNames: string[];
    selectedPlugin: string | null;
    onSelectCategory: (categoryId: string) => void;
    onSelectPlugin: (pluginName: string) => void;
  }

  let {
    categories,
    activeCategory,
    pluginNames,
    selectedPlugin,
    onSelectCategory,
    onSelectPlugin,
  }: Props = $props();
</script>

<div
  class="flex w-36 shrink-0 flex-col gap-4 border-r border-neutral-100 pr-4 dark:border-neutral-800"
>
  <!-- 内置指令分类 -->
  <div class="flex flex-col gap-1">
    <h3
      class="px-2 py-1 text-[10px] font-semibold tracking-wider text-neutral-400 uppercase dark:text-neutral-500"
    >
      内置指令
    </h3>
    <div class="flex flex-col gap-0.5">
      {#each categories as category}
        <Button.Root
          class="flex w-full items-center justify-start gap-2 rounded-md px-2 py-1 text-sm font-medium transition-colors {activeCategory?.id ===
          category.id
            ? 'bg-neutral-100 text-neutral-900 dark:bg-neutral-800 dark:text-white'
            : 'text-neutral-500 hover:bg-neutral-50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-neutral-200'}"
          onclick={() => onSelectCategory(category.id)}
        >
          {@const Icon = iconMap[category.id] || CommandIcon}
          <Icon size={15} />
          {category.name}
        </Button.Root>
      {/each}
    </div>
  </div>

  <!-- 插件指令分类 -->
  {#if pluginNames.length > 0}
    <div class="flex flex-col gap-1">
      <h3
        class="px-2 py-1 text-[10px] font-semibold tracking-wider text-neutral-400 uppercase dark:text-neutral-500"
      >
        插件指令
      </h3>
      <div class="flex flex-col gap-0.5">
        {#each pluginNames as pluginName}
          <Button.Root
            class="flex w-full items-center justify-start gap-2 truncate rounded-md px-2 py-1 text-left text-sm font-medium transition-colors {selectedPlugin ===
            pluginName
              ? 'bg-neutral-100 text-neutral-900 dark:bg-neutral-800 dark:text-white'
              : 'text-neutral-500 hover:bg-neutral-50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-neutral-200'}"
            onclick={() => onSelectPlugin(pluginName)}
          >
            <PuzzlePiece size={15} class="shrink-0" />
            <span class="truncate">{pluginName}</span>
          </Button.Root>
        {/each}
      </div>
    </div>
  {/if}
</div>
