<script lang="ts">
  /**
   * Onin 内置 OCR (文字识别) 扩展
   */
  import { onMount, onDestroy } from "svelte";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";
  import {
    Copy,
    Trash,
    ArrowLeft,
    Spinner,
    FileImage,
    ClipboardText,
    ArrowCounterClockwise,
    MagnifyingGlass,
  } from "phosphor-svelte";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";

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
  let ocrLanguage = $state<string>("auto");
  let displayImageSrc = $derived(
    imageSrc
      ? imageSrc.startsWith("data:")
        ? imageSrc
        : convertFileSrc(imageSrc)
      : null,
  );

  // 图片展示宽高尺寸
  let naturalWidth = $state(1);
  let naturalHeight = $state(1);
  let displayWidth = $state(1);
  let displayHeight = $state(1);
  let imgContainer = $state<HTMLDivElement | null>(null);

  // Tab 状态
  let activeTab = $state<"merged" | "lines">("merged");

  // 对合并文本的响应式编辑状态
  let editableText = $state("");

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

    try {
      const result = await invoke<OcrResult>("plugin_ocr_recognize", {
        image: src,
        options: ocrLanguage === "auto" ? null : { language: ocrLanguage },
      });

      ocrResult = result;
      editableText = result.text;
      toast.success("文字识别完成");
    } catch (error) {
      console.error("OCR Failed:", error);
      toast.error(typeof error === "string" ? error : "识别失败，请重试");
      imageSrc = null;
    } finally {
      isProcessing = false;
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

    // 默认最大宽度限制在容器的可用宽度（假设约 480 像素），最大高度限制 400 像素
    const maxW = Math.min(480, imgContainer?.clientWidth || 480);
    const maxH = 380;

    let scale = Math.min(maxW / naturalWidth, maxH / naturalHeight);
    if (scale > 1) {
      scale = 1; // 不放大原图
    }

    displayWidth = naturalWidth * scale;
    displayHeight = naturalHeight * scale;
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

  // 清空数据
  function handleClear() {
    imageSrc = null;
    ocrResult = null;
    editableText = "";
    lastClipboardImage = null;
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

  onMount(() => {
    // 监听聚焦，如果是自动切回来可以实现无缝识别
    window.addEventListener("focus", handleWindowFocus);
    window.addEventListener("resize", calculateDisplaySize);
    document.addEventListener("paste", handlePaste);

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
</script>

<div class="flex h-full w-full flex-col p-3 select-none">
  <!-- 头部顶栏 -->
  <ExtensionHeader
    placeholder="在识别结果中过滤/搜索行..."
    bind:value={searchQuery}
    onBack={handleBack}
  />

  <div class="mt-2 flex min-h-0 flex-1 flex-row gap-4 overflow-hidden">
    <!-- 未选择图片时的 Dropzone 状态 -->
    {#if !imageSrc}
      <div
        class="flex flex-1 flex-col items-center justify-center rounded-2xl border-2 border-dashed border-neutral-300 bg-white/40 p-8 transition-colors hover:border-neutral-400 hover:bg-white/60 dark:border-neutral-700 dark:bg-neutral-800/40 dark:hover:border-neutral-600 dark:hover:bg-neutral-800/60 {isDragging
          ? 'border-blue-500 bg-blue-500/5 dark:border-blue-400 dark:bg-blue-400/5'
          : ''}"
        role="button"
        tabindex="0"
        ondragover={handleDragOver}
        ondragleave={handleDragLeave}
        ondrop={handleDrop}
        onclick={selectLocalFile}
        onkeydown={(e) => e.key === "Enter" && selectLocalFile()}
      >
        <FileImage
          class="mb-4 size-16 text-neutral-400 dark:text-neutral-500"
        />
        <h3
          class="mb-2 text-xl font-medium text-neutral-800 dark:text-neutral-200"
        >
          拖入图片文件，或点击选择本地图片
        </h3>
        <p
          class="mb-6 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400"
        >
          支持直接按下 <kbd
            class="rounded border border-neutral-300 bg-neutral-100 px-1.5 py-0.5 text-xs text-neutral-700 dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-300"
            >Ctrl+V</kbd
          >
          (或
          <kbd
            class="rounded border border-neutral-300 bg-neutral-100 px-1.5 py-0.5 text-xs text-neutral-700 dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-300"
            >Cmd+V</kbd
          >) 粘贴已复制的图片
        </p>

        <div class="flex gap-3">
          <button
            class="flex items-center gap-2 rounded-xl bg-neutral-200 px-5 py-2.5 font-medium text-neutral-700 transition-colors hover:bg-neutral-300 active:scale-95 dark:bg-neutral-700 dark:text-neutral-200 dark:hover:bg-neutral-600"
            onclick={(e) => {
              e.stopPropagation();
              readClipboardImage();
            }}
          >
            <ClipboardText class="size-5" />
            识别剪贴板图片
          </button>
        </div>
      </div>
    {:else}
      <!-- 左右分栏面板 -->
      <!-- 左栏：图片和高亮定位 -->
      <div
        bind:this={imgContainer}
        class="relative flex w-1/2 flex-col items-center justify-center overflow-hidden rounded-2xl border border-neutral-200 bg-neutral-50/50 p-4 dark:border-neutral-800 dark:bg-neutral-900/50"
      >
        {#if isProcessing}
          <!-- 加载遮罩 -->
          <div
            class="absolute inset-0 z-10 flex flex-col items-center justify-center bg-white/75 backdrop-blur-sm dark:bg-neutral-900/75"
          >
            <Spinner class="mb-3 size-10 animate-spin text-blue-500" />
            <span
              class="text-sm font-medium text-neutral-600 dark:text-neutral-400"
              >正在识别文字，请稍候...</span
            >
          </div>
        {/if}

        <div
          class="relative flex items-center justify-center overflow-hidden rounded-xl border border-neutral-200 bg-neutral-100 shadow-inner dark:border-neutral-800 dark:bg-neutral-950"
          style="width: {displayWidth}px; height: {displayHeight}px;"
        >
          <img
            src={displayImageSrc}
            class="h-full w-full object-fill select-none"
            onload={handleImageLoad}
            alt="OCR Source"
          />

          <!-- 定位高亮框图层 -->
          {#if ocrResult && !isProcessing}
            <div class="pointer-events-auto absolute inset-0">
              {#each ocrResult.lines as line}
                <!-- 仅当此行匹配搜索条件时高亮显示 -->
                {#if searchQuery === "" || line.text
                    .toLowerCase()
                    .includes(searchQuery.toLowerCase())}
                  <div
                    class="group absolute cursor-pointer rounded border border-blue-500/20 bg-blue-500/5 transition-all hover:border-blue-500/80 hover:bg-blue-500/25"
                    style="
                      left: {(line.x / naturalWidth) * 100}%;
                      top: {(line.y / naturalHeight) * 100}%;
                      width: {(line.width / naturalWidth) * 100}%;
                      height: {(line.height / naturalHeight) * 100}%;
                    "
                    onclick={() => copyText(line.text)}
                    onkeydown={(e) => e.key === "Enter" && copyText(line.text)}
                    role="button"
                    tabindex="0"
                    title="点击复制此行"
                  >
                    <!-- 悬浮提示气泡 -->
                    <span
                      class="pointer-events-none absolute bottom-full left-1/2 z-50 mb-1.5 line-clamp-3 hidden max-w-[250px] min-w-[150px] -translate-x-1/2 rounded-lg bg-neutral-900 px-2.5 py-1.5 text-center text-xs whitespace-normal text-white shadow-lg group-hover:block"
                    >
                      {line.text}
                    </span>
                  </div>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <!-- 右栏：合并文本与搜索 -->
      <div
        class="flex w-1/2 flex-col overflow-hidden rounded-2xl border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-800/40"
      >
        <!-- 选项卡与操作栏 -->
        <div
          class="flex items-center justify-between border-b border-neutral-200 bg-neutral-50/50 px-4 py-2 dark:border-neutral-800 dark:bg-neutral-900/30"
        >
          <div
            class="flex gap-1 rounded-lg bg-neutral-200/60 p-0.5 dark:bg-neutral-700/60"
          >
            <button
              class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors {activeTab ===
              'merged'
                ? 'bg-white text-neutral-800 shadow-sm dark:bg-neutral-800 dark:text-white'
                : 'text-neutral-500 hover:text-neutral-800 dark:text-neutral-400'}"
              onclick={() => (activeTab = "merged")}
            >
              完整文本
            </button>
            <button
              class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors {activeTab ===
              'lines'
                ? 'bg-white text-neutral-800 shadow-sm dark:bg-neutral-800 dark:text-white'
                : 'text-neutral-500 hover:text-neutral-800 dark:text-neutral-400'}"
              onclick={() => (activeTab = "lines")}
            >
              逐行列表
            </button>
          </div>

          <div class="flex items-center gap-2">
            <select
              bind:value={ocrLanguage}
              onchange={() => imageSrc && recognizeImage(imageSrc)}
              class="rounded-lg border border-neutral-300 bg-white px-2 py-1 text-xs font-semibold text-neutral-700 transition-colors outline-none hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-200 dark:hover:bg-neutral-700"
            >
              <option value="auto">自动 (混合)</option>
              <option value="zh-Hans">中文 (简体)</option>
              <option value="en-US">英文 (English)</option>
            </select>

            {#if ocrResult}
              <button
                class="flex items-center gap-1.5 rounded-lg bg-blue-500 px-3 py-1.5 text-xs font-semibold text-white shadow-sm transition-colors hover:bg-blue-600 active:scale-95"
                onclick={() =>
                  copyText(
                    activeTab === "merged" ? editableText : ocrResult!.text,
                  )}
              >
                <Copy class="size-4" />
                复制全部
              </button>
            {/if}
            <button
              class="flex items-center gap-1.5 rounded-lg border border-neutral-300 px-3 py-1.5 text-xs font-semibold text-neutral-700 transition-colors hover:bg-neutral-100 active:scale-95 dark:border-neutral-600 dark:text-neutral-200 dark:hover:bg-neutral-700"
              onclick={handleClear}
            >
              <Trash class="size-4" />
              清空
            </button>
          </div>
        </div>

        <!-- 文本内容展示区 -->
        <div class="relative min-h-0 flex-1">
          {#if activeTab === "merged"}
            <!-- 完整文本面板 (支持编辑以便快速修改) -->
            <textarea
              bind:value={editableText}
              disabled={isProcessing}
              placeholder="识别文本将显示在此处，您也可以在此编辑或调整内容..."
              class="h-full w-full resize-none border-0 bg-transparent p-4 text-base text-neutral-800 focus:border-0 focus:ring-0 focus:outline-none dark:text-neutral-200"
            ></textarea>
          {:else}
            <!-- 逐行显示面板 -->
            <AppScrollArea>
              <div class="flex flex-col gap-2 p-3">
                {#if filteredLines.length === 0}
                  <div
                    class="flex flex-col items-center justify-center py-12 text-neutral-400 dark:text-neutral-500"
                  >
                    <MagnifyingGlass class="mb-2 size-8" />
                    <span>没有找到匹配的文字行</span>
                  </div>
                {:else}
                  {#each filteredLines as line, i}
                    <div
                      class="group flex items-center justify-between gap-3 rounded-xl border border-neutral-100 bg-neutral-50/30 p-2.5 transition-colors hover:bg-neutral-100/50 dark:border-neutral-800 dark:bg-neutral-900/10 dark:hover:bg-neutral-900/30"
                    >
                      <span
                        class="text-sm break-all text-neutral-800 select-text dark:text-neutral-200"
                      >
                        {line.text}
                      </span>
                      <button
                        class="flex-shrink-0 rounded-lg bg-neutral-200 p-1.5 text-neutral-600 opacity-0 transition-all group-hover:opacity-100 hover:bg-neutral-300 active:scale-90 dark:bg-neutral-700 dark:text-neutral-300 dark:hover:bg-neutral-600"
                        onclick={() => copyText(line.text)}
                        title="复制此行"
                      >
                        <Copy class="size-3.5" />
                      </button>
                    </div>
                  {/each}
                {/if}
              </div>
            </AppScrollArea>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
