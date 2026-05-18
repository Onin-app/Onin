<script lang="ts">
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  interface ColorPickerCapture {
    width: number;
    height: number;
    logicalWidth: number;
    logicalHeight: number;
    scaleFactor: number;
  }

  interface ColorPickerKeyboardEvent {
    targetLabel: string;
    key: string;
    shiftKey: boolean;
  }

  const GRID_RADIUS = 7;
  const CELL = 9;
  const GRID_PX = (GRID_RADIUS * 2 + 1) * CELL;
  const LOUPE_W = GRID_PX + 20;
  const LABEL_H = 48;
  const LOUPE_H = GRID_PX + LABEL_H + 16;
  const OFFSET = 22;
  const MARGIN = 16;
  const KEYBOARD_STEP = 1;
  const KEYBOARD_FAST_STEP = 10;

  let sf = 1;
  let cellPx = CELL;
  let gridPx = GRID_PX;
  let loupeWPx = LOUPE_W;
  let loupeHPx = LOUPE_H;
  let offsetPx = OFFSET;
  let marginPx = MARGIN;
  let labelHPx = LABEL_H;

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;

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

  function samplePixel(px: number, py: number) {
    if (!imageData || !capture) return { r: 0, g: 0, b: 0, hex: "#000000" };

    const sx = Math.max(0, Math.min(capture.width - 1, Math.round(px)));
    const sy = Math.max(0, Math.min(capture.height - 1, Math.round(py)));
    const off = (sy * capture.width + sx) * 4;
    const r = imageData.data[off];
    const g = imageData.data[off + 1];
    const b = imageData.data[off + 2];
    const hex =
      `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`.toUpperCase();
    return { r, g, b, hex };
  }

  function draw() {
    if (!ctx || !canvas || !imageData || !capture) return;

    ctx.putImageData(imageData, 0, 0);

    const color = samplePixel(mouseX, mouseY);
    currentHex = color.hex;
    currentRgb = { r: color.r, g: color.g, b: color.b };

    const x = Math.round(mouseX);
    const y = Math.round(mouseY);

    ctx.save();
    ctx.strokeStyle = "rgba(255,255,255,0.85)";
    ctx.lineWidth = 1 * sf;
    ctx.setLineDash([]);
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, canvas.height);
    ctx.moveTo(0, y);
    ctx.lineTo(canvas.width, y);
    ctx.stroke();
    ctx.strokeStyle = "rgba(0,0,0,0.65)";
    ctx.setLineDash([4 * sf, 4 * sf]);
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, canvas.height);
    ctx.moveTo(0, y);
    ctx.lineTo(canvas.width, y);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.restore();

    let px = x + offsetPx;
    let py = y + offsetPx;
    if (px + loupeWPx > canvas.width - marginPx) px = x - loupeWPx - offsetPx;
    if (py + loupeHPx > canvas.height - marginPx) py = y - loupeHPx - offsetPx;
    px = Math.max(marginPx, Math.min(canvas.width - loupeWPx - marginPx, px));
    py = Math.max(marginPx, Math.min(canvas.height - loupeHPx - marginPx, py));

    drawLoupe(px, py);
  }

  function drawLoupe(px: number, py: number) {
    if (!ctx || !imageData || !capture) return;

    const r = 12 * sf;

    ctx.save();
    ctx.beginPath();
    ctx.roundRect(px, py, loupeWPx, loupeHPx, r);
    ctx.fillStyle = "rgba(18, 18, 20, 0.92)";
    ctx.fill();
    ctx.strokeStyle = "rgba(255,255,255,0.15)";
    ctx.lineWidth = 1 * sf;
    ctx.stroke();
    ctx.restore();

    const gridLeft = px + Math.floor((loupeWPx - gridPx) / 2);
    const gridTop = py + 8 * sf;
    const originPx = Math.round(mouseX) - GRID_RADIUS;
    const originPy = Math.round(mouseY) - GRID_RADIUS;
    const gridSize = GRID_RADIUS * 2 + 1;

    for (let row = 0; row < gridSize; row++) {
      for (let col = 0; col < gridSize; col++) {
        const sx = Math.max(0, Math.min(capture.width - 1, originPx + col));
        const sy = Math.max(0, Math.min(capture.height - 1, originPy + row));
        const off = (sy * capture.width + sx) * 4;
        ctx.fillStyle = `rgb(${imageData.data[off]},${imageData.data[off + 1]},${imageData.data[off + 2]})`;
        ctx.fillRect(
          gridLeft + col * cellPx,
          gridTop + row * cellPx,
          cellPx,
          cellPx,
        );
      }
    }

    const cx = gridLeft + GRID_RADIUS * cellPx;
    const cy = gridTop + GRID_RADIUS * cellPx;
    ctx.strokeStyle = "#fff";
    ctx.lineWidth = 2 * sf;
    ctx.strokeRect(cx - 1 * sf, cy - 1 * sf, cellPx + 2 * sf, cellPx + 2 * sf);
    ctx.strokeStyle = "rgba(0,0,0,0.5)";
    ctx.lineWidth = 1 * sf;
    ctx.strokeRect(cx, cy, cellPx, cellPx);

    const labelY = gridTop + gridPx + 8 * sf;
    ctx.fillStyle = "rgba(255,255,255,0.08)";
    ctx.fillRect(px + 1 * sf, labelY - 1 * sf, loupeWPx - 2 * sf, 1);

    const swX = px + 12 * sf;
    const swY = labelY + 6 * sf;
    const textX = swX + 26 * sf;
    const swSize = 18 * sf;
    ctx.fillStyle = currentHex;
    ctx.beginPath();
    ctx.roundRect(swX, swY, swSize, swSize, 4 * sf);
    ctx.fill();
    ctx.strokeStyle = "rgba(255,255,255,0.3)";
    ctx.lineWidth = 1 * sf;
    ctx.stroke();

    ctx.fillStyle = "#fff";
    ctx.font = `600 ${12 * sf}px ui-monospace, Menlo, Consolas, monospace`;
    ctx.textBaseline = "middle";
    const textMaxW = loupeWPx - 24 * sf;
    ctx.fillText(currentHex, textX, swY + swSize / 2, textMaxW);

    ctx.fillStyle = "rgba(255,255,255,0.45)";
    ctx.font = `${9.5 * sf}px ui-monospace, Menlo, Consolas, monospace`;
    ctx.textBaseline = "top";
    ctx.fillText(
      `rgb(${currentRgb.r}, ${currentRgb.g}, ${currentRgb.b})`,
      swX,
      swY + swSize + 2 * sf,
      loupeWPx - 24 * sf,
    );
  }

  function scheduleRedraw() {
    if (rafId) return;
    rafId = requestAnimationFrame(() => {
      rafId = 0;
      draw();
    });
  }

  function cssToPhysical(clientX: number, clientY: number) {
    const rect = canvas.getBoundingClientRect();
    const scaleX = capture ? capture.width / rect.width : 1;
    const scaleY = capture ? capture.height / rect.height : 1;
    return { x: clientX * scaleX, y: clientY * scaleY };
  }

  function onPointerMove(e: PointerEvent) {
    const pos = cssToPhysical(e.clientX, e.clientY);
    mouseX = pos.x;
    mouseY = pos.y;
    scheduleRedraw();
  }

  function moveCursor(dx: number, dy: number) {
    if (!canvas) return;
    mouseX = Math.max(0, Math.min(canvas.width - 1, mouseX + dx));
    mouseY = Math.max(0, Math.min(canvas.height - 1, mouseY + dy));
    scheduleRedraw();
  }

  function onPointerDown(e: PointerEvent) {
    e.preventDefault();
    e.stopPropagation();
    canvas?.focus();

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

  function handleKeyboardAction(key: string, shiftKey: boolean) {
    if (key === "Escape") {
      finish(null);
      return;
    }

    if (key === "Enter") {
      finish(samplePixel(mouseX, mouseY).hex);
      return;
    }

    const step = shiftKey ? KEYBOARD_FAST_STEP : KEYBOARD_STEP;
    switch (key) {
      case "ArrowUp":
        moveCursor(0, -step);
        break;
      case "ArrowDown":
        moveCursor(0, step);
        break;
      case "ArrowLeft":
        moveCursor(-step, 0);
        break;
      case "ArrowRight":
        moveCursor(step, 0);
        break;
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key !== "Escape" && e.key !== "Enter" && !e.key.startsWith("Arrow"))
      return;
    e.preventDefault();
    handleKeyboardAction(e.key, e.shiftKey);
  }

  function finish(hex: string | null) {
    if (done) return;
    done = true;
    loadId += 1;

    if (rafId) {
      cancelAnimationFrame(rafId);
      rafId = 0;
    }

    invoke("finish_color_picker", { hex }).catch(() => {});

    setTimeout(() => {
      getCurrentWindow()
        .hide()
        .catch(() => {})
        .finally(releaseCaptureResources);
    }, 500);
  }

  function releaseCaptureResources() {
    imageData = null;
    capture = null;
  }

  function toClampedPixels(buffer: any): Uint8ClampedArray {
    if (!buffer) {
      return new Uint8ClampedArray(0);
    }
    if (buffer instanceof Uint8ClampedArray) {
      return buffer;
    }
    if (buffer instanceof ArrayBuffer) {
      return new Uint8ClampedArray(buffer);
    }
    if (buffer.buffer && buffer.buffer instanceof ArrayBuffer) {
      return new Uint8ClampedArray(
        buffer.buffer,
        buffer.byteOffset ?? 0,
        buffer.byteLength ?? 0,
      );
    }
    if (
      Array.isArray(buffer) ||
      (typeof buffer === "object" && typeof buffer.length === "number")
    ) {
      return new Uint8ClampedArray(buffer);
    }
    if (buffer.data) {
      return toClampedPixels(buffer.data);
    }
    return new Uint8ClampedArray(buffer);
  }

  async function init() {
    if (loading) return;
    loading = true;
    done = false;
    const currentLoadId = ++loadId;
    const label = getCurrentWindow().label;
    releaseCaptureResources();

    try {
      capture = await invoke<ColorPickerCapture>("get_color_picker_capture", {
        label,
      });
    } catch {
      loading = false;
      return;
    }

    if (currentLoadId !== loadId || !capture) {
      loading = false;
      return;
    }

    canvas.width = capture.width;
    canvas.height = capture.height;
    ctx = canvas.getContext("2d", { willReadFrequently: true });

    if (!ctx) {
      loading = false;
      finish(null);
      return;
    }

    sf = capture.scaleFactor;
    cellPx = Math.round(CELL * sf);
    gridPx = Math.round(GRID_PX * sf);
    loupeWPx = Math.round(LOUPE_W * sf);
    loupeHPx = Math.round(LOUPE_H * sf);
    offsetPx = Math.round(OFFSET * sf);
    marginPx = Math.round(MARGIN * sf);
    labelHPx = Math.round(LABEL_H * sf);

    try {
      const buffer = await invoke<ArrayBuffer | Uint8Array>(
        "get_color_picker_image",
        { label },
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
      await currentWindow.setFocus().catch(() => {});
      await invoke("focus_color_picker_overlay", { label }).catch(() => {});
      await tick();
      canvas?.focus();
      loading = false;
    } catch (e) {
      console.error("加载截图像素失败", e);
      loading = false;
      finish(null);
    }
  }

  onMount(() => {
    let unlistenCaptureReady: UnlistenFn | undefined;
    let unlistenKeyboard: UnlistenFn | undefined;

    listen("color_picker_capture_ready", () => {
      init();
    }).then((fn) => {
      unlistenCaptureReady = fn;
    });

    listen<ColorPickerKeyboardEvent>("color_picker_keyboard", (event) => {
      if (done || event.payload.targetLabel !== getCurrentWindow().label)
        return;
      handleKeyboardAction(event.payload.key, event.payload.shiftKey);
    }).then((fn) => {
      unlistenKeyboard = fn;
    });

    init();

    return () => {
      unlistenCaptureReady?.();
      unlistenKeyboard?.();
      releaseCaptureResources();
    };
  });
</script>

<svelte:window onkeydown={onKeyDown} oncontextmenu={onContextMenu} />

<canvas
  bind:this={canvas}
  class="overlay-canvas"
  tabindex="0"
  onpointermove={onPointerMove}
  onpointerdown={onPointerDown}
  onkeydown={onKeyDown}
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
    image-rendering: crisp-edges;
  }
</style>
