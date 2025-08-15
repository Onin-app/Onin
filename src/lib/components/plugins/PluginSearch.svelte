<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Button } from 'bits-ui';

  interface Props {
    searchQuery: string;
    showAllPlugins: boolean;
  }

  let { searchQuery = $bindable(), showAllPlugins = $bindable() }: Props = $props();

  const dispatch = createEventDispatcher<{
    importPlugin: void;
  }>();
</script>

<div class="flex items-center gap-3">
  <!-- 搜索框 -->
  <div class="relative">
    <svg
      class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-neutral-400"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
      />
    </svg>
    <input
      type="text"
      bind:value={searchQuery}
      placeholder="搜索插件..."
      class="bg-background text-foreground w-64 rounded border border-neutral-300 py-2 pr-4 pl-10 text-sm focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:focus:border-neutral-400"
    />
  </div>

  <!-- 全部/已安装切换 -->
  <div class="flex rounded border border-neutral-300 dark:border-neutral-600">
    <Button.Root
      class="px-3 py-2 text-sm {showAllPlugins
        ? 'bg-neutral-200 text-neutral-900 dark:bg-neutral-600 dark:text-neutral-100'
        : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-700'}"
      onclick={() => (showAllPlugins = true)}
    >
      全部
    </Button.Root>
    <Button.Root
      class="px-3 py-2 text-sm {!showAllPlugins
        ? 'bg-neutral-200 text-neutral-900 dark:bg-neutral-600 dark:text-neutral-100'
        : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-700'}"
      onclick={() => (showAllPlugins = false)}
    >
      已安装
    </Button.Root>
  </div>

  <!-- 手动导入插件按钮 -->
  <Button.Root
    class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex h-9 items-center justify-center px-4 text-sm font-medium active:scale-[0.98] active:transition-all"
    onclick={() => dispatch('importPlugin')}
  >
    <svg
      class="mr-2 h-4 w-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M12 4v16m8-8H4"
      />
    </svg>
    手动导入
  </Button.Root>
</div>