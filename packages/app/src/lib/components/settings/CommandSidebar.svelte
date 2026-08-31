<script lang="ts">
  /**
   * CommandSidebar Component
   *
   * 指令设置侧边栏组件
   * 显示内置指令分类和插件指令列表
   */
  import { Button } from "$lib/components/ui/button";
  import {
    Command as CommandIcon,
    RocketLaunch,
    File,
    Plugs,
    PuzzlePiece,
    Sparkle,
    Compass,
  } from "phosphor-svelte";

  // 分类图标映射
  const iconMap: Record<string, any> = {
    Command: CommandIcon,
    Extension: Sparkle,
    Application: RocketLaunch,
    FileCommand: File,
    Plugin: Plugs,
    Internal: Compass,
  };

  interface Category {
    id: string;
    name: string;
  }

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

<div class="border-border/40 flex w-36 shrink-0 flex-col gap-4 border-r pr-4">
  <!-- 内置指令分类 -->
  <div class="flex flex-col gap-1">
    <h3
      class="text-muted-foreground px-2 py-1 text-[10px] font-semibold tracking-wider uppercase select-none"
    >
      内置指令
    </h3>
    <div class="flex flex-col gap-0.5">
      {#each categories as category}
        {@const isActive = activeCategory?.id === category.id}
        <Button
          variant={isActive ? "secondary" : "ghost"}
          size="sm"
          class="h-8.5 w-full cursor-pointer justify-start gap-2 rounded-xl px-2.5 text-xs font-medium transition-[transform,background-color,color,box-shadow] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-[0.97] {isActive
            ? 'bg-card text-foreground border-border/50 border shadow-2xs'
            : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
          onclick={() => onSelectCategory(category.id)}
        >
          {@const Icon = iconMap[category.id] || CommandIcon}
          <Icon size={14} class="shrink-0" />
          <span class="truncate">{category.name}</span>
        </Button>
      {/each}
    </div>
  </div>

  <!-- 插件指令分类 -->
  {#if pluginNames.length > 0}
    <div class="flex flex-col gap-1">
      <h3
        class="text-muted-foreground px-2 py-1 text-[10px] font-semibold tracking-wider uppercase select-none"
      >
        插件指令
      </h3>
      <div class="flex flex-col gap-0.5">
        {#each pluginNames as pluginName}
          {@const isActive = selectedPlugin === pluginName}
          <Button
            variant={isActive ? "secondary" : "ghost"}
            size="sm"
            class="h-8.5 w-full cursor-pointer justify-start gap-2 rounded-xl px-2.5 text-xs font-medium transition-[transform,background-color,color,box-shadow] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-[0.97] {isActive
              ? 'bg-card text-foreground border-border/50 border shadow-2xs'
              : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
            onclick={() => onSelectPlugin(pluginName)}
          >
            <PuzzlePiece size={14} class="shrink-0" />
            <span class="truncate">{pluginName}</span>
          </Button>
        {/each}
      </div>
    </div>
  {/if}
</div>
