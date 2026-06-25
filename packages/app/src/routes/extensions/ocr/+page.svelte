<script lang="ts">
  /**
   * Onin 内置 OCR (文字识别) 扩展
   */
  import { onMount, onDestroy, tick } from "svelte";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";
  import {
    Copy,
    Trash,
    Spinner,
    FileImage,
    ClipboardText,
    MagnifyingGlass,
    TextIndent,
  } from "phosphor-svelte";
  import { Tabs, Button } from "bits-ui";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";
  import type { AppConfig } from "$lib/type";

  // 定义类型
  interface OcrWord {
    text: string;
    x: number;
    y: number;
    width: number;
    height: number;
  }

  interface OcrLine {
    text: string;
    x: number;
    y: number;
    width: number;
    height: number;
    words: OcrWord[];
  }

  interface OcrResult {
    text: string;
    lines: OcrLine[];
  }

  // 状态变量 (Svelte 5 语法)
  let searchQuery = $state("");
  let imageSrc = $state<string | null>(null);
  let isProcessing = $state(false);
  let ocrResult = $state<OcrResult | null>(null);
  let lastClipboardImage = $state<string | null>(null);
  let ocrEngine = $state<"local" | "ai">("local");

  let displayImageSrc = $derived(
    imageSrc
      ? imageSrc.startsWith("data:")
        ? imageSrc
        : convertFileSrc(imageSrc)
      : null,
  );

  // 图片缩放和展示尺寸
  let zoom = $state(1.0);
  let naturalWidth = $state(0);
  let naturalHeight = $state(0);
  let baseDisplayWidth = $state(0);
  let baseDisplayHeight = $state(0);

  // 平移与拖拽状态
  let translateX = $state(0);
  let translateY = $state(0);
  let isMouseDown = $state(false);
  let hasMoved = $state(false);

  let startX = 0;
  let startY = 0;
  let startTranslateX = 0;
  let startTranslateY = 0;

  let wheelContainer = $state<HTMLDivElement | null>(null);

  // Tab 状态
  let activeTab = $state<"merged" | "lines">("merged");

  // 对合并文本的响应式编辑状态
  let editableText = $state("");

  // 点击选中的行索引，实现图片和逐行列表的联动
  let selectedLineIndex = $state<number | null>(null);

  // 根据搜索过滤后的行数据
  let filteredLines = $derived(
    ocrResult
      ? ocrResult.lines.filter((line) =>
          line.text.toLowerCase().includes(searchQuery.toLowerCase()),
        )
      : [],
  );

  // OCR 识别执行
  async function recognizeImage(src: string) {
    isProcessing = true;
    ocrResult = null;
    editableText = "";
    selectedLineIndex = null;

    try {
      const result = await invoke<OcrResult>("plugin_ocr_recognize", {
        image: src,
        options: {
          engine: ocrEngine,
        },
      });

      ocrResult = result;
      editableText = result.text;
      toast.success("文字识别完成");
    } catch (error) {
      console.error("OCR Failed:", error);
      const errMsg = typeof error === "string" ? error : "识别失败，请重试";
      if (errMsg.includes("AI 管理器未激活")) {
        toast.error(
          "未启用 AI 模型。请前往系统设置 - 模型中配置并启用支持多模态（图片输入）的模型。",
        );
      } else if (
        errMsg.includes("支持图片") ||
        errMsg.includes("image") ||
        errMsg.includes("multimodal") ||
        errMsg.includes("400")
      ) {
        toast.error(
          "识别失败：当前启用的 AI 模型可能不支持图片识别。请前往“设置 - 模型”中配置并启用支持多模态（图片输入）的模型。",
        );
      } else {
        toast.error(errMsg);
      }
      imageSrc = null;
    } finally {
      isProcessing = false;
    }
  }

  // 切换引擎，如果已有图片则重新识别
  async function handleEngineChange(engine: "local" | "ai") {
    if (ocrEngine === engine) return;
    ocrEngine = engine;

    if (imageSrc) {
      await recognizeImage(imageSrc);
    }
  }

  // 辅助函数：根据比例缩放计算图片在容器里的显示大小
  function handleImageLoad(e: Event) {
    const img = e.target as HTMLImageElement;
    naturalWidth = img.naturalWidth;
    naturalHeight = img.naturalHeight;

    calculateDisplaySize();
  }

  function calculateDisplaySize() {
    if (naturalWidth === 0 || naturalHeight === 0) return;

    // 自适应容器大小，不再硬编码 480/360
    const containerW = wheelContainer?.clientWidth || 400;
    const containerH = wheelContainer?.clientHeight || 300;

    const maxW = containerW - 24;
    const maxH = containerH - 24;

    let scale = Math.min(maxW / naturalWidth, maxH / naturalHeight);
    if (scale > 1) {
      scale = 1;
    }

    baseDisplayWidth = naturalWidth * scale;
    baseDisplayHeight = naturalHeight * scale;

    zoom = 1.0;
    resetPosition();
  }

  function resetPosition() {
    if (
      wheelContainer &&
      naturalWidth > 0 &&
      baseDisplayWidth > 0 &&
      baseDisplayHeight > 0
    ) {
      const containerW = wheelContainer.clientWidth;
      const containerH = wheelContainer.clientHeight;
      translateX = (containerW - baseDisplayWidth) / 2;
      translateY = (containerH - baseDisplayHeight) / 2;
    } else {
      translateX = 0;
      translateY = 0;
    }
  }

  function zoomTo(nextZoom: number, centerX?: number, centerY?: number) {
    if (!wheelContainer) return;

    const minZoom = 0.1;
    const maxZoom = 10.0;
    nextZoom = Math.max(minZoom, Math.min(maxZoom, nextZoom));

    const rect = wheelContainer.getBoundingClientRect();
    const cx = centerX !== undefined ? centerX : rect.width / 2;
    const cy = centerY !== undefined ? centerY : rect.height / 2;

    const oldZoom = zoom;
    translateX = cx - (cx - translateX) * (nextZoom / oldZoom);
    translateY = cy - (cy - translateY) * (nextZoom / oldZoom);
    zoom = nextZoom;
  }

  // 滚轮缩放处理函数
  function handleWheel(e: WheelEvent) {
    if (!imageSrc || !wheelContainer) return;

    e.preventDefault();
    const zoomStep = 0.08;
    const rect = wheelContainer.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;

    const nextZoom =
      e.deltaY < 0 ? zoom + zoomStep * zoom : zoom - zoomStep * zoom;
    zoomTo(nextZoom, mx, my);
  }

  // 拖拽处理函数
  function handleMouseDown(e: MouseEvent) {
    if (!imageSrc) return;
    if (e.button !== 0) return; // 只响应左键

    const target = e.target as HTMLElement;
    if (target.closest(".no-pan") || target.closest("button")) {
      return;
    }

    isMouseDown = true;
    hasMoved = false;
    startX = e.clientX;
    startY = e.clientY;
    startTranslateX = translateX;
    startTranslateY = translateY;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isMouseDown) return;

    const dx = e.clientX - startX;
    const dy = e.clientY - startY;

    if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
      hasMoved = true;
    }

    translateX = startTranslateX + dx;
    translateY = startTranslateY + dy;
  }

  function handleMouseUp() {
    if (isMouseDown) {
      isMouseDown = false;
      setTimeout(() => {
        hasMoved = false;
      }, 0);
    }
  }

  function handleDoubleClick() {
    if (!imageSrc) return;
    zoom = 1.0;
    resetPosition();
  }

  // 滚动至并高亮选中的行
  async function highlightAndScrollToLine(lineIndex: number) {
    activeTab = "lines";
    selectedLineIndex = lineIndex;

    // 等待 Tab 切换和 DOM 渲染
    await tick();

    const element = document.getElementById(`line-item-${lineIndex}`);
    if (element) {
      element.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }

  // 从剪贴板读取并识别图片
  async function readClipboardImage(quiet = false) {
    try {
      const img = await invoke<string | null>("plugin_clipboard_read_image");
      if (img) {
        lastClipboardImage = img;
        imageSrc = img;
        await recognizeImage(img);
      } else if (!quiet) {
        toast.error("剪贴板中未检测到图片，请先复制一张图片");
      }
    } catch (error) {
      console.error("Read clipboard image failed:", error);
      if (!quiet) {
        toast.error("读取剪贴板图片失败");
      }
    }
  }

  // 选择本地文件进行识别
  async function selectLocalFile() {
    let tauriWindow: any = null;
    try {
      // 1. 锁定窗口，防止打开对话框时因为失去焦点导致 Onin 窗口自动隐藏
      await invoke("acquire_window_close_lock");

      // 2. 临时取消置顶层级，防止遮挡系统文件选择器
      if ((window as any).__TAURI_INTERNALS__) {
        try {
          const { getCurrentWindow } = await import("@tauri-apps/api/window");
          tauriWindow = getCurrentWindow();
          await tauriWindow.setAlwaysOnTop(false);
        } catch (winError) {
          console.error("Failed to temporarily disable alwaysOnTop:", winError);
        }
      }

      const selected = await invoke<string | null>("plugin_dialog_open", {
        options: {
          title: "选择识别图片",
          filters: [
            {
              name: "Images",
              extensions: ["png", "jpg", "jpeg", "webp", "bmp"],
            },
          ],
        },
      });

      if (selected) {
        imageSrc = selected;
        await recognizeImage(selected);
      }
    } catch (error) {
      console.error("Select file failed:", error);
      toast.error("打开文件选择器失败");
    } finally {
      // 3. 恢复置顶状态与释放窗口锁
      if (tauriWindow) {
        try {
          await tauriWindow.setAlwaysOnTop(true);
        } catch (e) {
          console.error("Failed to restore alwaysOnTop:", e);
        }
      }
      await invoke("release_window_close_lock");
    }
  }

  // 处理拖拽上传
  let isDragging = $state(false);

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  // 处理拖拽离开
  function handleDragLeave() {
    isDragging = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;

    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      if (file.type.startsWith("image/")) {
        const reader = new FileReader();
        reader.onload = async (event) => {
          const base64 = event.target?.result as string;
          imageSrc = base64;
          await recognizeImage(base64);
        };
        reader.readAsDataURL(file);
      } else {
        toast.error("仅支持拖入图片文件进行识别");
      }
    }
  }

  // 监听粘贴按键 (Ctrl+V / Cmd+V)
  async function handlePaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items;
    if (items) {
      for (let i = 0; i < items.length; i++) {
        if (items[i].type.indexOf("image") !== -1) {
          const file = items[i].getAsFile();
          if (file) {
            const reader = new FileReader();
            reader.onload = async (event) => {
              const base64 = event.target?.result as string;
              imageSrc = base64;
              await recognizeImage(base64);
            };
            reader.readAsDataURL(file);
            toast.info("已读取粘贴的图片数据");
            return;
          }
        }
      }
    }
    // 降级尝试调用底层剪贴板
    await readClipboardImage(true);
  }

  // 复制文字并进行气泡反馈
  async function copyText(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast.success("已复制到剪贴板");
    } catch (e) {
      console.error("Failed to copy:", e);
      toast.error("复制失败");
    }
  }

  // 整理换行和空白
  function cleanTextSpaces() {
    if (!editableText) return;
    editableText = editableText
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length > 0)
      .join("\n");
    toast.success("已完成文本空白整理");
  }

  // 清空数据
  function handleClear() {
    imageSrc = null;
    ocrResult = null;
    editableText = "";
    lastClipboardImage = null;
    zoom = 1.0;
    selectedLineIndex = null;
    translateX = 0;
    translateY = 0;
  }

  // 监听重新聚焦事件，自动读取剪贴板
  async function handleWindowFocus() {
    if (!isProcessing) {
      const img = await invoke<string | null>("plugin_clipboard_read_image");
      if (img && img !== lastClipboardImage) {
        lastClipboardImage = img;
        imageSrc = img;
        await recognizeImage(img);
      }
    }
  }

  // 返回主界面
  const handleBack = () => {
    goto("/");
  };

  onMount(async () => {
    // 监听聚焦，如果是自动切回来可以实现无缝识别
    window.addEventListener("focus", handleWindowFocus);
    window.addEventListener("resize", calculateDisplaySize);
    document.addEventListener("paste", handlePaste);

    // 读取默认引擎配置
    try {
      const config = await invoke<AppConfig>("get_app_config");
      if (
        config &&
        (config.ocr_default_engine === "local" ||
          config.ocr_default_engine === "ai")
      ) {
        ocrEngine = config.ocr_default_engine;
      }
    } catch (e) {
      console.error("Failed to load OCR default engine config:", e);
    }

    // 挂载时尝试自动识别一次当前剪贴板的图片
    readClipboardImage(true);
  });

  onDestroy(() => {
    if (typeof window !== "undefined") {
      window.removeEventListener("focus", handleWindowFocus);
      window.removeEventListener("resize", calculateDisplaySize);
    }
    if (typeof document !== "undefined") {
      document.removeEventListener("paste", handlePaste);
    }
  });

  // 使用 Svelte 5 的 $effect 动态且安全地在 wheelContainer 上绑定非 passive wheel 事件
  $effect(() => {
    if (wheelContainer) {
      const container = wheelContainer;
      const onWheel = (e: WheelEvent) => handleWheel(e);
      container.addEventListener("wheel", onWheel, { passive: false });
      return () => {
        container.removeEventListener("wheel", onWheel);
      };
    }
  });
