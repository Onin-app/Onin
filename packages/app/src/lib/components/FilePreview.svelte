<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import FileTypeIcon from "$lib/components/FileTypeIcon.svelte";
  import { inferMimeType } from "$lib/utils/mimeTypeMap";

  interface Props {
    path?: string;
    fileName?: string;
    imageSrc?: string;
    onOpen?: () => void;
  }

  let { path, fileName, imageSrc, onOpen }: Props = $props();

  interface TextPreview {
    content: string;
    truncated: boolean;
    bytes_read: number;
    total_bytes: number;
  }

  let textPreview = $state<TextPreview | null>(null);
  let textPreviewError = $state<string | null>(null);
  let isLoadingTextPreview = $state(false);
  let textPreviewRequestId = 0;

  const previewName = $derived(fileName || path?.split(/[/\\]/).pop() || "");
  const mimeType = $derived(path ? inferMimeType(path) : "");
  const fileSrc = $derived(path ? convertFileSrc(path) : "");
  const isImage = $derived(Boolean(imageSrc) || mimeType.startsWith("image/"));
  const isPdf = $derived(mimeType === "application/pdf");
  const isVideo = $derived(mimeType.startsWith("video/"));
  const isAudio = $derived(mimeType.startsWith("audio/"));
  const isText = $derived(
    Boolean(path) &&
      !isImage &&
      !isPdf &&
      !isVideo &&
      !isAudio &&
      (mimeType.startsWith("text/") ||
        mimeType === "application/json" ||
        mimeType === "application/xml"),
  );

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  };

  const loadTextPreview = async (nextPath: string, requestId: number) => {
    isLoadingTextPreview = true;
    textPreview = null;
    textPreviewError = null;

    try {
      const preview = await invoke<TextPreview>("read_file_text_preview", {
        path: nextPath,
      });

      if (requestId !== textPreviewRequestId) return;
      textPreview = preview;
    } catch (error) {
      if (requestId !== textPreviewRequestId) return;
      textPreviewError = String(error);
    } finally {
      if (requestId === textPreviewRequestId) {
        isLoadingTextPreview = false;
      }
    }
  };

  $effect(() => {
    textPreviewRequestId += 1;
    const requestId = textPreviewRequestId;

    if (!path || !isText) {
      textPreview = null;
      textPreviewError = null;
      isLoadingTextPreview = false;
      return;
    }

    void loadTextPreview(path, requestId);
  });
</script>

{#if isImage}
  <div
    class="flex min-h-full items-center justify-center bg-[url('/checker-board.svg')] bg-repeat p-8"
  >
    <img
      src={imageSrc || fileSrc}
      class="max-h-[75vh] max-w-full rounded border border-neutral-200 shadow-lg dark:border-neutral-700"
      alt="Preview"
    />
  </div>
{:else if isPdf && path}
  <div
    class="flex h-full min-h-full flex-col bg-neutral-100 dark:bg-neutral-950"
  >
    <object
      data={fileSrc}
      type="application/pdf"
      class="h-full min-h-[520px] w-full flex-1"
      aria-label="PDF Preview"
    >
      <div
        class="flex min-h-[360px] flex-col items-center justify-center gap-4 px-8 text-center"
      >
        <FileTypeIcon
          fileType={mimeType}
          fileName={previewName}
          class="h-20 w-20"
        />
        {#if onOpen}
          <button
            class="rounded-md bg-neutral-900 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-neutral-700 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-300"
            onclick={onOpen}
          >
            打开
          </button>
        {/if}
      </div>
    </object>
  </div>
{:else if isVideo && path}
  <div class="flex min-h-full items-center justify-center bg-black p-6">
    <video
      src={fileSrc}
      class="max-h-[75vh] max-w-full rounded border border-neutral-800 shadow-lg"
      controls
      preload="metadata"
    >
      <track kind="captions" />
    </video>
  </div>
{:else if isAudio && path}
  <div
    class="flex min-h-full flex-col items-center justify-center gap-5 bg-neutral-100 px-8 py-10 dark:bg-neutral-950"
  >
    <div
      class="flex h-24 w-24 items-center justify-center rounded-2xl border border-neutral-200 bg-white shadow-sm dark:border-neutral-800 dark:bg-neutral-900"
    >
      <FileTypeIcon
        fileType={mimeType}
        fileName={previewName}
        class="h-14 w-14"
      />
    </div>
    <div
      class="max-w-full truncate text-sm font-medium text-neutral-800 dark:text-neutral-200"
    >
      {previewName}
    </div>
    <audio src={fileSrc} class="w-full max-w-md" controls preload="metadata">
      <track kind="captions" />
    </audio>
  </div>
{:else if isText && path}
  <div class="flex min-h-full flex-col bg-neutral-50 dark:bg-neutral-950">
    <div
      class="flex flex-shrink-0 items-center justify-between gap-3 border-b border-neutral-200 px-4 py-2 text-xs text-neutral-500 dark:border-neutral-800 dark:text-neutral-400"
    >
      <div class="min-w-0 truncate font-mono">{previewName}</div>
      <div class="flex-shrink-0 tabular-nums">
        {#if textPreview}
          {formatBytes(textPreview.total_bytes)}
          {#if textPreview.truncated}
            · preview
          {/if}
        {:else if isLoadingTextPreview}
          Loading
        {:else}
          Text
        {/if}
      </div>
    </div>

    {#if isLoadingTextPreview}
      <div
        class="flex min-h-[240px] flex-1 items-center justify-center text-sm text-neutral-500"
      >
        正在加载预览...
      </div>
    {:else if textPreview}
      <pre
        class="min-h-full overflow-auto p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap text-neutral-800 dark:text-neutral-200"><code
          >{textPreview.content}</code
        ></pre>
      {#if textPreview.truncated}
        <div
          class="border-t border-amber-200 bg-amber-50 px-4 py-2 text-xs text-amber-700 dark:border-amber-900/60 dark:bg-amber-950/40 dark:text-amber-300"
        >
          文件较大，仅显示前 {formatBytes(textPreview.bytes_read)}。
        </div>
      {/if}
    {:else}
      <div
        class="flex min-h-[240px] flex-1 flex-col items-center justify-center gap-4 px-8 text-center"
      >
        <FileTypeIcon
          fileType={mimeType}
          fileName={previewName}
          class="h-14 w-14"
        />
        <div class="max-w-md text-sm text-neutral-500 dark:text-neutral-400">
          {textPreviewError || "无法预览此文本文件"}
        </div>
        {#if onOpen}
          <button
            class="rounded-md bg-neutral-900 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-neutral-700 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-300"
            onclick={onOpen}
          >
            打开
          </button>
        {/if}
      </div>
    {/if}
  </div>
{:else}
  <div
    class="flex min-h-full flex-col items-center justify-center gap-4 px-8 py-10 text-center"
  >
    <div
      class="flex h-24 w-24 items-center justify-center rounded-2xl border border-neutral-200 bg-white shadow-sm dark:border-neutral-800 dark:bg-neutral-900"
    >
      <FileTypeIcon
        fileType={mimeType}
        fileName={previewName}
        class="h-14 w-14"
      />
    </div>
    {#if previewName}
      <div
        class="max-w-full truncate text-sm font-medium text-neutral-800 dark:text-neutral-200"
      >
        {previewName}
      </div>
    {/if}
    {#if onOpen}
      <button
        class="rounded-md bg-neutral-900 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-neutral-700 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-300"
        onclick={onOpen}
      >
        打开
      </button>
    {/if}
  </div>
{/if}
