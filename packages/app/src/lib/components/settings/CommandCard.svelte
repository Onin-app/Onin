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
    /** 显示模式：function 只显示别名，match 只显示匹配规则，all 全部显示 */
    mode?: "function" | "match" | "all";
    onExecute: (commandName: string) => void;
    onToggleKeyword: (commandName: string, keywordName: string) => void;
    onAddKeyword: (commandName: string, keyword: string) => void;
    onRemoveKeyword: (commandName: string, keywordName: string) => void;
  }

  let {
    command,
    mode = "all",
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

  <!-- 描述信息 -->
  {#if command.description}
    <p class="text-xs text-neutral-500 dark:text-neutral-400">
      {command.description}
    </p>
  {/if}

  <!-- 匹配规则显示区域（function 模式下隐藏） -->
  {#if mode !== "function" && command.matches && command.matches.length > 0}
    <div class="flex flex-col gap-1.5">
      {#each command.matches as match}
        <div class="flex flex-wrap items-center gap-1.5 text-xs">
          <!-- 匹配类型标签 -->
          <span
            class="inline-flex items-center gap-1 rounded-md px-2 py-0.5 font-medium
            {match.type === 'text'
              ? 'bg-blue-50 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400'
              : match.type === 'image'
                ? 'bg-purple-50 text-purple-600 dark:bg-purple-900/30 dark:text-purple-400'
                : match.type === 'file'
                  ? 'bg-green-50 text-green-600 dark:bg-green-900/30 dark:text-green-400'
                  : 'bg-amber-50 text-amber-600 dark:bg-amber-900/30 dark:text-amber-400'}"
          >
            {#if match.type === "text"}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><path d="M17 6.1H3" /><path d="M21 12.1H3" /><path
                  d="M15.1 18H3"
                /></svg
              >
              文本
            {:else if match.type === "image"}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><rect
                  width="18"
                  height="18"
                  x="3"
                  y="3"
                  rx="2"
                  ry="2"
                /><circle cx="9" cy="9" r="2" /><path
                  d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"
                /></svg
              >
              图片
            {:else if match.type === "file"}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><path
                  d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"
                /><path d="M14 2v4a2 2 0 0 0 2 2h4" /></svg
              >
              文件
            {:else}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><path
                  d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"
                /></svg
              >
              文件夹
            {/if}
          </span>

          <!-- 匹配规则详情 -->
          {#if match.regexp}
            <span
              class="rounded bg-neutral-100 px-1.5 py-0.5 font-mono text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
            >
              /{match.regexp}/
            </span>
          {/if}

          {#if match.extensions && match.extensions.length > 0}
            <span class="text-neutral-500 dark:text-neutral-400">
              扩展名: {match.extensions.join(", ")}
            </span>
          {/if}

          {#if match.min != null || match.max != null}
            <span class="text-neutral-500 dark:text-neutral-400">
              {#if match.type === "text"}
                {#if match.min != null && match.max != null}
                  {match.min}-{match.max} 字符
                {:else if match.min != null}
                  ≥{match.min} 字符
                {:else if match.max != null}
                  ≤{match.max} 字符
                {/if}
              {:else if match.min != null && match.max != null}
                {match.min}-{match.max} 个
              {:else if match.min != null}
                ≥{match.min} 个
              {:else if match.max != null}
                ≤{match.max} 个
              {/if}
            </span>
          {/if}

          <!-- 匹配名称和描述 -->
          {#if match.description}
            <span class="text-neutral-400 dark:text-neutral-500">
              ({match.description})
            </span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  <!-- 关键词列表（match 模式下隐藏） -->
  {#if mode !== "match"}
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
            aria-label="删除关键词"
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
        onblur={(e) => {
          if (e.currentTarget.value.trim()) {
            onAddKeyword(command.name, e.currentTarget.value);
            e.currentTarget.value = "";
          }
        }}
      />
    </div>
  </div>
  {/if}
</div>
