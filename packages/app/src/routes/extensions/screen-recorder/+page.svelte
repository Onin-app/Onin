<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    VideoCamera,
    Microphone,
    SpeakerHigh,
    Trash,
    Play,
    Spinner,
  } from "phosphor-svelte";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";
  import { goto } from "$app/navigation";

  interface RecordedVideo {
    name: string;
    path: string;
    sizeBytes: number;
    createdTime: number;
  }

  // 录屏配置 (Svelte 5 Runes)
  let recordAudio = $state(true);
  let recordSystemSound = $state(false);
  let excludeOwnWindow = $state(true);
  let fps = $state(30);

  // 历史视频列表 (Svelte 5 Runes)
  let videos = $state<RecordedVideo[]>([]);
  let isLoading = $state(true);

  // 格式化时间戳
  function formatDateTime(timestamp: number): string {
    if (!timestamp) return "未知时间";
    const date = new Date(timestamp);
    return date.toLocaleString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }

  // 格式化文件大小
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  // 加载录制好的视频列表
  async function loadVideos() {
    isLoading = true;
    try {
      videos = await invoke<RecordedVideo[]>("get_recorded_videos");
    } catch (e) {
      console.error("Failed to load recorded videos:", e);
    } finally {
      isLoading = false;
    }
  }

  // 开始录制
  async function handleStartRecord() {
    try {
      await invoke("show_screen_recorder_bar");
      const win = getCurrentWindow();
      await win.hide();
    } catch (e) {
      alert("启动录制服务失败: " + e);
    }
  }

  // 播放视频文件
  async function handlePlay(video: RecordedVideo) {
    try {
      await invoke("open_video_file", { path: video.path });
    } catch (e) {
      alert("播放失败: " + e);
    }
  }

  // 删除视频文件
  async function handleDelete(video: RecordedVideo) {
    const confirmed = confirm(`确定要永久删除录像文件 "${video.name}" 吗？`);
    if (!confirmed) return;

    try {
      await invoke("delete_video_file", { path: video.path });
      videos = videos.filter((v) => v.path !== video.path);
    } catch (e) {
      alert("删除失败: " + e);
    }
  }

  function handleBack() {
    goto("/");
  }

  onMount(() => {
    loadVideos();

    // 监听主窗口重新聚焦的事件，自动刷新视频列表
    const unlistenPromise = listen("tauri://focus", () => {
      loadVideos();
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });
</script>

<div
  class="relative flex h-full w-full flex-col overflow-hidden select-none"
  data-tauri-drag-region
>
  <ExtensionHeader
    showSearch={false}
    extensionId="screen_recorder"
    title="屏幕录制"
    onBack={handleBack}
  />

  <div class="mt-3 flex min-h-0 flex-1 flex-row gap-5 overflow-hidden p-6">
    <!-- 左侧：参数配置面板 -->
    <div class="w-[280px] flex-shrink-0">
      <div
        class="flex h-full flex-col rounded-3xl border border-neutral-200/80 bg-white/40 p-5 shadow-xs backdrop-blur-md dark:border-neutral-800/80 dark:bg-neutral-900/40"
      >
        <h3
          class="mb-5 text-sm font-semibold tracking-tight text-neutral-800 dark:text-neutral-100"
        >
          录制配置
        </h3>

        <div class="flex flex-grow flex-col gap-5">
          <!-- 帧率 -->
          <div class="flex flex-col gap-2">
            <span
              class="text-xs font-semibold text-neutral-500 dark:text-neutral-400"
              >FPS 帧率</span
            >
            <div
              class="flex rounded-lg border border-neutral-200/40 bg-neutral-100 p-0.5 dark:border-neutral-800/40 dark:bg-neutral-950/40"
            >
              <button
                class="flex-1 rounded-md py-1.5 text-center text-xs font-semibold transition-all {fps ===
                30
                  ? 'bg-white text-neutral-900 shadow-xs dark:bg-neutral-800 dark:text-white'
                  : 'text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300'}"
                onclick={() => (fps = 30)}>30 FPS</button
              >
              <button
                class="flex-1 rounded-md py-1.5 text-center text-xs font-semibold transition-all {fps ===
                60
                  ? 'bg-white text-neutral-900 shadow-xs dark:bg-neutral-800 dark:text-white'
                  : 'text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300'}"
                onclick={() => (fps = 60)}>60 FPS</button
              >
            </div>
          </div>

          <!-- 麦克风 -->
          <div class="flex items-center justify-between">
            <div class="flex flex-col">
              <span
                class="text-xs font-semibold text-neutral-800 dark:text-neutral-200"
                >🎙️ 麦克风音频</span
              >
              <span class="text-[10px] text-neutral-400 dark:text-neutral-500"
                >录制外界谈话声音</span
              >
            </div>
            <label class="relative inline-block h-5 w-9">
              <input
                type="checkbox"
                bind:checked={recordAudio}
                class="peer h-0 w-0 opacity-0"
              />
              <span
                class="absolute inset-0 cursor-pointer rounded-full bg-neutral-200 transition-all duration-200 peer-checked:bg-red-500 before:absolute before:bottom-0.5 before:left-0.5 before:h-3.5 before:w-3.5 before:rounded-full before:bg-white before:transition-all before:content-[''] peer-checked:before:translate-x-4 dark:bg-neutral-800"
              ></span>
            </label>
          </div>

          <!-- 系统音 -->
          <div class="flex items-center justify-between">
            <div class="flex flex-col">
              <span
                class="text-xs font-semibold text-neutral-800 dark:text-neutral-200"
                >🔊 系统声卡声音</span
              >
              <span class="text-[10px] text-neutral-400 dark:text-neutral-500"
                >录制播放的系统音量</span
              >
            </div>
            <label class="relative inline-block h-5 w-9">
              <input
                type="checkbox"
                bind:checked={recordSystemSound}
                class="peer h-0 w-0 opacity-0"
              />
              <span
                class="absolute inset-0 cursor-pointer rounded-full bg-neutral-200 transition-all duration-200 peer-checked:bg-red-500 before:absolute before:bottom-0.5 before:left-0.5 before:h-3.5 before:w-3.5 before:rounded-full before:bg-white before:transition-all before:content-[''] peer-checked:before:translate-x-4 dark:bg-neutral-800"
              ></span>
            </label>
          </div>

          <!-- 排除自身 -->
          <div class="flex items-center justify-between">
            <div class="flex flex-col">
              <span
                class="text-xs font-semibold text-neutral-800 dark:text-neutral-200"
                >🛡️ 排除本程序</span
              >
              <span class="text-[10px] text-neutral-400 dark:text-neutral-500"
                >录像时不显示主窗口</span
              >
            </div>
            <label class="relative inline-block h-5 w-9">
              <input
                type="checkbox"
                bind:checked={excludeOwnWindow}
                class="peer h-0 w-0 opacity-0"
              />
              <span
                class="absolute inset-0 cursor-pointer rounded-full bg-neutral-200 transition-all duration-200 peer-checked:bg-red-500 before:absolute before:bottom-0.5 before:left-0.5 before:h-3.5 before:w-3.5 before:rounded-full before:bg-white before:transition-all before:content-[''] peer-checked:before:translate-x-4 dark:bg-neutral-800"
              ></span>
            </label>
          </div>
        </div>

        <button
          class="mt-5 flex items-center justify-center gap-2 rounded-xl bg-red-600 py-3 text-sm font-semibold text-white shadow-md transition-all hover:bg-red-500 active:scale-95"
          onclick={handleStartRecord}
        >
          <span class="size-2 animate-pulse rounded-full bg-white"></span>
          开始屏幕录制
        </button>
      </div>
    </div>

    <!-- 右侧：视频列表 -->
    <div class="flex h-full flex-grow flex-col overflow-hidden">
      <h2
        class="mb-4 text-sm font-semibold text-neutral-500 dark:text-neutral-400"
      >
        历史录像记录
      </h2>

      {#if isLoading}
        <div
          class="flex flex-grow flex-col items-center justify-center gap-3 text-xs text-neutral-500"
        >
          <Spinner class="size-6 animate-spin text-red-500" />
          <span>正在检索视频文件...</span>
        </div>
      {:else if videos.length === 0}
        <div
          class="flex flex-grow flex-col items-center justify-center rounded-2xl border border-dashed border-neutral-200/80 bg-white/20 p-10 text-neutral-400 dark:border-neutral-800/80 dark:bg-neutral-900/10 dark:text-neutral-500"
        >
          <VideoCamera
            class="mb-3 size-10 text-neutral-300 dark:text-neutral-700"
          />
          <p class="text-sm font-semibold">当前没有录像记录</p>
          <span class="text-[11px] font-medium text-neutral-400/80"
            >点击左侧开始您的首次屏幕录制</span
          >
        </div>
      {:else}
        <div
          class="scrollbar-thin flex flex-grow flex-col gap-2 overflow-y-auto pr-1.5"
        >
          {#each videos as video}
            <div
              class="group flex items-center rounded-2xl border border-neutral-200/50 bg-white/40 p-3.5 backdrop-blur-md transition-all duration-200 hover:border-neutral-300/80 dark:border-neutral-800/40 dark:bg-neutral-900/15 dark:hover:border-neutral-700/60"
            >
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="mr-3.5 flex size-9 flex-shrink-0 cursor-pointer items-center justify-center rounded-full bg-red-500/10 text-red-500 transition-all hover:bg-red-500 hover:text-white"
                onclick={() => handlePlay(video)}
                title="点击播放"
              >
                <Play class="size-4 fill-current" />
              </div>
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="flex min-w-0 flex-grow cursor-pointer flex-col gap-1"
                onclick={() => handlePlay(video)}
              >
                <span
                  class="truncate text-xs font-semibold text-neutral-800 dark:text-neutral-100"
                  title={video.name}>{video.name}</span
                >
                <div
                  class="flex gap-4 font-mono text-[10px] text-neutral-400 dark:text-neutral-500"
                >
                  <span>📅 {formatDateTime(video.createdTime)}</span>
                  <span>💾 {formatFileSize(video.sizeBytes)}</span>
                </div>
              </div>
              <div
                class="ml-3 flex-shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
              >
                <button
                  class="flex size-7 items-center justify-center rounded-lg text-neutral-400 transition-colors hover:bg-red-500/10 hover:text-red-500"
                  onclick={() => handleDelete(video)}
                  title="删除视频"
                >
                  <Trash class="size-4" />
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
