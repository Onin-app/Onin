<script lang="ts">
  /**
   * CommandCard Component
   *
   * 单个指令卡片组件
   * 显示指令标题和关键词，支持添加/删除/启用禁用关键词
   */
  import { Button, DropdownMenu } from "bits-ui";
  import type { Command } from "$lib/type";

  // Props 接口
  interface Props {
    command: Command;
    onExecute: (commandName: string) => void;
    onToggleKeyword: (commandName: string, keywordName: string) => void;
    onAddKeyword: (commandName: string, keyword: string) => void;
    onRemoveKeyword: (commandName: string, keywordName: string) => void;
  }

  let {
    command,
    onExecute,
    onToggleKeyword,
    onAddKeyword,
    onRemoveKeyword,
  }: Props = $props();
</script>

<div
  class="group/card flex flex-col gap-2 rounded-xl border border-neutral-200 bg-white p-3 transition-all hover:border-neutral-300 dark:border-neutral-800 dark:bg-neutral-900 dark:hover:border-neutral-700"
>
  <!-- 标题 -->
  <div class="flex items-center justify-between">
    <h4 class="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
      {command.title}
    </h4>
  </div>

  <!-- 关键词列表 -->
  <div class="flex flex-wrap gap-1.5">
    {#each command.keywords as keyword}
      <div
        class="group/chip relative inline-flex items-center rounded-md border border-neutral-200 bg-neutral-50 px-2 py-0.5 text-sm font-medium text-neutral-600 transition-colors dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300
        {keyword.disabled
          ? 'line-through opacity-50'
          : 'hover:bg-neutral-100 dark:hover:bg-neutral-700/80'}"
      >
        <!-- 关键词下拉菜单 -->
        <DropdownMenu.Root>
          <DropdownMenu.Trigger class="cursor-default outline-none select-none">
            {keyword.name}
          </DropdownMenu.Trigger>
          <DropdownMenu.Portal>
            <DropdownMenu.Content
              class="animate-in fade-in-0 zoom-in-95 z-50 min-w-[140px] overflow-hidden rounded-lg border border-neutral-200 bg-white p-1 shadow-lg dark:border-neutral-800 dark:bg-neutral-900"
              sideOffset={4}
              align="start"
            >
              <DropdownMenu.Item
                class="relative flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-xs text-neutral-700 transition-colors outline-none select-none hover:bg-neutral-100 data-[disabled]:pointer-events-none data-[disabled]:opacity-50 dark:text-neutral-200 dark:hover:bg-neutral-800"
              >
                <Button.Root
                  class="w-full text-left"
                  onclick={() => onExecute(command.name)}
                >
                  执行指令
                </Button.Root>
              </DropdownMenu.Item>
              <DropdownMenu.Item
                class="relative flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-xs text-neutral-700 transition-colors outline-none select-none hover:bg-neutral-100 data-[disabled]:pointer-events-none data-[disabled]:opacity-50 dark:text-neutral-200 dark:hover:bg-neutral-800"
              >
                <Button.Root
                  class="w-full text-left"
                  onclick={() => onToggleKeyword(command.name, keyword.name)}
                >
                  {keyword.disabled ? "启用指令" : "禁用指令"}
                </Button.Root>
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Portal>
        </DropdownMenu.Root>

        <!-- 删除按钮（仅非默认关键词显示） -->
        {#if !keyword.is_default}
          <button
            class="-mr-0.5 ml-1 rounded-full p-0.5 text-neutral-400 opacity-0 transition-all group-hover/chip:opacity-100 hover:bg-neutral-200 hover:text-red-500 dark:text-neutral-500 dark:hover:bg-neutral-700"
            onclick={(e) => {
              e.stopPropagation();
              onRemoveKeyword(command.name, keyword.name);
            }}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="10"
              height="10"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M18 6 6 18" />
              <path d="m6 6 12 12" />
            </svg>
          </button>
        {/if}
      </div>
    {/each}

    <!-- 添加关键词输入框 -->
    <div class="relative flex items-center">
      <input
        type="text"
        placeholder="+ 添加"
        class="h-[28px] w-16 rounded-md border border-dashed border-neutral-300 bg-transparent px-2 text-sm text-neutral-500 transition-all placeholder:text-neutral-400 focus:w-24 focus:border-solid focus:border-neutral-400 focus:bg-white focus:text-neutral-900 focus:outline-none dark:border-neutral-700 dark:text-neutral-400 dark:focus:border-neutral-600 dark:focus:bg-neutral-900 dark:focus:text-neutral-100"
        onkeydown={(e) => {
          if (e.key === "Enter") {
            onAddKeyword(command.name, e.currentTarget.value);
            e.currentTarget.value = "";
          }
        }}
      />
    </div>
  </div>
</div>