</script>

<svelte:window onmousemove={handleMouseMove} onmouseup={handleMouseUp} />

{#snippet rightSnippet()}
  <div class="flex items-center gap-2">
    <div
      class="flex items-center gap-0.5 rounded-xl border border-neutral-200/50 bg-neutral-100/80 p-0.5 dark:border-neutral-700/50 dark:bg-neutral-800/80"
    >
      <button
        class="rounded-lg px-2.5 py-1 text-[11px] font-semibold transition-all {ocrEngine ===
        'local'
          ? 'bg-white text-neutral-900 shadow-xs dark:bg-neutral-700 dark:text-white'
          : 'text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white'}"
        onclick={() => handleEngineChange("local")}
      >
        本地 OCR
      </button>
      <button
        class="rounded-lg px-2.5 py-1 text-[11px] font-semibold transition-all {ocrEngine ===
        'ai'
          ? 'bg-white text-neutral-900 shadow-xs dark:bg-neutral-700 dark:text-white'
          : 'text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white'}"
        onclick={() => handleEngineChange("ai")}
      >
        AI OCR
      </button>
    </div>
  </div>
{/snippet}

<div class="flex h-full w-full flex-col p-3 select-none">
  <ExtensionHeader
    placeholder="在识别结果中过滤/搜索行..."
    bind:value={searchQuery}
    onBack={handleBack}
    right={rightSnippet}
  />

  <div class="mt-3 flex min-h-0 flex-1 flex-row gap-5 overflow-hidden">
    {#if !imageSrc}
      <div
        class="group relative flex flex-1 flex-col items-center justify-center overflow-hidden rounded-3xl border border-neutral-200/80 bg-white/40 p-8 shadow-sm backdrop-blur-md transition-all duration-500 hover:border-neutral-300 hover:shadow-xl hover:shadow-neutral-200/20 dark:border-neutral-800/80 dark:bg-neutral-900/40 dark:hover:border-neutral-700 dark:hover:shadow-neutral-950/40 {isDragging
          ? 'border-blue-500/80 bg-blue-500/5 ring-4 ring-blue-500/10 dark:border-blue-400/80 dark:bg-blue-400/5 dark:ring-blue-400/10'
          : ''}"
        role="button"
        tabindex="0"
        ondragover={handleDragOver}
        ondragleave={handleDragLeave}
        ondrop={handleDrop}
        onclick={selectLocalFile}
        onkeydown={(e) => e.key === "Enter" && selectLocalFile()}
      >
        <div
          class="absolute -top-40 -left-40 size-80 rounded-full bg-blue-400/10 blur-3xl transition-all duration-500 group-hover:bg-blue-400/15"
        ></div>
        <div
          class="absolute -right-40 -bottom-40 size-80 rounded-full bg-indigo-400/10 blur-3xl transition-all duration-500 group-hover:bg-indigo-400/15"
        ></div>

        <div class="relative flex max-w-md flex-col items-center">
          <div
            class="mb-6 flex size-20 items-center justify-center rounded-2xl bg-neutral-100 shadow-inner transition-transform duration-500 group-hover:scale-105 dark:bg-neutral-800"
          >
            <FileImage
              class="size-10 text-neutral-500 transition-transform duration-500 group-hover:-translate-y-0.5 dark:text-neutral-400"
            />
          </div>
          <h3
            class="mb-2 text-lg font-semibold tracking-tight text-neutral-800 dark:text-neutral-100"
          >
            拖入图片文件，或点击选择本地图片
          </h3>
          <p
            class="mb-8 text-sm leading-relaxed text-neutral-500 dark:text-neutral-400"
          >
            支持直接按下 <kbd
              class="rounded-lg border border-neutral-200 bg-neutral-50 px-2 py-1 font-mono text-xs text-neutral-600 shadow-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300"
              >Ctrl+V</kbd
            >
            /
            <kbd
              class="rounded-lg border border-neutral-200 bg-neutral-50 px-2 py-1 font-mono text-xs text-neutral-600 shadow-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300"
              >Cmd+V</kbd
            > 粘贴已复制的图片
          </p>

          <div class="flex gap-4">
            <Button.Root
              class="flex items-center gap-2 rounded-xl bg-neutral-900 px-5 py-2.5 text-sm font-semibold text-white shadow-md shadow-neutral-950/10 transition-all hover:bg-neutral-800 hover:shadow-lg hover:shadow-neutral-950/20 active:scale-95 dark:bg-neutral-100 dark:text-neutral-900 dark:shadow-neutral-900/10 dark:hover:bg-neutral-200 dark:hover:shadow-neutral-900/20"
              onclick={(e: MouseEvent) => {
                e.stopPropagation();
                readClipboardImage();
              }}
            >
              <ClipboardText class="size-4" />
              识别剪贴板图片
            </Button.Root>
          </div>
        </div>
      </div>
    {:else}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        bind:this={wheelContainer}
        class="relative w-[45%] cursor-grab overflow-hidden rounded-3xl border border-neutral-200/80 bg-neutral-50/40 shadow-inner backdrop-blur-md active:cursor-grabbing dark:border-neutral-800/80 dark:bg-neutral-900/20"
        onmousedown={handleMouseDown}
        ondblclick={handleDoubleClick}
        role="region"
        aria-label="图片预览区"
      >
        <div
          class="pointer-events-none absolute inset-0 bg-[radial-gradient(#e5e7eb_1.5px,transparent_1.5px)] [background-size:24px_24px] opacity-60 dark:bg-[radial-gradient(#262626_1.5px,transparent_1.5px)] dark:opacity-100"
        ></div>

        {#if ocrResult && !isProcessing && ocrEngine === "ai"}
          <div
            class="pointer-events-none absolute top-4 left-4 z-10 rounded-xl border border-neutral-200/50 bg-white/80 px-2.5 py-1.5 text-[10px] font-semibold text-neutral-600 shadow-sm backdrop-blur-md dark:border-neutral-800/50 dark:bg-neutral-950/80 dark:text-neutral-300"
          >
            ✨ AI 高精度识别模式（无高亮定位框）
          </div>
        {/if}

        {#if isProcessing}
          <div
            class="animate-scan absolute right-0 left-0 z-20 h-1 bg-gradient-to-r from-transparent via-blue-500 to-transparent shadow-[0_0_15px_#3b82f6]"
          ></div>

          <div
            class="absolute inset-0 z-10 flex flex-col items-center justify-center bg-white/40 backdrop-blur-xs dark:bg-neutral-950/40"
          >
            <div class="relative flex items-center justify-center">
              <div
                class="absolute size-16 animate-ping rounded-full border-2 border-blue-500/10"
              ></div>
              <Spinner class="size-10 animate-spin text-blue-500" />
            </div>
            <span
              class="mt-5 animate-pulse text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
              >正在扫描图像文字...</span
            >
          </div>
        {/if}
        <div
          class="relative"
          style="width: {baseDisplayWidth}px; height: {baseDisplayHeight}px; transform: translate({translateX}px, {translateY}px) scale({zoom}); transform-origin: 0 0;"
        >
          <img
            src={displayImageSrc}
            class="h-full w-full object-fill select-none"
            onload={handleImageLoad}
            alt="OCR Source"
            draggable="false"
          />

          {#if ocrResult && !isProcessing}
            <div class="pointer-events-auto absolute inset-0">
              {#each ocrResult.lines as line}
                {#if searchQuery === "" || line.text
                    .toLowerCase()
                    .includes(searchQuery.toLowerCase())}
                  <div
                    class="group absolute cursor-pointer rounded border border-blue-500/25 bg-blue-500/5 transition-all duration-200 hover:border-blue-500/90 hover:bg-blue-500/15 hover:shadow-lg hover:shadow-blue-500/10"
                    style="
                        left: {(line.x / naturalWidth) * 100}%;
                        top: {(line.y / naturalHeight) * 100}%;
                        width: {(line.width / naturalWidth) * 100}%;
                        height: {(line.height / naturalHeight) * 100}%;
                      "
                    onclick={() => {
                      if (hasMoved) return;
                      copyText(line.text);
                      if (ocrResult) {
                        const idx = ocrResult.lines.indexOf(line);
                        if (idx !== -1) {
                          highlightAndScrollToLine(idx);
                        }
                      }
                    }}
                    onkeydown={(e) => {
                      if (e.key === "Enter" && ocrResult) {
                        copyText(line.text);
                        const idx = ocrResult.lines.indexOf(line);
                        if (idx !== -1) {
                          highlightAndScrollToLine(idx);
                        }
                      }
                    }}
                    role="button"
                    tabindex="0"
                    title="点击定位并复制"
                  >
                    <span
                      class="pointer-events-none absolute bottom-full left-1/2 z-50 mb-2.5 line-clamp-3 hidden max-w-[250px] min-w-[150px] -translate-x-1/2 rounded-xl border border-neutral-800 bg-neutral-950 px-3 py-2 text-center text-xs whitespace-normal text-white shadow-xl group-hover:block dark:border-neutral-700"
                    >
                      {line.text}
                    </span>
                  </div>
                {/if}
              {/each}
            </div>
          {/if}
        </div>

        <div
          class="no-pan absolute right-4 bottom-4 z-20 flex items-center gap-1.5 rounded-xl border border-neutral-200/60 bg-white/80 p-1.5 shadow-md backdrop-blur-md dark:border-neutral-800/80 dark:bg-neutral-950/80"
        >
          <Button.Root
            class="flex size-6 items-center justify-center rounded-lg text-sm font-bold text-neutral-600 transition-colors hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800"
            onclick={() => zoomTo(zoom - 0.1 * zoom)}
            title="缩小"
          >
            -
          </Button.Root>
          <span
            class="min-w-[36px] text-center font-mono text-[11px] font-semibold text-neutral-700 select-text dark:text-neutral-300"
          >
            {Math.round(zoom * 100)}%
          </span>
          <Button.Root
            class="flex size-6 items-center justify-center rounded-lg text-sm font-bold text-neutral-600 transition-colors hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800"
            onclick={() => zoomTo(zoom + 0.1 * zoom)}
            title="放大"
          >
            +
          </Button.Root>
          <span class="mx-0.5 h-3 w-[1px] bg-neutral-200 dark:bg-neutral-800"
          ></span>
          <Button.Root
            class="rounded-lg bg-neutral-100 px-2 py-0.5 text-[10px] font-semibold text-neutral-700 transition-colors hover:bg-neutral-200 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
            onclick={() => {
              zoom = 1.0;
              resetPosition();
            }}
          >
            自适应
          </Button.Root>
        </div>
      </div>

      <Tabs.Root
        value={activeTab}
        onValueChange={(v) => v && (activeTab = v as "merged" | "lines")}
        class="flex w-[55%] flex-col overflow-hidden rounded-3xl border border-neutral-200/80 bg-white/90 shadow-xl shadow-neutral-200/10 backdrop-blur-md dark:border-neutral-800/80 dark:bg-neutral-900/40 dark:shadow-neutral-950/30"
      >
        <div
          class="flex items-center justify-between border-b border-neutral-200/60 bg-neutral-50/40 px-4 py-2.5 dark:border-neutral-800/60 dark:bg-neutral-950/20"
        >
          <div class="w-40">
            <Tabs.List
              class="flex gap-1 rounded-lg bg-neutral-100 p-0.5 dark:bg-neutral-800/80"
            >
              <Tabs.Trigger
                value="merged"
                class="flex-1 rounded-md py-1 text-xs font-semibold transition-all data-[state=active]:bg-white data-[state=active]:text-neutral-900 data-[state=active]:shadow-xs dark:text-neutral-400 dark:data-[state=active]:bg-neutral-700 dark:data-[state=active]:text-white"
              >
                完整文本
              </Tabs.Trigger>
              <Tabs.Trigger
                value="lines"
                class="flex-1 rounded-md py-1 text-xs font-semibold transition-all data-[state=active]:bg-white data-[state=active]:text-neutral-900 data-[state=active]:shadow-xs dark:text-neutral-400 dark:data-[state=active]:bg-neutral-700 dark:data-[state=active]:text-white"
              >
                逐行列表
              </Tabs.Trigger>
            </Tabs.List>
          </div>

          <div class="flex items-center gap-2">
            {#if ocrResult}
              <Button.Root
                class="flex h-7 items-center gap-1 rounded-lg bg-blue-500 px-2.5 text-xs font-semibold text-white shadow-sm transition-colors hover:bg-blue-600 active:scale-95"
                onclick={() =>
                  copyText(
                    activeTab === "merged" ? editableText : ocrResult!.text,
                  )}
              >
                <Copy class="size-3.5" />
                复制全部
              </Button.Root>
            {/if}

            <Button.Root
              class="flex h-7 items-center gap-1 rounded-lg border border-neutral-200 px-2.5 text-xs font-semibold text-neutral-700 transition-colors hover:bg-neutral-50 active:scale-95 dark:border-neutral-800/80 dark:text-neutral-200 dark:hover:bg-neutral-800"
              onclick={handleClear}
            >
              <Trash class="size-3.5" />
              清空
            </Button.Root>
          </div>
        </div>

        <div class="relative flex min-h-0 flex-1 flex-col">
          <Tabs.Content
            value="merged"
            class="flex min-h-0 w-full flex-1 flex-col justify-between focus:outline-none"
          >
            <textarea
              bind:value={editableText}
              disabled={isProcessing}
              placeholder="识别文本将显示在此处，您也可以在此编辑或调整内容..."
              class="w-full flex-1 resize-none border-0 bg-transparent p-4 font-sans text-sm leading-relaxed text-neutral-800 focus:border-0 focus:ring-0 focus:outline-none dark:text-neutral-200"
            ></textarea>

            <div
              class="flex items-center justify-between border-t border-neutral-200/60 bg-neutral-50/50 px-4 py-2 text-xs text-neutral-500 dark:border-neutral-800/60 dark:bg-neutral-950/20 dark:text-neutral-400"
            >
              <div class="flex items-center gap-3">
                <span
                  >字符数: <strong
                    class="font-mono text-neutral-700 dark:text-neutral-200"
                    >{editableText.length}</strong
                  > 字</span
                >
                <span class="h-3 w-[1px] bg-neutral-300 dark:bg-neutral-700"
                ></span>
                <span
                  >行数: <strong
                    class="font-mono text-neutral-700 dark:text-neutral-200"
                    >{editableText
                      ? editableText.split("\n").length
                      : 0}</strong
                  > 行</span
                >
              </div>
              {#if editableText}
                <div class="flex items-center gap-1.5">
                  <Button.Root
                    class="flex items-center gap-1 rounded-lg px-2 py-1 text-[11px] font-semibold transition-colors hover:bg-neutral-200 hover:text-neutral-800 dark:hover:bg-neutral-800 dark:hover:text-neutral-200"
                    onclick={cleanTextSpaces}
                    title="清理段落多余空格并去除首尾空白"
                  >
                    <TextIndent class="size-3" />
                    <span>整理空白</span>
                  </Button.Root>
                </div>
              {/if}
            </div>
          </Tabs.Content>

          <Tabs.Content
            value="lines"
            class="flex h-full min-h-0 w-full flex-1 flex-col focus:outline-none"
          >
            <div class="h-full min-h-0 w-full flex-1">
              <AppScrollArea
                class="h-full w-full"
                viewportClass="h-full w-full"
              >
                <div class="flex flex-col gap-2 p-4">
                  {#if filteredLines.length === 0}
                    <div
                      class="flex flex-col items-center justify-center py-16 text-neutral-400 dark:text-neutral-500"
                    >
                      <MagnifyingGlass class="mb-2 size-8" />
                      <span class="text-xs">没有找到匹配的文字行</span>
                    </div>
                  {:else}
                    {#each filteredLines as line}
                      {@const originalIndex = ocrResult!.lines.indexOf(line)}
                      <div
                        id="line-item-{originalIndex}"
                        class="group flex items-center justify-between gap-3 rounded-xl border p-3 transition-all duration-200 {selectedLineIndex ===
                        originalIndex
                          ? 'border-blue-500/50 bg-blue-500/10 shadow-md ring-1 shadow-blue-500/5 ring-blue-500/20 dark:border-blue-400/50 dark:bg-blue-400/10 dark:ring-blue-400/20'
                          : 'border-neutral-200/40 bg-neutral-50/20 hover:border-neutral-200 hover:bg-neutral-50/50 dark:border-neutral-800/40 dark:bg-neutral-900/10 dark:hover:border-neutral-800 dark:hover:bg-neutral-900/30'}"
                        onclick={() => {
                          selectedLineIndex = originalIndex;
                        }}
                        onkeydown={(e) =>
                          e.key === "Enter" &&
                          (selectedLineIndex = originalIndex)}
                        role="button"
                        tabindex="0"
                      >
                        <span
                          class="font-sans text-xs leading-relaxed break-all text-neutral-800 select-text dark:text-neutral-200"
                        >
                          {line.text}
                        </span>
                        <Button.Root
                          class="flex-shrink-0 rounded-lg border border-neutral-200 bg-white p-1.5 text-neutral-500 opacity-0 shadow-xs transition-all group-hover:opacity-100 hover:bg-neutral-50 hover:text-neutral-700 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800 dark:hover:text-neutral-200"
                          onclick={(e: MouseEvent) => {
                            e.stopPropagation();
                            copyText(line.text);
                          }}
                          title="复制此行"
                        >
                          <Copy class="size-3.5" />
                        </Button.Root>
                      </div>
                    {/each}
                  {/if}
                </div>
              </AppScrollArea>
            </div>
          </Tabs.Content>
        </div>
      </Tabs.Root>
    {/if}
  </div>
</div>

<style>
  @keyframes scan {
    0% {
      top: 0%;
    }
    50% {
      top: 100%;
    }
    100% {
      top: 0%;
    }
  }
  .animate-scan {
    animation: scan 2.5s linear infinite;
  }
</style>
