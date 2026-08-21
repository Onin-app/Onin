<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    getCurrentWebviewWindow,
    getAllWebviewWindows,
  } from "@tauri-apps/api/webviewWindow";
  import { Play, X } from "phosphor-svelte";
  import "../../index.css";

  // 屏幕尺寸及缩放比 (逻辑像素)
  let screenWidth = $state(800);
  let screenHeight = $state(600);
  let scaleFactor = $state(1);

  // 绑定的 DOM 引用以进行原生操作
  let boxEl: HTMLDivElement | null = $state(null);
  let maskTopEl: HTMLDivElement | null = $state(null);
  let maskBottomEl: HTMLDivElement | null = $state(null);
  let maskLeftEl: HTMLDivElement | null = $state(null);
  let maskRightEl: HTMLDivElement | null = $state(null);
  let toolbarEl: HTMLDivElement | null = $state(null);
  let sizeTextEl: HTMLSpanElement | null = $state(null);

  // 当前真实的选区数据，仅在 mouseup 或是 mount 时回流到 Svelte state
  let x = $state(100);
  let y = $state(100);
  let w = $state(600);
  let h = $state(400);

  // 拖动时的物理像素缓存变量，供 mousemove 高频修改
  let currentX = 100;
  let currentY = 100;
  let currentW = 600;
  let currentH = 400;

  // 拖动状态
  let isDragging = false;
  let dragType = ""; // "move" | "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w"
  let startMouseX = 0;
  let startMouseY = 0;
  let startRect = { x: 0, y: 0, w: 0, h: 0 };

  // 全局配置缓存，用于拉起录屏时拼装完整配置
  let currentConfig: any = null;
  let isStarting = $state(false);
  let initialized = $state(false);

  // 通过直接操纵 style 的方式高频更新 DOM，消除 Svelte 细粒度状态的重绘损耗，保障拖拽 60FPS 流畅度
  function updateDOM(cx: number, cy: number, cw: number, ch: number) {
    if (
      !boxEl ||
      !maskTopEl ||
      !maskBottomEl ||
      !maskLeftEl ||
      !maskRightEl ||
      !toolbarEl ||
      !sizeTextEl
    )
      return;

    // 1. 更新虚线选区框
    boxEl.style.left = `${cx}px`;
    boxEl.style.top = `${cy}px`;
    boxEl.style.width = `${cw}px`;
    boxEl.style.height = `${ch}px`;

    // 2. 更新四个背景遮罩块
    maskTopEl.style.height = `${cy}px`;

    maskBottomEl.style.top = `${cy + ch}px`;
    maskBottomEl.style.height = `${screenHeight - (cy + ch)}px`;

    maskLeftEl.style.top = `${cy}px`;
    maskLeftEl.style.width = `${cx}px`;
    maskLeftEl.style.height = `${ch}px`;

    maskRightEl.style.top = `${cy}px`;
    maskRightEl.style.left = `${cx + cw}px`;
    maskRightEl.style.width = `${screenWidth - (cx + cw)}px`;
    maskRightEl.style.height = `${ch}px`;

    // 3. 计算并更新控制条位置
    let ty = 0;
    const spaceBelow = screenHeight - (cy + ch);
    if (spaceBelow >= 60) {
      ty = cy + ch + 10;
    } else if (cy >= 60) {
      ty = cy - 50;
    } else {
      ty = cy + ch - 50;
    }
    let tx = cx + (cw - 280) / 2;
    tx = Math.max(10, Math.min(tx, screenWidth - 290));

    toolbarEl.style.left = `${tx}px`;
    toolbarEl.style.top = `${ty}px`;

    // 4. 更新物理分辨率指示文本
    const physicalW = Math.round(cw * scaleFactor) & ~1;
    const physicalH = Math.round(ch * scaleFactor) & ~1;
    sizeTextEl.innerText = `${physicalW} × ${physicalH} Px`;
  }

  onMount(async () => {
    // 强制清除全局 DOM 的背景色污染，保证 Webview 透明背景能够被穿透并镂空显示桌面内容
    document.documentElement.style.backgroundColor = "transparent";
    document.body.style.backgroundColor = "transparent";

    const win = getCurrentWebviewWindow();

    // 从 Rust 获取当前的录屏配置和可用屏幕列表
    try {
      currentConfig = await invoke("get_recorder_config");
      const monitorIndex = currentConfig?.monitorIndex ?? 0;

      const monitors = await invoke<any[]>("get_available_monitors");
      const monitor = monitors[monitorIndex] || monitors[0];
      if (monitor) {
        scaleFactor = monitor.scaleFactor || 1;
        screenWidth = monitor.width / scaleFactor;
        screenHeight = monitor.height / scaleFactor;
      }
    } catch (e) {
      console.error("Failed to load global config / monitor list:", e);
    }

    // 设定默认选区：居中，大小为屏幕高宽的 60%
    w = Math.max(200, Math.round(screenWidth * 0.6) & ~1);
    h = Math.max(150, Math.round(screenHeight * 0.6) & ~1);
    x = Math.round((screenWidth - w) / 2);
    y = Math.round((screenHeight - h) / 2);

    // 如果配置中已经有了之前的选区，我们可以恢复它
    if (currentConfig && currentConfig.areaRect) {
      x = currentConfig.areaRect.x;
      y = currentConfig.areaRect.y;
      w = currentConfig.areaRect.width;
      h = currentConfig.areaRect.height;
    }

    currentX = x;
    currentY = y;
    currentW = w;
    currentH = h;

    // 初始渲染一次 DOM 坐标和高宽
    updateDOM(x, y, w, h);
    initialized = true;

    window.addEventListener("keydown", handleKeyDown);
  });

  onDestroy(() => {
    // 恢复全局 DOM 背景颜色
    document.documentElement.style.backgroundColor = "";
    document.body.style.backgroundColor = "";

    if (typeof window !== "undefined") {
      window.removeEventListener("keydown", handleKeyDown);
    }
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      cancelSelection();
    }
  }

  // 鼠标按下选区或手柄
  function handleMouseDown(e: MouseEvent, type: string) {
    e.preventDefault();
    e.stopPropagation();
    isDragging = true;
    dragType = type;
    startMouseX = e.clientX;
    startMouseY = e.clientY;
    startRect = { x: currentX, y: currentY, w: currentW, h: currentH };
  }

  // 鼠标移动更新位置和尺寸 (直改 style，避开 svelte 细粒度 rune 频繁调度)
  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;

    const deltaX = e.clientX - startMouseX;
    const deltaY = e.clientY - startMouseY;

    let newX = startRect.x;
    let newY = startRect.y;
    let newW = startRect.w;
    let newH = startRect.h;

    const minW = 120;
    const minH = 120;

    if (dragType === "move") {
      newX = startRect.x + deltaX;
      newY = startRect.y + deltaY;

      // 限制拖动不能超出屏幕
      newX = Math.max(0, Math.min(newX, screenWidth - newW));
      newY = Math.max(0, Math.min(newY, screenHeight - newH));
    } else {
      if (dragType.includes("w")) {
        const requestedW = startRect.w - deltaX;
        if (requestedW >= minW) {
          newX = startRect.x + deltaX;
          newW = requestedW;
        } else {
          newX = startRect.x + (startRect.w - minW);
          newW = minW;
        }
      }
      if (dragType.includes("e")) {
        newW = Math.max(minW, startRect.w + deltaX);
      }
      if (dragType.includes("n")) {
        const requestedH = startRect.h - deltaY;
        if (requestedH >= minH) {
          newY = startRect.y + deltaY;
          newH = requestedH;
        } else {
          newY = startRect.y + (startRect.h - minH);
          newH = minH;
        }
      }
      if (dragType.includes("s")) {
        newH = Math.max(minH, startRect.h + deltaY);
      }

      // 边界限制
      if (newX < 0) {
        newW += newX;
        newX = 0;
      }
      if (newY < 0) {
        newH += newY;
        newY = 0;
      }
      if (newX + newW > screenWidth) {
        newW = screenWidth - newX;
      }
      if (newY + newH > screenHeight) {
        newH = screenHeight - newY;
      }
    }

    currentX = Math.round(newX);
    currentY = Math.round(newY);
    currentW = Math.round(newW);
    currentH = Math.round(newH);

    updateDOM(currentX, currentY, currentW, currentH);
  }

  function handleMouseUp() {
    if (isDragging) {
      isDragging = false;
      // 拖拽停止时才将数据写回 Svelte state，供确认时提交
      x = currentX;
      y = currentY;
      w = currentW;
      h = currentH;
    }
  }

  // 取消并关闭
  async function cancelSelection() {
    // 恢复主配置窗口
    try {
      const windows = await getAllWebviewWindows();
      const mainWin = windows.find((win) => win.label === "main");
      if (mainWin) {
        await mainWin.show();
        await mainWin.setFocus().catch(() => {});
      }
    } catch (e) {
      console.error("恢复主窗口失败:", e);
    }

    // 关闭当前选区窗口
    try {
      const current = getCurrentWebviewWindow();
      await current.close();
    } catch (e) {
      console.error("关闭当前窗口失败:", e);
    }
  }

  // 确认选区并直接开始录像
  async function startRecording() {
    if (isStarting) return;
    isStarting = true;

    try {
      const area = {
        x: currentX,
        y: currentY,
        width: currentW,
        height: currentH,
      };
      const config = {
        ...(currentConfig || {}),
        recordTargetType: "area",
        areaRect: area,
      };

      // 1. 先保存配置到 Rust
      await invoke("save_recorder_config", { config });

      // 2. 将配置同步至 localstorage 解决持久化
      localStorage.setItem(
        "onin_screen_recorder_config",
        JSON.stringify(config),
      );

      // 3. 拉起控制计时小浮条 Bar
      await invoke("show_screen_recorder_bar");

      // 4. 关掉自身
      const current = getCurrentWebviewWindow();
      await current.close();
    } catch (e) {
      alert("启动录制失败: " + e);
      isStarting = false;
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="relative h-screen w-screen overflow-hidden bg-transparent transition-opacity duration-150 select-none {initialized
    ? 'opacity-100'
    : 'opacity-0'}"
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
>
  <!-- 四个半透明黑色遮罩块，用来镂空中间选区，纯 CSS 定位自适应视口 -->
  <!-- 上遮罩 -->
  <div
    bind:this={maskTopEl}
    class="absolute top-0 right-0 left-0 bg-black/55"
    style="height: {y}px;"
  ></div>
  <!-- 下遮罩 -->
  <div
    bind:this={maskBottomEl}
    class="absolute right-0 bottom-0 left-0 bg-black/55"
    style="top: {y + h}px;"
  ></div>
  <!-- 左遮罩 -->
  <div
    bind:this={maskLeftEl}
    class="absolute left-0 bg-black/55"
    style="top: {y}px; width: {x}px; height: {h}px;"
  ></div>
  <!-- 右遮罩 -->
  <div
    bind:this={maskRightEl}
    class="absolute right-0 bg-black/55"
    style="top: {y}px; left: {x + w}px; height: {h}px;"
  ></div>

  <!-- 中间镂空的虚线选区框 -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={boxEl}
    class="absolute cursor-move border-2 border-dashed border-red-500"
    style="left: {x}px; top: {y}px; width: {w}px; height: {h}px;"
    onmousedown={(e) => handleMouseDown(e, "move")}
  >
    <!-- 8 个方向的调整大小手柄 -->
    <!-- 角点 -->
    <div
      class="absolute -top-1.5 -left-1.5 size-3.5 cursor-nwse-resize rounded-full border-2 border-red-500 bg-white shadow-md transition-transform hover:scale-125"
      onmousedown={(e) => handleMouseDown(e, "nw")}
    ></div>
    <div
      class="absolute -top-1.5 -right-1.5 size-3.5 cursor-nesw-resize rounded-full border-2 border-red-500 bg-white shadow-md transition-transform hover:scale-125"
      onmousedown={(e) => handleMouseDown(e, "ne")}
    ></div>
    <div
      class="absolute -bottom-1.5 -left-1.5 size-3.5 cursor-nesw-resize rounded-full border-2 border-red-500 bg-white shadow-md transition-transform hover:scale-125"
      onmousedown={(e) => handleMouseDown(e, "sw")}
    ></div>
    <div
      class="absolute -right-1.5 -bottom-1.5 size-3.5 cursor-nwse-resize rounded-full border-2 border-red-500 bg-white shadow-md transition-transform hover:scale-125"
      onmousedown={(e) => handleMouseDown(e, "se")}
    ></div>

    <!-- 边线中点 -->
    <div
      class="absolute -top-1.5 left-1/2 -ml-1.5 size-3 cursor-ns-resize rounded-full border-2 border-red-500 bg-white shadow-md transition-transform hover:scale-125"
      onmousedown={(e) => handleMouseDown(e, "n")}
    ></div>
    <div
      class="absolute -bottom-1.5 left-1/2 -ml-1.5 size-3 cursor-ns-resize rounded-full border-2 border-red-500 bg-white shadow-md transition-transform hover:scale-125"
      onmousedown={(e) => handleMouseDown(e, "s")}
    ></div>
    <div
      class="absolute top-1/2 -left-1.5 -mt-1.5 size-3 cursor-ew-resize rounded-full border-2 border-red-500 bg-white shadow-md transition-transform hover:scale-125"
      onmousedown={(e) => handleMouseDown(e, "w")}
    ></div>
    <div
      class="absolute top-1/2 -right-1.5 -mt-1.5 size-3 cursor-ew-resize rounded-full border-2 border-red-500 bg-white shadow-md transition-transform hover:scale-125"
      onmousedown={(e) => handleMouseDown(e, "e")}
    ></div>
  </div>

  <!-- 跟随选区的悬浮控制面板 -->
  <div
    bind:this={toolbarEl}
    class="absolute flex h-10 w-[280px] items-center justify-between rounded-xl border border-neutral-800 bg-neutral-950/90 px-3 text-white shadow-2xl backdrop-blur-md"
    style="left: {x + (w - 280) / 2}px; top: {y + h + 10}px;"
  >
    <!-- 像素大小展示 -->
    <span
      bind:this={sizeTextEl}
      class="font-mono text-xs font-bold text-neutral-300"
    >
      0 × 0 Px
    </span>

    <div class="flex items-center gap-2">
      <!-- 取消按钮 -->
      <button
        class="flex cursor-pointer items-center gap-1 rounded-lg bg-neutral-800 px-2.5 py-1 text-xs font-semibold text-neutral-300 transition-all hover:bg-neutral-700 hover:text-white"
        onclick={cancelSelection}
      >
        <X class="size-3" />
        <span>取消</span>
      </button>

      <!-- 开始录制按钮 -->
      <button
        class="flex cursor-pointer items-center gap-1 rounded-lg bg-red-600 px-3 py-1 text-xs font-semibold text-white shadow-md transition-all hover:bg-red-500 active:scale-95 disabled:opacity-50"
        onclick={startRecording}
        disabled={isStarting}
      >
        <Play class="size-3 fill-white" />
        <span>{isStarting ? "启动中..." : "开始录像"}</span>
      </button>
    </div>
  </div>
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent !important;
  }
</style>
