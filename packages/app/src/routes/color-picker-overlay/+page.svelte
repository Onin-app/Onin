<script lang="ts">
  /**
   * 取色 Overlay 页面
   *
   * 架构：
   * - 不透明全屏窗口，用截图做背景（Windows WebView2 不支持真正透明）
   * - 每帧：绘制截图背景 → 十字线 → 鼠标旁放大镜
   * - 左键点击 → 取色并返回 hex
   * - Esc / 右键 → 取消
   */
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  // ── 截图数据结构 ────────────────────────────────────────────────────────────

  interface ColorPickerCapture {
    width: number;
    height: number;
    logicalWidth: number;
    logicalHeight: number;
    scaleFactor: number;
  }

  // ── 放大镜配置 ──────────────────────────────────────────────────────────────

  const GRID_RADIUS = 7;
  const CELL = 9;
  const GRID_PX = (GRID_RADIUS * 2 + 1) * CELL; // 135
  const LOUPE_W = GRID_PX + 20; // 155
  const LABEL_H = 48;
  const LOUPE_H = GRID_PX + LABEL_H + 16; // 199
  const OFFSET = 22;
  const MARGIN = 16;

  // ── 状态 ────────────────────────────────────────────────────────────────────

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;

  /** 截图原始 ImageData（作为 ImageBitmap 不可用时的背景 fallback） */
  let bgFrame: ImageData | null = null;
  /** GPU 侧背景位图（用于每帧绘制背景） */
  let bgBitmap: ImageBitmap | null = null;
  /** 截图物理像素 ImageData（用于颜色采样） */
  let imageData: ImageData | null = null;
  let capture: ColorPickerCapture | null = null;

  let mouseX = 0;
  let mouseY = 0;
  let currentHex = "#000000";
  let currentRgb = { r: 0, g: 0, b: 0 };

  let done = false;
  let loading = false;
  let rafId = 0;
  let loadId = 0;

  // ── 颜色采样 ────────────────────────────────────────────────────────────────

  function samplePixel(lx: number, ly: number) {
    if (!imageData || !capture) return { r: 0, g: 0, b: 0, hex: "#000000" };

    const px = Math.max(
      0,
      Math.min(capture.width - 1, Math.round(lx * capture.scaleFactor)),
    );
    const py = Math.max(
      0,
      Math.min(capture.height - 1, Math.round(ly * capture.scaleFactor)),
    );
    const off = (py * capture.width + px) * 4;
    const r = imageData.data[off];
    const g = imageData.data[off + 1];
    const b = imageData.data[off + 2];
    const hex =
      `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`.toUpperCase();
    return { r, g, b, hex };
  }

  // ── 渲染 ────────────────────────────────────────────────────────────────────

  function draw() {
    if (!ctx || !canvas || !imageData || !capture) return;

    // ① 绘制截图背景（铺满整个 canvas）
    if (bgBitmap) {
      ctx.drawImage(bgBitmap, 0, 0, canvas.width, canvas.height);
    } else if (bgFrame) {
      ctx.putImageData(bgFrame, 0, 0);
    } else {
      return;
    }

    // ② 采样当前颜色
    const color = samplePixel(mouseX, mouseY);
    currentHex = color.hex;
    currentRgb = { r: color.r, g: color.g, b: color.b };

    // ③ 十字线
    const x = mouseX;
    const y = mouseY;

    ctx.save();
    // 白色实线
    ctx.strokeStyle = "rgba(255,255,255,0.85)";
    ctx.lineWidth = 1;
    ctx.setLineDash([]);
    ctx.beginPath();
    ctx.moveTo(x + 0.5, 0);
    ctx.lineTo(x + 0.5, canvas.height);
    ctx.moveTo(0, y + 0.5);
    ctx.lineTo(canvas.width, y + 0.5);
    ctx.stroke();
    // 黑色虚线叠加
    ctx.strokeStyle = "rgba(0,0,0,0.65)";
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(x + 0.5, 0);
    ctx.lineTo(x + 0.5, canvas.height);
    ctx.moveTo(0, y + 0.5);
    ctx.lineTo(canvas.width, y + 0.5);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.restore();

    // ④ 放大镜面板位置
    let px = mouseX + OFFSET;
    let py = mouseY + OFFSET;
    if (px + LOUPE_W > canvas.width - MARGIN) px = mouseX - LOUPE_W - OFFSET;
    if (py + LOUPE_H > canvas.height - MARGIN) py = mouseY - LOUPE_H - OFFSET;
    px = Math.max(MARGIN, Math.min(canvas.width - LOUPE_W - MARGIN, px));
    py = Math.max(MARGIN, Math.min(canvas.height - LOUPE_H - MARGIN, py));

    drawLoupe(px, py);
  }

  function drawLoupe(px: number, py: number) {
    if (!ctx || !imageData || !capture) return;

    // 面板背景
    ctx.save();
    ctx.beginPath();
    ctx.roundRect(px, py, LOUPE_W, LOUPE_H, 12);
    ctx.fillStyle = "rgba(18, 18, 20, 0.92)";
    ctx.fill();
    ctx.strokeStyle = "rgba(255,255,255,0.15)";
    ctx.lineWidth = 1;
    ctx.stroke();
    ctx.restore();

    // 格子区域
    const gridLeft = px + Math.floor((LOUPE_W - GRID_PX) / 2);
    const gridTop = py + 8;
    const originPx = Math.round(mouseX * capture.scaleFactor) - GRID_RADIUS;
    const originPy = Math.round(mouseY * capture.scaleFactor) - GRID_RADIUS;
    const gridSize = GRID_RADIUS * 2 + 1;

    for (let row = 0; row < gridSize; row++) {
      for (let col = 0; col < gridSize; col++) {
        const sx = Math.max(0, Math.min(capture.width - 1, originPx + col));
        const sy = Math.max(0, Math.min(capture.height - 1, originPy + row));
        const off = (sy * capture.width + sx) * 4;
        ctx.fillStyle = `rgb(${imageData.data[off]},${imageData.data[off + 1]},${imageData.data[off + 2]})`;
        ctx.fillRect(gridLeft + col * CELL, gridTop + row * CELL, CELL, CELL);
      }
    }

    // 中心格子高亮
    const cx = gridLeft + GRID_RADIUS * CELL;
    const cy = gridTop + GRID_RADIUS * CELL;
    ctx.strokeStyle = "#fff";
    ctx.lineWidth = 2;
    ctx.strokeRect(cx - 1, cy - 1, CELL + 2, CELL + 2);
    ctx.strokeStyle = "rgba(0,0,0,0.5)";
    ctx.lineWidth = 1;
    ctx.strokeRect(cx, cy, CELL, CELL);

    // 分割线
    const labelY = gridTop + GRID_PX + 8;
    ctx.fillStyle = "rgba(255,255,255,0.08)";
    ctx.fillRect(px + 1, labelY - 1, LOUPE_W - 2, 1);

    // 颜色色块
    const swX = px + 12;
    const swY = labelY + 6;
    const textX = swX + 26;
    const textMaxW = LOUPE_W - textX + px - 12;
    ctx.fillStyle = currentHex;
    ctx.beginPath();
    ctx.roundRect(swX, swY, 18, 18, 4);
    ctx.fill();
    ctx.strokeStyle = "rgba(255,255,255,0.3)";
    ctx.lineWidth = 1;
    ctx.stroke();

    // HEX 文字
    ctx.fillStyle = "#fff";
    ctx.font = "600 12px ui-monospace, Menlo, Consolas, monospace";
    ctx.textBaseline = "middle";
    ctx.fillText(currentHex, textX, swY + 9, textMaxW);

    // RGB 文字
    ctx.fillStyle = "rgba(255,255,255,0.45)";
    ctx.font = "9.5px ui-monospace, Menlo, Consolas, monospace";
    ctx.textBaseline = "top";
    ctx.fillText(
      `rgb(${currentRgb.r}, ${currentRgb.g}, ${currentRgb.b})`,
      swX,
      swY + 24,
      LOUPE_W - 24,
    );
  }

  function scheduleRedraw() {
    if (rafId) return;
    rafId = requestAnimationFrame(() => {
      rafId = 0;
      draw();
    });
  }

  // ── 事件处理 ────────────────────────────────────────────────────────────────

  function onPointerMove(e: PointerEvent) {
    mouseX = e.clientX;
    mouseY = e.clientY;
    scheduleRedraw();
  }

  function onPointerDown(e: PointerEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (e.button === 0) {
      finish(currentHex);
    } else {
      finish(null);
    }
  }

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    finish(null);
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      finish(null);
    }
  }

  // ── 结束取色 ────────────────────────────────────────────────────────────────

  function finish(hex: string | null) {
    if (done) return;
    done = true;
    loadId += 1;

    if (rafId) {
      cancelAnimationFrame(rafId);
      rafId = 0;
    }

    // Fire-and-forget：不等 IPC 返回，避免卡死
    // Rust 端收到后会 spawn 异步任务隐藏窗口 + 恢复主窗口
    invoke("finish_color_picker", { hex }).catch(() => {});

    // 兜底：如果 Rust 端没能隐藏窗口（IPC 失败等），500ms 后自己隐藏
    setTimeout(() => {
      getCurrentWindow()
        .hide()
        .catch(() => {})
        .finally(releaseCaptureResources);
    }, 500);
  }

  function releaseCaptureResources() {
    bgBitmap?.close();
    bgBitmap = null;
    bgFrame = null;
    imageData = null;
    capture = null;
  }

  function toClampedPixels(buffer: ArrayBuffer | Uint8Array) {
    if (buffer instanceof ArrayBuffer) {
      return new Uint8ClampedArray(buffer);
    }
    return new Uint8ClampedArray(
      buffer.buffer,
      buffer.byteOffset,
      buffer.byteLength,
    );
  }

  async function init() {
    if (loading) return;
    loading = true;
    done = false;
    const currentLoadId = ++loadId;
    releaseCaptureResources();

    try {
      capture = await invoke<ColorPickerCapture>("get_color_picker_capture");
    } catch {
      loading = false;
      return;
    }

    if (currentLoadId !== loadId || !capture) {
      loading = false;
      return;
    }

    canvas.width = Math.round(capture.logicalWidth);
    canvas.height = Math.round(capture.logicalHeight);
    ctx = canvas.getContext("2d", { willReadFrequently: true });

    if (!ctx) {
      loading = false;
      console.error("[color-picker-overlay] canvas context unavailable");
      finish(null);
      return;
    }

    try {
      const buffer = await invoke<ArrayBuffer | Uint8Array>(
        "get_color_picker_image",
      );
      const pixels = toClampedPixels(buffer);
      const expectedLength = capture.width * capture.height * 4;
      if (pixels.byteLength !== expectedLength || pixels.byteLength % 4 !== 0) {
        throw new Error(
          `Invalid color picker image buffer: got ${pixels.byteLength}, expected ${expectedLength}`,
        );
      }
      if (currentLoadId !== loadId) {
        releaseCaptureResources();
        loading = false;
        return;
      }

      imageData = new ImageData(pixels, capture.width, capture.height);
      try {
        bgBitmap = await createImageBitmap(imageData);
      } catch {
        bgBitmap = null;
      }

      if (!bgBitmap && capture.scaleFactor === 1) {
        bgFrame = imageData;
      } else if (!bgBitmap) {
        const source = document.createElement("canvas");
        source.width = capture.width;
        source.height = capture.height;
        const sourceCtx = source.getContext("2d");
        if (!sourceCtx) {
          loading = false;
          console.error(
            "[color-picker-overlay] source canvas context unavailable",
          );
          finish(null);
          return;
        }
        sourceCtx.putImageData(imageData, 0, 0);

        const offscreen = document.createElement("canvas");
        offscreen.width = canvas.width;
        offscreen.height = canvas.height;
        const offCtx = offscreen.getContext("2d", { willReadFrequently: true });
        if (!offCtx) {
          loading = false;
          console.error(
            "[color-picker-overlay] scaled canvas context unavailable",
          );
          finish(null);
          return;
        }
        offCtx.drawImage(source, 0, 0, canvas.width, canvas.height);
        bgFrame = offCtx.getImageData(0, 0, canvas.width, canvas.height);
      }

      if (currentLoadId !== loadId) {
        releaseCaptureResources();
        loading = false;
        return;
      }

      mouseX = canvas.width / 2;
      mouseY = canvas.height / 2;
      draw();

      const currentWindow = getCurrentWindow();
      await currentWindow.show();
      currentWindow.setFocus().catch(() => {});
      loading = false;
    } catch (e) {
      console.error("加载截图像素失败", e);
      loading = false;
      finish(null);
    }
  }

  onMount(() => {
    let unlistenCaptureReady: UnlistenFn | undefined;

    listen("color_picker_capture_ready", () => {
      init();
    }).then((fn) => {
      unlistenCaptureReady = fn;
    });

    init();

    return () => {
      unlistenCaptureReady?.();
      releaseCaptureResources();
    };
  });
</script>

<svelte:window onkeydown={onKeyDown} oncontextmenu={onContextMenu} />

<canvas
  bind:this={canvas}
  class="overlay-canvas"
  onpointermove={onPointerMove}
  onpointerdown={onPointerDown}
></canvas>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: #000;
    cursor: crosshair;
    user-select: none;
  }

  .overlay-canvas {
    position: fixed;
    inset: 0;
    display: block;
    width: 100vw;
    height: 100vh;
    cursor: crosshair;
    touch-action: none;
    background: #000;
  }
</style>
