<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Copy, DownloadSimple, X } from "phosphor-svelte";
  import "../../index.css";

  interface CaptureInfo {
    logicalWidth: number;
    logicalHeight: number;
    scaleFactor: number;
  }

  type Selection = { x: number; y: number; width: number; height: number };
  type DragMode =
    | "new"
    | "move"
    | "nw"
    | "n"
    | "ne"
    | "e"
    | "se"
    | "s"
    | "sw"
    | "w";

  let surface: HTMLDivElement;
  let capture: CaptureInfo | null = null;
  let selection: Selection | null = null;
  let start: { x: number; y: number } | null = null;
  let startSelection: Selection | null = null;
  let dragMode: DragMode | null = null;
  let busy = false;
  let error = "";

  const minimumSize = 8;
  const resizeHandles: ReadonlyArray<[DragMode, string]> = [
    ["nw", "-top-1.5 -left-1.5 cursor-nwse-resize"],
    ["n", "-top-1.5 left-1/2 -translate-x-1/2 cursor-ns-resize"],
    ["ne", "-top-1.5 -right-1.5 cursor-nesw-resize"],
    ["e", "top-1/2 -right-1.5 -translate-y-1/2 cursor-ew-resize"],
    ["se", "-right-1.5 -bottom-1.5 cursor-nwse-resize"],
    ["s", "-bottom-1.5 left-1/2 -translate-x-1/2 cursor-ns-resize"],
    ["sw", "-bottom-1.5 -left-1.5 cursor-nesw-resize"],
    ["w", "top-1/2 -left-1.5 -translate-y-1/2 cursor-ew-resize"],
  ];

  function point(event: MouseEvent) {
    const rect = surface.getBoundingClientRect();
    return {
      x: Math.max(0, Math.min(rect.width, event.clientX - rect.left)),
      y: Math.max(0, Math.min(rect.height, event.clientY - rect.top)),
    };
  }

  function beginSelection(event: MouseEvent, mode: DragMode = "new") {
    if (busy || event.button !== 0) return;
    error = "";
    start = point(event);
    startSelection = selection ? { ...selection } : null;
    dragMode = mode;
    if (mode === "new") {
      selection = { x: start.x, y: start.y, width: 0, height: 0 };
    }
  }

  function updateSelection(event: MouseEvent) {
    if (!start || !dragMode) return;
    const current = point(event);
    const viewportWidth = capture?.logicalWidth ?? surface.clientWidth;
    const viewportHeight = capture?.logicalHeight ?? surface.clientHeight;

    if (dragMode === "new") {
      selection = {
        x: Math.min(start.x, current.x),
        y: Math.min(start.y, current.y),
        width: Math.abs(current.x - start.x),
        height: Math.abs(current.y - start.y),
      };
      return;
    }

    if (!startSelection) return;
    const deltaX = current.x - start.x;
    const deltaY = current.y - start.y;
    let { x, y, width, height } = startSelection;

    if (dragMode === "move") {
      selection = {
        x: Math.max(0, Math.min(viewportWidth - width, x + deltaX)),
        y: Math.max(0, Math.min(viewportHeight - height, y + deltaY)),
        width,
        height,
      };
      return;
    }

    if (dragMode.includes("w")) {
      const nextX = Math.max(0, Math.min(x + width - minimumSize, x + deltaX));
      width += x - nextX;
      x = nextX;
    }
    if (dragMode.includes("e")) {
      width = Math.max(
        minimumSize,
        Math.min(viewportWidth - x, width + deltaX),
      );
    }
    if (dragMode.includes("n")) {
      const nextY = Math.max(0, Math.min(y + height - minimumSize, y + deltaY));
      height += y - nextY;
      y = nextY;
    }
    if (dragMode.includes("s")) {
      height = Math.max(
        minimumSize,
        Math.min(viewportHeight - y, height + deltaY),
      );
    }
    selection = {
      x,
      y,
      width,
      height,
    };
  }

  function endSelection() {
    if (!start) return;
    const wasNewSelection = dragMode === "new";
    start = null;
    startSelection = null;
    dragMode = null;
    if (
      wasNewSelection &&
      (!selection ||
        selection.width < minimumSize ||
        selection.height < minimumSize)
    ) {
      selection = null;
    }
  }

  async function finish(restoreLauncher: boolean) {
    await invoke("finish_screenshot_selection", { restoreLauncher });
  }

  async function cancel() {
    await finish(true);
  }

  function selectedRect() {
    if (!selection) throw new Error("请先拖动选择截图区域");
    return selection;
  }

  async function copy() {
    if (busy) return;
    try {
      busy = true;
      error = "";
      await invoke("copy_screenshot_region", { rect: selectedRect() });
      await finish(false);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : "复制截图失败";
      busy = false;
    }
  }

  async function saveImage() {
    if (busy) return;
    try {
      const path = await save({
        title: "保存截图",
        defaultPath: `Onin截图_${new Date().toISOString().slice(0, 19).replaceAll(":", "-")}.png`,
        filters: [{ name: "PNG 图片", extensions: ["png"] }],
      });
      if (!path) return;
      busy = true;
      error = "";
      await invoke("save_screenshot_region", { rect: selectedRect(), path });
      await finish(false);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : "保存截图失败";
      busy = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      void cancel();
    } else if (event.key === "Enter" && selection) {
      void copy();
    }
  }

  onMount(async () => {
    try {
      capture = await invoke<CaptureInfo>("get_screenshot_overlay_info");
      selection = {
        width: Math.min(
          Math.max(320, capture.logicalWidth * 0.6),
          capture.logicalWidth - 24,
        ),
        height: Math.min(
          Math.max(220, capture.logicalHeight * 0.6),
          capture.logicalHeight - 24,
        ),
        x: 0,
        y: 0,
      };
      selection.x = Math.round((capture.logicalWidth - selection.width) / 2);
      selection.y = Math.round((capture.logicalHeight - selection.height) / 2);
      const currentWindow = getCurrentWindow();
      await currentWindow.show();
      await currentWindow.setFocus().catch(() => {});
    } catch (reason) {
      error = reason instanceof Error ? reason.message : "截图加载失败";
    }
    window.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<main
  bind:this={surface}
  class="relative h-screen w-screen cursor-crosshair overflow-hidden bg-transparent select-none"
  onmousemove={updateSelection}
  onmouseup={endSelection}
  onmouseleave={endSelection}
  onmousedown={(event) => {
    if (event.target === surface) beginSelection(event);
  }}
>
  {#if selection}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="absolute cursor-move border border-white/95 shadow-[0_0_0_1px_rgba(0,0,0,0.65)]"
      style="left: {selection.x}px; top: {selection.y}px; width: {selection.width}px; height: {selection.height}px;"
      onmousedown={(event) => beginSelection(event, "move")}
    >
      {#each resizeHandles as [mode, positionClass]}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="absolute z-10 size-3 rounded-sm border border-neutral-900 bg-white shadow-sm {positionClass}"
          onmousedown={(event) => {
            event.stopPropagation();
            beginSelection(event, mode);
          }}
        ></div>
      {/each}
    </div>
    <div
      class="pointer-events-none absolute bg-black/55"
      style="inset: 0 0 auto 0; height: {selection.y}px;"
    ></div>
    <div
      class="pointer-events-none absolute right-0 bottom-0 left-0 bg-black/55"
      style="top: {selection.y + selection.height}px;"
    ></div>
    <div
      class="pointer-events-none absolute bg-black/55"
      style="left: 0; top: {selection.y}px; width: {selection.x}px; height: {selection.height}px;"
    ></div>
    <div
      class="pointer-events-none absolute bg-black/55"
      style="right: 0; top: {selection.y}px; left: {selection.x +
        selection.width}px; height: {selection.height}px;"
    ></div>

    <section
      class="absolute flex items-center gap-1 rounded-xl border border-white/15 bg-neutral-950/92 p-1.5 text-white shadow-2xl backdrop-blur"
      style="left: {Math.max(
        12,
        Math.min(selection.x, (capture?.logicalWidth ?? 240) - 228),
      )}px; top: {selection.y + selection.height + 12 >
      (capture?.logicalHeight ?? 0) - 52
        ? Math.max(12, selection.y - 52)
        : selection.y + selection.height + 12}px;"
    >
      <span class="px-2 font-mono text-[11px] text-neutral-300">
        {Math.round(selection.width * (capture?.scaleFactor ?? 1))} × {Math.round(
          selection.height * (capture?.scaleFactor ?? 1),
        )}
      </span>
      <button
        class="flex items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs font-medium text-neutral-300 transition hover:bg-white/10 hover:text-white disabled:opacity-50"
        onclick={cancel}
        disabled={busy}><X class="size-3.5" />取消</button
      >
      <button
        class="flex items-center gap-1 rounded-lg bg-white/12 px-2.5 py-1.5 text-xs font-medium text-white transition hover:bg-white/20 disabled:opacity-50"
        onclick={saveImage}
        disabled={busy}><DownloadSimple class="size-3.5" />保存</button
      >
      <button
        class="flex items-center gap-1 rounded-lg bg-emerald-400 px-3 py-1.5 text-xs font-semibold text-neutral-950 transition hover:bg-emerald-300 disabled:opacity-50"
        onclick={copy}
        disabled={busy}
        ><Copy class="size-3.5" />{busy ? "处理中" : "复制"}</button
      >
    </section>
  {:else}
    <div class="pointer-events-none absolute inset-0 bg-black/28"></div>
    <div
      class="pointer-events-none absolute bottom-9 left-1/2 -translate-x-1/2 rounded-full border border-white/15 bg-neutral-950/85 px-4 py-2 text-xs text-white shadow-xl backdrop-blur"
    >
      拖动框选区域 <span class="mx-1 text-white/35">·</span> Esc 取消
    </div>
  {/if}

  {#if error}
    <div
      class="absolute top-5 left-1/2 flex -translate-x-1/2 items-center gap-2 rounded-lg bg-red-500 px-3 py-2 text-xs font-medium text-white shadow-xl"
    >
      {error}<button onclick={cancel} aria-label="关闭截图"
        ><X class="size-3.5" /></button
      >
    </div>
  {/if}
</main>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    overflow: hidden;
    background: transparent !important;
  }
</style>
