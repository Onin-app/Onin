<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    Microphone,
    SpeakerHigh,
    Stop,
    Pause,
    Play,
    X,
  } from "phosphor-svelte";
  import "../../index.css"; // 引入全局 CSS 样式，保证 Tailwind 样式 100% 生效

  // 状态机定义
  type RecordState = "idle" | "recording" | "paused";

  interface RecordStateSnapshot {
    state: RecordState;
    durationSecs: number;
  }

  // 录制配置
  let isInitialized = $state(false);
  let recordAudio = $state(true);
  let recordSystemSound = $state(false);
  let excludeOwnWindow = $state(true);
  let fps = $state(30);
  let selectedMonitorIndex = $state(0); // 默认选择第一块屏幕
  let saveFolderType = $state<"video" | "download" | "desktop" | "custom">(
    "video",
  );
  let customSaveFolder = $state("");
  let saveTimeout = $state<number | undefined>(undefined);
  let recordTargetType = $state<"screen" | "window" | "area">("screen");
  let selectedWindowHandle = $state<string | null>(null);
  let areaRect = $state<{
    x: number;
    y: number;
    width: number;
    height: number;
  } | null>(null);

  // 延迟 300ms 异步防抖保存配置，过滤开关连击引发的系统 IPC 及磁盘 I/O 阻塞
  function debouncedSaveConfig() {
    if (!isInitialized) return;
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      try {
        const config = {
          fps,
          recordAudio,
          recordSystemSound,
          excludeOwnWindow,
          monitorIndex: selectedMonitorIndex,
          saveFolderType,
          customSaveFolder,
          recordTargetType,
          windowHandle: selectedWindowHandle,
          areaRect,
        };
        localStorage.setItem(
          "onin_screen_recorder_config",
          JSON.stringify(config),
        );
        await invoke("save_recorder_config", { config });
      } catch (e) {
        console.error("Failed to save config in bar:", e);
      }
    }, 300) as unknown as number;
  }

  $effect(() => {
    debouncedSaveConfig();
  });

  // 录制状态
  let isRecording = $state(false);
  let isPaused = $state(false);
  let durationSecs = $state(0);
  let outputFilePath = $state("");

  let timerId: any = null;

  // 格式化时间 00:00
  let formattedTime = $derived.by(() => {
    const mins = Math.floor(durationSecs / 60);
    const secs = durationSecs % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  });

  // 轮询查询 Rust 状态
  async function pollState() {
    try {
      const snapshot = await invoke<RecordStateSnapshot>(
        "get_screen_record_state",
      );
      durationSecs = snapshot.durationSecs;

      if (snapshot.state === "recording") {
        isRecording = true;
        isPaused = false;
      } else if (snapshot.state === "paused") {
        isRecording = true;
        isPaused = true;
      } else {
        isRecording = false;
        isPaused = false;
      }
    } catch (e) {
      console.error("Failed to poll recording state", e);
    }
  }

  // 开始录制
  async function handleStart() {
    try {
      const config = {
        fps,
        recordAudio,
        recordSystemSound,
        excludeOwnWindow,
        monitorIndex: selectedMonitorIndex,
        saveFolderType,
        customSaveFolder,
        recordTargetType,
        windowHandle: selectedWindowHandle,
        areaRect,
      };

      outputFilePath = await invoke<string>("start_screen_record", { config });
      isRecording = true;
      isPaused = false;
    } catch (e) {
      alert("启动录像失败: " + e);
    }
  }

  // 暂停
  async function handlePause() {
    try {
      await invoke("pause_screen_record");
      isPaused = true;
    } catch (e) {
      console.error(e);
    }
  }

  // 恢复
  async function handleResume() {
    try {
      await invoke("resume_screen_record");
      isPaused = false;
    } catch (e) {
      console.error(e);
    }
  }

  // 停止并保存
  async function handleStop() {
    try {
      await invoke("stop_screen_record");
      isRecording = false;
      isPaused = false;

      // 直接销毁当前独立子窗口
      const win = getCurrentWindow();
      await win.close();
    } catch (e) {
      console.error(e);
    }
  }

  // 取消并关闭
  async function handleCancel() {
    try {
      if (isRecording) {
        await invoke("stop_screen_record");
      }
      const win = getCurrentWindow();
      await win.close();
    } catch (e) {
      console.error(e);
    }
  }

  onMount(async () => {
    pollState();
    timerId = setInterval(pollState, 1000);

    // 从 Rust 全局唯一的可信数据源加载配置，解决跨 Webview 窗口本地存储同步缺陷
    try {
      const config = await invoke<any>("get_recorder_config");
      fps = config.fps ?? 30;
      recordAudio = config.recordAudio ?? true;
      recordSystemSound = config.recordSystemSound ?? false;
      excludeOwnWindow = config.excludeOwnWindow ?? true;
      selectedMonitorIndex = config.monitorIndex ?? 0;
      if (selectedMonitorIndex === -1) {
        selectedMonitorIndex = 0;
      }
      saveFolderType = config.saveFolderType ?? "video";
      customSaveFolder = config.customSaveFolder ?? "";
      recordTargetType = config.recordTargetType ?? "screen";
      selectedWindowHandle = config.windowHandle ?? null;
      areaRect = config.areaRect ?? null;
    } catch (e) {
      console.error("Failed to load screen recorder config in bar:", e);
    } finally {
      isInitialized = true;
    }

    // 渲染完毕后通知 Rust 显示当前专属独立窗口，彻底防止白屏闪烁
    invoke("show_recorder_bar_window").catch(console.error);
  });

  onDestroy(() => {
    if (timerId) {
      clearInterval(timerId);
    }
    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
  });
