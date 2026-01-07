<script lang="ts">
  /**
   * SearchInput Component
   *
   * 搜索输入区域组件
   * 包含文本/文件附件展示和输入框
   */
  import autoAnimate from "@formkit/auto-animate";
  import type { Action } from "svelte/action";
  import { CaretRight, CaretLeft } from "phosphor-svelte";
  import FileAttachment from "./FileAttachment.svelte";
  import TextAttachment from "./TextAttachment.svelte";

  interface Props {
    value: string;
    attachedText: string;
    attachedFiles: File[];
    showAllFiles: boolean;
    placeholder?: string;
    onInput: (value: string) => void;
    onPaste: (e: ClipboardEvent) => void;
    onDrop: (e: DragEvent) => void;
    onDragOver: (e: DragEvent) => void;
    onRemoveFile: (index: number) => void;
    onRemoveText: () => void;
    onEditText: () => void;
    onToggleShowAllFiles: () => void;
    onBackspace: () => void;
  }

  let {
    value = $bindable(),
    attachedText,
    attachedFiles,
    showAllFiles,
    placeholder = "Hi Onin!",
    onInput,
    onPaste,
    onDrop,
    onDragOver,
    onRemoveFile,
    onRemoveText,
    onEditText,
    onToggleShowAllFiles,
    onBackspace,
  }: Props = $props();

  // AutoAnimate action for file attachments
  const animate: Action<HTMLElement> = (node) => {
    autoAnimate(node, {
      duration: 200,
      easing: "ease-in-out",
    });
  };

  // Input element reference
  let inputElement: HTMLInputElement;

  // 暴露 focus 方法
  export function focus() {
    inputElement?.focus();
  }

  // 暴露 select 方法
  export function select() {
    inputElement?.select();
  }

  const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === "Backspace" && value === "") {
      e.preventDefault();
      onBackspace();
    }
  };
</script>

<div
  class="flex w-full {showAllFiles
    ? 'flex-col gap-2'
    : 'flex-row items-center gap-2'} rounded-lg border border-neutral-300 bg-white px-2 py-2 dark:border-neutral-600 dark:bg-neutral-800"
  ondrop={onDrop}
  ondragover={onDragOver}
  role="region"
  aria-label="输入区域"
>
  {#if attachedText}
    <div use:animate class="flex flex-wrap items-center gap-1.5">
      <TextAttachment
        text={attachedText}
        onEdit={onEditText}
        onRemove={onRemoveText}
      />
    </div>
  {:else if attachedFiles.length > 0}
    <div use:animate class="flex flex-wrap items-center gap-1.5">
      {#if showAllFiles}
        <!-- 展开模式：显示所有文件 -->
        {#each attachedFiles as file, index (file.name + index)}
          <FileAttachment {file} onRemove={() => onRemoveFile(index)} />
        {/each}
      {:else}
        <!-- 折叠模式：只显示第一个文件 -->
        <FileAttachment
          file={attachedFiles[0]}
          onRemove={() => onRemoveFile(0)}
        />
      {/if}
      {#if attachedFiles.length > 1}
        <button
          class="inline-flex h-[34px] items-center gap-1 rounded-md border px-2 text-sm font-medium transition-colors {showAllFiles
            ? 'border-blue-300 bg-blue-50 text-blue-700 hover:bg-blue-100 dark:border-blue-600 dark:bg-blue-900/30 dark:text-blue-300 dark:hover:bg-blue-900/50'
            : 'border-orange-300 bg-orange-50 text-orange-700 hover:bg-orange-100 dark:border-orange-600 dark:bg-orange-900/30 dark:text-orange-300 dark:hover:bg-orange-900/50'}"
          onclick={onToggleShowAllFiles}
          aria-label={showAllFiles ? "收起文件" : "展开所有文件"}
        >
          {#if showAllFiles}
            <CaretLeft class="size-4" weight="bold" />
            <span>收起</span>
          {:else}
            <span>+{attachedFiles.length - 1}</span>
            <CaretRight class="size-4" weight="bold" />
          {/if}
        </button>
      {/if}
    </div>
  {/if}
  <input
    bind:this={inputElement}
    class="{showAllFiles
      ? 'w-full'
      : 'min-w-0 flex-1'} h-[34px] bg-transparent text-2xl focus:outline-none focus:ring-0 active:outline-none active:ring-0"
    type="text"
    {placeholder}
    bind:value
    oninput={(e) => onInput(e.currentTarget.value)}
    onpaste={onPaste}
    onkeydown={handleKeydown}
  />
</div>
