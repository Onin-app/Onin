<script lang="ts">
  /**
   * CommandCard Component
   *
   * 单个指令卡片组件
   * 显示指令标题和关键词，支持添加/删除/启用禁用关键词
   */
  import {
    DropdownMenu,
    DropdownMenuTrigger,
    DropdownMenuContent,
    DropdownMenuItem,
  } from "$lib/components/ui/dropdown-menu";
  import { Card } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { X } from "phosphor-svelte";
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

<Card
  class="group/card hover:border-border flex flex-col gap-2 p-3 transition-all"
>
  <!-- 标题 -->
  <div class="flex items-center justify-between">
    <h4 class="text-foreground text-sm font-semibold">
      {command.title}
    </h4>
  </div>

  <!-- 描述信息 -->
  {#if command.description}
    <p class="text-muted-foreground text-xs">
      {command.description}
    </p>
  {/if}

  <!-- 匹配规则显示区域（function 模式下隐藏） -->
  {#if mode !== "function" && command.matches && command.matches.length > 0}
    <div class="flex flex-col gap-1.5">
      {#each command.matches as match}
        <div class="flex flex-wrap items-center gap-1.5 text-xs">
          <!-- 匹配类型标签 -->
          <Badge
            variant="secondary"
            class="gap-1 px-2 py-0.5 text-[11px] font-medium"
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
          </Badge>

          <!-- 匹配规则详情 -->
          {#if match.regexp}
            <span
              class="bg-muted text-muted-foreground rounded px-1.5 py-0.5 font-mono"
            >
              /{match.regexp}/
            </span>
          {/if}

          {#if match.extensions && match.extensions.length > 0}
            <span class="text-muted-foreground">
              扩展名: {match.extensions.join(", ")}
            </span>
          {/if}

          {#if match.min != null || match.max != null}
            <span class="text-muted-foreground">
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
            <span class="text-muted-foreground/70">
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
          class="group/chip bg-muted/50 text-foreground relative inline-flex items-center rounded-md border px-2 py-0.5 text-sm font-medium transition-colors
        {keyword.disabled ? 'line-through opacity-50' : 'hover:bg-muted'}"
        >
          <!-- 关键词下拉菜单 -->
          <DropdownMenu>
            <DropdownMenuTrigger
              class="cursor-pointer text-xs outline-none select-none"
            >
              {keyword.name}
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" class="w-32">
              <DropdownMenuItem onclick={() => onExecute(command.name)}>
                执行指令
              </DropdownMenuItem>
              <DropdownMenuItem
                onclick={() => onToggleKeyword(command.name, keyword.name)}
              >
                {keyword.disabled ? "启用指令" : "禁用指令"}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <!-- 删除按钮（仅非默认关键词显示） -->
          {#if !keyword.is_default}
            <button
              class="text-muted-foreground hover:bg-destructive/10 hover:text-destructive -mr-0.5 ml-1 cursor-pointer rounded-full p-0.5 opacity-0 transition-all group-hover/chip:opacity-100"
              aria-label="删除关键词"
              onclick={(e) => {
                e.stopPropagation();
                onRemoveKeyword(command.name, keyword.name);
              }}
            >
              <X class="h-2.5 w-2.5" />
            </button>
          {/if}
        </div>
      {/each}

      <!-- 添加关键词输入框 -->
      <div class="relative flex items-center">
        <input
          type="text"
          placeholder="+ 添加"
          class="border-input text-muted-foreground placeholder:text-muted-foreground/60 focus:border-ring focus:bg-background focus:text-foreground h-[26px] w-16 rounded-md border border-dashed bg-transparent px-2 text-xs transition-all focus:w-24 focus:border-solid focus:outline-none"
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
</Card>