</script>

<!-- 外层透明容器，作为物理缓冲边界，防止系统为透明窗口渲染白边或投影 -->
<div
  class="flex h-full w-full items-center justify-center overflow-hidden bg-transparent"
  data-tauri-drag-region
>
  <main
    class="box-sizing-border flex h-[72px] w-[360px] items-center rounded-2xl border border-neutral-800 bg-neutral-950/90 px-5 text-white shadow-xl backdrop-blur-md select-none dark:bg-neutral-950/90"
    data-tauri-drag-region
  >
    {#if !isRecording}
      <!-- 准备配置区 -->
      <div class="flex h-full flex-1 items-center gap-5" data-tauri-drag-region>
        <div
          class="flex cursor-pointer items-center gap-1.5"
          data-tauri-drag-region
        >
          <label
            class="flex cursor-pointer items-center gap-1.5 text-xs text-neutral-400 transition-colors hover:text-white"
          >
            <input
              type="checkbox"
              bind:checked={recordAudio}
              class="cursor-pointer accent-red-500"
            />
            <Microphone class="size-3.5" />
            <span>麦风</span>
          </label>
        </div>
        <div
          class="flex cursor-pointer items-center gap-1.5"
          data-tauri-drag-region
        >
          <label
            class="flex cursor-pointer items-center gap-1.5 text-xs text-neutral-400 transition-colors hover:text-white"
          >
            <input
              type="checkbox"
              bind:checked={recordSystemSound}
              class="cursor-pointer accent-red-500"
            />
            <SpeakerHigh class="size-3.5" />
            <span>系统音</span>
          </label>
        </div>

        <!-- 空白拉伸占位符以支持拖拽 -->
        <div class="h-full flex-grow" data-tauri-drag-region></div>

        <button
          class="flex items-center gap-2 rounded-xl bg-red-600 px-4 py-2 text-xs font-semibold text-white shadow-md transition-all hover:bg-red-500 active:scale-95"
          onclick={handleStart}
          title="开始录制"
        >
          <span class="size-2 animate-pulse rounded-full bg-white"></span>
          录屏
        </button>
      </div>
    {:else}
      <!-- 录像状态区 -->
      <div
        class="flex h-full flex-1 items-center justify-between"
        data-tauri-drag-region
      >
        <div class="flex items-center gap-2.5" data-tauri-drag-region>
          <span
            class="size-2.5 rounded-full {isPaused
              ? 'bg-yellow-500 shadow-[0_0_10px_rgba(234,179,8,0.5)]'
              : 'animate-pulse bg-red-500 shadow-[0_0_10px_rgba(239,68,68,0.5)]'}"
          ></span>
          <span
            class="font-mono text-lg font-bold tracking-wide"
            data-tauri-drag-region>{formattedTime}</span
          >
        </div>

        <div class="mr-3 flex items-center gap-2">
          {#if !isPaused}
            <button
              class="flex size-9 items-center justify-center rounded-full border border-neutral-800 bg-neutral-900 text-white transition-all hover:bg-neutral-800 active:scale-95"
              onclick={handlePause}
              title="暂停录制"
            >
              <Pause class="size-4" />
            </button>
          {:else}
            <button
              class="flex size-9 items-center justify-center rounded-full border border-yellow-800/40 bg-yellow-950/50 text-yellow-500 transition-all hover:bg-yellow-900/50 active:scale-95"
              onclick={handleResume}
              title="恢复录制"
            >
              <Play class="size-4 fill-yellow-500" />
            </button>
          {/if}

          <button
            class="flex size-9 items-center justify-center rounded-full border border-red-800/40 bg-red-950/50 text-red-500 transition-all hover:bg-red-900/50 active:scale-95"
            onclick={handleStop}
            title="完成并保存"
          >
            <Stop class="size-4 fill-red-500" />
          </button>
        </div>
      </div>
    {/if}

    <!-- 关闭按钮 -->
    <button
      class="ml-1 flex size-6 items-center justify-center rounded-full bg-transparent text-neutral-500 transition-all hover:bg-neutral-800 hover:text-white"
      onclick={handleCancel}
      title="退出"
    >
      <X class="size-4" />
    </button>
  </main>
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
