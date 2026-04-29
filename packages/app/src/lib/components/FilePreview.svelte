<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import FileTypeIcon from "$lib/components/FileTypeIcon.svelte";
  import { inferMimeType } from "$lib/utils/mimeTypeMap";

  interface Props {
    path?: string;
    fileName?: string;
    imageSrc?: string;
    onOpen?: () => void;
  }

  let { path, fileName, imageSrc, onOpen }: Props = $props();

  const previewName = $derived(fileName || path?.split(/[/\\]/).pop() || "");
  const mimeType = $derived(path ? inferMimeType(path) : "");
  const fileSrc = $derived(path ? convertFileSrc(path) : "");
  const isImage = $derived(Boolean(imageSrc) || mimeType.startsWith("image/"));
  const isPdf = $derived(mimeType === "application/pdf");
  const isVideo = $derived(mimeType.startsWith("video/"));
  const isAudio = $derived(mimeType.startsWith("audio/"));
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
