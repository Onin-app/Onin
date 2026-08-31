<script lang="ts">
  /**
   * SearchInput Component
   *
   * 搜索输入区域组件
   * 遵循无缝融合设计：无突兀外边框，直接与主启动器面板融为一体
   */
  import autoAnimate from "@formkit/auto-animate";
  import type { Action } from "svelte/action";
  import { CaretRight, CaretLeft } from "phosphor-svelte";
  import FileAttachment from "./FileAttachment.svelte";
  import TextAttachment from "./TextAttachment.svelte";
  import { Button } from "$lib/components/ui/button";

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

  const animate: Action<HTMLElement> = (node) => {
    autoAnimate(node, {
      duration: 150,
      easing: "cubic-bezier(0.23, 1, 0.32, 1)",
    });
  };

  let inputElement: HTMLInputElement;

  export function focus() {
    inputElement?.focus();
  }

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
    : 'flex-row items-center gap-2'} bg-transparent px-1 py-1"
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
        {#each attachedFiles as file, index (file.name + index)}
          <FileAttachment {file} onRemove={() => onRemoveFile(index)} />
        {/each}
      {:else}
        <FileAttachment
          file={attachedFiles[0]}
          onRemove={() => onRemoveFile(0)}
        />
      {/if}
      {#if attachedFiles.length > 1}
        <Button
          variant="outline"
          size="sm"
          class="h-7 gap-1 px-2 text-xs shadow-2xs transition-[transform,background-color] duration-120 active:scale-95"
          onclick={onToggleShowAllFiles}
          aria-label={showAllFiles ? "收起文件" : "展开所有文件"}
        >
          {#if showAllFiles}
            <CaretLeft class="size-3.5" weight="bold" />
            <span>收起</span>
          {:else}
            <span>+{attachedFiles.length - 1}</span>
            <CaretRight class="size-3.5" weight="bold" />
          {/if}
        </Button>
      {/if}
    </div>
  {/if}
  <input
    id="main-search-input"
    bind:this={inputElement}
    class="{showAllFiles
      ? 'w-full'
      : 'min-w-0 flex-1'} text-foreground placeholder:text-muted-foreground/50 h-10 border-none bg-transparent text-xl font-normal tracking-tight outline-none focus:ring-0 focus:outline-none active:outline-none"
    type="text"
    {placeholder}
    bind:value
    oninput={(e) => onInput(e.currentTarget.value)}
    onpaste={onPaste}
    onkeydown={handleKeydown}
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    autocomplete="off"
  />
</div>
