<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { marked } from "marked";
  import { onDestroy, onMount } from "svelte";
  import FileTypeIcon from "$lib/components/FileTypeIcon.svelte";
  import { Theme } from "$lib/type";
  import { inferMimeType } from "$lib/utils/mimeTypeMap";
  import { getTheme, theme } from "$lib/utils/theme";

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
  let highlightedHtml = $state<string | null>(null);
  let renderedMarkdownHtml = $state<string | null>(null);
  let isHighlighting = $state(false);
  let highlightRequestId = 0;
  let currentResolvedTheme = $state<Theme.DARK | Theme.LIGHT>(Theme.DARK);
  let unsubscribeTheme: (() => void) | null = null;

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

  const getExtension = (name: string) => {
    const index = name.lastIndexOf(".");
    if (index <= 0 || index === name.length - 1) return "";
    return name.slice(index + 1).toLowerCase();
  };

  const getHighlightLanguage = (name: string, type: string) => {
    const ext = getExtension(name);
    const byExtension: Record<string, string> = {
      c: "c",
      conf: "ini",
      cpp: "cpp",
      css: "css",
      go: "go",
      h: "c",
      htm: "html",
      html: "html",
      ini: "ini",
      java: "java",
      js: "javascript",
      json: "json",
      jsx: "jsx",
      log: "log",
      markdown: "markdown",
      md: "markdown",
      php: "php",
      ps1: "powershell",
      py: "python",
      rb: "ruby",
      rs: "rust",
      sh: "shellscript",
      sql: "sql",
      svelte: "svelte",
      toml: "toml",
      ts: "typescript",
      tsx: "tsx",
      txt: "text",
      vue: "vue",
      xml: "xml",
      yaml: "yaml",
      yml: "yaml",
    };

    if (byExtension[ext]) return byExtension[ext];
    if (type === "application/json") return "json";
    if (type === "application/xml" || type === "text/xml") return "xml";
    if (type === "text/markdown") return "markdown";
    if (type === "text/css") return "css";
    if (type === "text/html") return "html";
    if (type === "text/javascript") return "javascript";
    return "text";
  };

  const isMarkdownFile = (name: string, type: string) => {
    const ext = getExtension(name);
    return ext === "md" || ext === "markdown" || type === "text/markdown";
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

  const highlightTextPreview = async (
    content: string,
    language: string,
    resolvedTheme: Theme.DARK | Theme.LIGHT,
    requestId: number,
  ) => {
    isHighlighting = true;
    highlightedHtml = null;

    try {
      const { codeToHtml } = await import("shiki");
      const html = await codeToHtml(content, {
        lang: language,
        theme: resolvedTheme === Theme.DARK ? "github-dark" : "github-light",
      });

      if (requestId !== highlightRequestId) return;
      highlightedHtml = html;
    } catch (error) {
      console.warn("[FilePreview] Failed to highlight text preview:", error);
      if (requestId !== highlightRequestId) return;
      highlightedHtml = null;
    } finally {
      if (requestId === highlightRequestId) {
        isHighlighting = false;
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

  $effect(() => {
    highlightRequestId += 1;
    const requestId = highlightRequestId;

    if (!textPreview?.content) {
      highlightedHtml = null;
      renderedMarkdownHtml = null;
      isHighlighting = false;
      return;
    }

    if (isMarkdownFile(previewName, mimeType)) {
      highlightedHtml = null;
      isHighlighting = false;
      renderedMarkdownHtml = marked.parse(textPreview.content, {
        async: false,
      }) as string;
      return;
    }

    renderedMarkdownHtml = null;
    const language = getHighlightLanguage(previewName, mimeType);
    void highlightTextPreview(
      textPreview.content,
      language,
      currentResolvedTheme,
      requestId,
    );
  });

  onMount(() => {
    unsubscribeTheme = theme.subscribe((value) => {
      currentResolvedTheme = getTheme(value);
    });
  });

  onDestroy(() => {
    unsubscribeTheme?.();
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
      <div class="min-h-full overflow-auto text-xs leading-relaxed">
        {#if renderedMarkdownHtml}
          <div
            class="prose prose-sm dark:prose-invert file-preview-markdown max-w-none p-4"
          >
            {@html renderedMarkdownHtml}
          </div>
        {:else if highlightedHtml}
          <div class="file-preview-code">
            {@html highlightedHtml}
          </div>
        {:else}
          <pre
            class="min-h-full p-4 font-mono whitespace-pre-wrap text-neutral-800 dark:text-neutral-200"><code
              >{textPreview.content}</code
            ></pre>
        {/if}
      </div>
      {#if textPreview.truncated}
        <div
          class="border-t border-amber-200 bg-amber-50 px-4 py-2 text-xs text-amber-700 dark:border-amber-900/60 dark:bg-amber-950/40 dark:text-amber-300"
        >
          文件较大，仅显示前 {formatBytes(textPreview.bytes_read)}。
        </div>
      {:else if isHighlighting}
        <div
          class="border-t border-neutral-200 bg-neutral-50 px-4 py-2 text-xs text-neutral-500 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-400"
        >
          正在应用语法高亮...
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

<style>
  :global(.file-preview-code pre.shiki) {
    min-height: 100%;
    margin: 0;
    padding: 1rem;
    overflow: visible;
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
      "Courier New", monospace;
    font-size: 0.75rem;
    line-height: 1.625;
    white-space: pre-wrap;
    word-break: break-word;
    background: transparent !important;
  }

  :global(.file-preview-code code) {
    font-family: inherit;
  }

  :global(.file-preview-markdown) {
    color: inherit;
  }

  :global(.file-preview-markdown pre) {
    overflow-x: auto;
  }
</style>
