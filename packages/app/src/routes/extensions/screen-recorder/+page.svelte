<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";
  import { VideoCamera, Trash, Play, Spinner } from "phosphor-svelte";
  import { Switch, Tabs } from "bits-ui";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";
  import { goto } from "$app/navigation";

  interface RecordedVideo {
    name: string;
    path: string;
    sizeBytes: number;
    createdTime: number;
  }

  interface ScreenRecorderConfig {
    fps?: number;
    recordAudio?: boolean;
    recordSystemSound?: boolean;
    excludeOwnWindow?: boolean;
    monitorIndex?: number;
    saveFolderType?: "video" | "download" | "desktop" | "custom";
    customSaveFolder?: string;
  }

  interface MonitorInfo {
    name: string;
    width: number;
    height: number;
    isPrimary: boolean;
    thumbnail: string;
  }

  // 录屏配置 (Svelte 5 Runes)
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

  let monitors = $state<MonitorInfo[]>([]);
  let activeTab = $state("record");

  // 融合从 Rust 全局内存及 LocalStorage 持久化加载配置，解决配置读取路径分裂和清除失效漏洞
  async function loadConfig() {
    try {
      // 1. 先尝试获取本地持久化历史缓存
      let localConfig: ScreenRecorderConfig | null = null;
      try {
        const configStr = localStorage.getItem("onin_screen_recorder_config");
        if (configStr) localConfig = JSON.parse(configStr);
      } catch (e) {}

      // 2. 从 Rust 内存获取最新的实时同步状态
      const rustConfig = await invoke<ScreenRecorderConfig>(
        "get_recorder_config",
      );

      // 3. 数据融合同步 (优先使用实时内存状态，若无则使用本地缓存恢复)
      fps = rustConfig.fps ?? localConfig?.fps ?? 30;
      recordAudio = rustConfig.recordAudio ?? localConfig?.recordAudio ?? true;
      recordSystemSound =
        rustConfig.recordSystemSound ?? localConfig?.recordSystemSound ?? false;
      excludeOwnWindow =
        rustConfig.excludeOwnWindow ?? localConfig?.excludeOwnWindow ?? true;
      selectedMonitorIndex =
        rustConfig.monitorIndex ?? localConfig?.monitorIndex ?? 0;
      if (selectedMonitorIndex === -1) {
        selectedMonitorIndex = 0;
      }
      saveFolderType =
        rustConfig.saveFolderType ?? localConfig?.saveFolderType ?? "video";
      customSaveFolder =
        rustConfig.customSaveFolder ?? localConfig?.customSaveFolder ?? "";
    } catch (e) {
      console.error("Failed to load screen recorder config:", e);
    }
  }

  // 延迟 300ms 异步防抖保存配置，过滤连击引发的系统 IPC 及磁盘 I/O 阻塞
  function debouncedSaveConfig() {
    if (!isInitialized) return;
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      try {
        const config: ScreenRecorderConfig = {
          fps,
          recordAudio,
          recordSystemSound,
          excludeOwnWindow,
          monitorIndex: selectedMonitorIndex,
          saveFolderType,
          customSaveFolder,
        };
        localStorage.setItem(
          "onin_screen_recorder_config",
          JSON.stringify(config),
        );
        await invoke("save_recorder_config", { config });
      } catch (e) {
        console.error("Failed to save screen recorder config:", e);
      }
    }, 300) as unknown as number;
  }

  // 自动监听并在配置变化时保存
  $effect(() => {
    debouncedSaveConfig();
  });

  // 获取可用屏幕
  async function loadMonitors() {
    try {
      monitors = await invoke<MonitorInfo[]>("get_available_monitors");
      if (selectedMonitorIndex >= monitors.length || selectedMonitorIndex < 0) {
        selectedMonitorIndex = 0;
      }
    } catch (e) {
      console.error("Failed to load monitors:", e);
    }
  }

  // 获取已录制视频
  let videos = $state<RecordedVideo[]>([]);
  let isLoading = $state(true);

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

  async function saveConfigImmediately() {
    if (saveTimeout) {
      clearTimeout(saveTimeout);
      saveTimeout = undefined;
    }
    try {
      const config: ScreenRecorderConfig = {
        fps,
        recordAudio,
        recordSystemSound,
        excludeOwnWindow,
        monitorIndex: selectedMonitorIndex,
        saveFolderType,
        customSaveFolder,
      };
      localStorage.setItem(
        "onin_screen_recorder_config",
        JSON.stringify(config),
      );
      await invoke("save_recorder_config", { config });
    } catch (e) {
      console.error("Failed to save screen recorder config immediately:", e);
    }
  }

  let choosingDirectory = false;
  async function chooseCustomDirectory() {
    if (choosingDirectory) return;
    choosingDirectory = true;
    await invoke("acquire_window_close_lock");
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择录屏视频保存目录",
      });
      if (selected && !Array.isArray(selected)) {
        customSaveFolder = selected;
        saveFolderType = "custom";
        await saveConfigImmediately();
        await loadVideos();
      }
    } catch (e) {
      console.error("Failed to choose custom folder:", e);
    } finally {
      choosingDirectory = false;
      await invoke("release_window_close_lock");
    }
  }

  async function handleFolderTypeChange(
    type: "video" | "download" | "desktop" | "custom",
  ) {
    if (type === "custom") {
      await chooseCustomDirectory();
    } else {
      saveFolderType = type;
      await saveConfigImmediately();
      await loadVideos();
    }
  }

  // 开始录制
  async function handleStartRecord() {
    try {
      // 录制前确保即时保存了最新的配置到 Rust 全局状态中，供 Bar 加载时获取
      const config: ScreenRecorderConfig = {
        fps,
        recordAudio,
        recordSystemSound,
        excludeOwnWindow,
        monitorIndex: selectedMonitorIndex,
        saveFolderType,
        customSaveFolder,
      };
      localStorage.setItem(
        "onin_screen_recorder_config",
        JSON.stringify(config),
      );
      await invoke("save_recorder_config", { config });
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

  // 格式化大小
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  // 格式化日期
  function formatDateTime(timestamp: number): string {
    if (!timestamp) return "-";
    const date = new Date(timestamp);
    const y = date.getFullYear();
    const m = (date.getMonth() + 1).toString().padStart(2, "0");
    const d = date.getDate().toString().padStart(2, "0");
    const hh = date.getHours().toString().padStart(2, "0");
    const mm = date.getMinutes().toString().padStart(2, "0");
    const ss = date.getSeconds().toString().padStart(2, "0");
    return `${y}/${m}/${d} ${hh}:${mm}:${ss}`;
  }

  onMount(() => {
    loadConfig().then(() => {
      isInitialized = true;
    });
    loadMonitors();
    loadVideos();

    // 定时刷新显示器缩略图（改为10秒一次，且仅在录屏配置页激活、主窗口可见时刷新，消除截图带来的系统能耗峰值）
    const monitorInterval = setInterval(() => {
      if (document.visibilityState === "visible" && activeTab === "record") {
        loadMonitors();
      }
    }, 10000);

    // 监听主窗口重新聚焦的事件，自动刷新视频列表
    const unlistenPromise = listen("tauri://focus", () => {
      loadVideos();
      loadMonitors();
    });

    return () => {
      clearInterval(monitorInterval);
      unlistenPromise.then((unlisten) => unlisten());
      if (saveTimeout) clearTimeout(saveTimeout);
    };
  });
</script>

<div
  class="relative flex h-full w-full flex-col overflow-hidden select-none"
  data-tauri-drag-region
>
  <ExtensionHeader showSearch={false} title="屏幕录制" onBack={handleBack} />

  <div class="mt-3 flex min-h-0 flex-1 flex-col overflow-hidden px-8 pb-6">
    <Tabs.Root bind:value={activeTab} class="flex h-full flex-col">
      <!-- 选项卡切换列表 -->
      <div class="mb-4 flex justify-center">
        <Tabs.List
          class="inline-flex items-center gap-1.5 rounded-xl bg-neutral-100 p-1 dark:bg-neutral-900"
        >
          <Tabs.Trigger
            value="record"
            class="inline-flex items-center justify-center rounded-lg px-4 py-1.5 text-xs font-semibold text-neutral-500 transition-all duration-200 hover:text-neutral-900 data-[state=active]:bg-white data-[state=active]:text-neutral-900 data-[state=active]:shadow-xs dark:text-neutral-400 dark:hover:text-neutral-100 dark:data-[state=active]:bg-neutral-800 dark:data-[state=active]:text-neutral-100"
          >
            开始录屏
          </Tabs.Trigger>
          <Tabs.Trigger
            value="history"
            class="inline-flex items-center justify-center rounded-lg px-4 py-1.5 text-xs font-semibold text-neutral-500 transition-all duration-200 hover:text-neutral-900 data-[state=active]:bg-white data-[state=active]:text-neutral-900 data-[state=active]:shadow-xs dark:text-neutral-400 dark:hover:text-neutral-100 dark:data-[state=active]:bg-neutral-800 dark:data-[state=active]:text-neutral-100"
          >
            历史记录
          </Tabs.Trigger>
        </Tabs.List>
      </div>

      <!-- Tab 内容区域 -->
      <div class="flex min-h-0 flex-1 flex-col">
        <!-- 1. 录屏配置界面 -->
        <Tabs.Content
          value="record"
          class="flex h-full justify-center overflow-hidden"
        >
          <div
            class="flex h-full w-[560px] flex-col rounded-2xl border border-neutral-200/80 bg-white/40 p-5 shadow-xs backdrop-blur-md dark:border-neutral-800/80 dark:bg-neutral-900/40"
          >
            <!-- 配置滚动主体 -->
            <AppScrollArea
              class="min-h-0 flex-1 pr-0.5"
              viewportClass="h-full w-full"
            >
              <div class="flex flex-col gap-5 pr-3.5 pb-4">
                <!-- 选择显示器 (平铺大缩略图网格) -->
                <div class="flex flex-col gap-2">
                  <span
                    class="text-xs font-bold text-neutral-800 dark:text-neutral-200"
                  >
                    选择录像屏幕
                  </span>
                  <div
                    class="grid gap-4.5 {monitors.length > 1
                      ? 'grid-cols-2'
                      : 'grid-cols-1'}"
                  >
                    {#each monitors as monitor, i}
                      <!-- svelte-ignore a11y_click_events_have_key_events -->
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div
                        class="relative aspect-[16/10] w-full cursor-pointer overflow-hidden rounded-xl border shadow-xs transition-all duration-200 hover:scale-[1.015]
                          {selectedMonitorIndex === i
                          ? 'border-red-600 shadow-[0_0_10px_rgba(220,38,38,0.22)]'
                          : 'border-neutral-200 bg-neutral-100/40 dark:border-neutral-800 dark:bg-neutral-950/20'}"
                        onclick={() => (selectedMonitorIndex = i)}
                      >
                        <!-- 缩略图图画 -->
                        {#if monitor.thumbnail}
                          <img
                            src={monitor.thumbnail}
                            alt={monitor.name}
                            class="h-full w-full object-cover"
                          />
                        {:else}
                          <div
                            class="flex h-full w-full items-center justify-center bg-neutral-100 text-neutral-400 dark:bg-neutral-950/40"
                          >
                            <span class="text-[9px]">正在捕获首帧...</span>
                          </div>
                        {/if}

                        <!-- 屏幕名称分辨率悬浮条 -->
                        <div
                          class="absolute right-0 bottom-0 left-0 flex items-center justify-between bg-black/60 px-2.5 py-1 text-white backdrop-blur-xs"
                        >
                          <span class="truncate text-[9.5px] font-bold">
                            {monitor.isPrimary ? "主显示器" : `显示器 ${i + 1}`}
                          </span>
                          <span
                            class="font-mono text-[8.5px] font-semibold opacity-90"
                          >
                            {monitor.width} x {monitor.height}
                          </span>
                        </div>

                        <!-- 选中对勾 -->
                        {#if selectedMonitorIndex === i}
                          <div
                            class="absolute top-2 right-2 rounded-full bg-red-600 p-0.5 text-white shadow-xs"
                          >
                            <svg
                              class="size-3.5 fill-none stroke-current stroke-2"
                              viewBox="0 0 24 24"
                            >
                              <polyline points="20 6 9 17 4 12"></polyline>
                            </svg>
                          </div>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>

                <!-- 细分隔线 -->
                <div
                  class="h-px bg-neutral-200/50 dark:bg-neutral-800/40"
                ></div>

                <!-- 参数配置 (横向两栏布局) -->
                <div class="grid grid-cols-2 gap-x-6 gap-y-4">
                  <!-- 左半边：帧率选择 -->
                  <div class="flex flex-col gap-2.5">
                    <span
                      class="text-xs font-bold text-neutral-500 dark:text-neutral-400"
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

                  <!-- 右半边：开关组 -->
                  <div class="flex flex-col gap-3.5">
                    <!-- 麦克风 -->
                    <div class="flex items-center justify-between">
                      <div class="flex flex-col">
                        <span
                          class="text-xs font-semibold text-neutral-800 dark:text-neutral-200"
                          >麦克风音频</span
                        >
                        <span
                          class="text-[9.5px] text-neutral-400 dark:text-neutral-500"
                          >录制外界谈话声音</span
                        >
                      </div>
                      <Switch.Root
                        bind:checked={recordAudio}
                        class="peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border border-transparent transition-colors focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-red-600 data-[state=unchecked]:bg-neutral-200 dark:data-[state=unchecked]:bg-neutral-800"
                      >
                        <Switch.Thumb
                          class="pointer-events-none block h-4 w-4 rounded-full bg-white shadow-md ring-0 transition-transform data-[state=checked]:translate-x-4.5 data-[state=unchecked]:translate-x-0.5 dark:bg-neutral-100"
                        />
                      </Switch.Root>
                    </div>

                    <!-- 系统声卡 -->
                    <div class="flex items-center justify-between">
                      <div class="flex flex-col">
                        <span
                          class="text-xs font-semibold text-neutral-800 dark:text-neutral-200"
                          >系统声卡声音</span
                        >
                        <span
                          class="text-[9.5px] text-neutral-400 dark:text-neutral-500"
                          >录制播放的系统音量</span
                        >
                      </div>
                      <Switch.Root
                        bind:checked={recordSystemSound}
                        class="peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border border-transparent transition-colors focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-red-600 data-[state=unchecked]:bg-neutral-200 dark:data-[state=unchecked]:bg-neutral-800"
                      >
                        <Switch.Thumb
                          class="pointer-events-none block h-4 w-4 rounded-full bg-white shadow-md ring-0 transition-transform data-[state=checked]:translate-x-4.5 data-[state=unchecked]:translate-x-0.5 dark:bg-neutral-100"
                        />
                      </Switch.Root>
                    </div>

                    <!-- 排除自身 -->
                    <div class="flex items-center justify-between">
                      <div class="flex flex-col">
                        <span
                          class="text-xs font-semibold text-neutral-800 dark:text-neutral-200"
                          >排除本程序</span
                        >
                        <span
                          class="text-[9.5px] text-neutral-400 dark:text-neutral-500"
                          >录像时不显示主窗口</span
                        >
                      </div>
                      <Switch.Root
                        bind:checked={excludeOwnWindow}
                        class="peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border border-transparent transition-colors focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-red-600 data-[state=unchecked]:bg-neutral-200 dark:data-[state=unchecked]:bg-neutral-800"
                      >
                        <Switch.Thumb
                          class="pointer-events-none block h-4 w-4 rounded-full bg-white shadow-md ring-0 transition-transform data-[state=checked]:translate-x-4.5 data-[state=unchecked]:translate-x-0.5 dark:bg-neutral-100"
                        />
                      </Switch.Root>
                    </div>
                  </div>
                </div>

                <!-- 细分隔线 -->
                <div
                  class="h-px bg-neutral-200/50 dark:bg-neutral-800/40"
                ></div>

                <!-- 视频保存位置 -->
                <div class="flex flex-col gap-2.5">
                  <span
                    class="text-xs font-bold text-neutral-500 dark:text-neutral-400"
                    >视频保存位置</span
                  >
                  <div class="flex flex-col gap-2">
                    <div
                      class="flex rounded-lg border border-neutral-200/40 bg-neutral-100 p-0.5 dark:border-neutral-800/40 dark:bg-neutral-950/40"
                    >
                      <button
                        class="flex-1 rounded-md py-1.5 text-center text-xs font-semibold transition-all {saveFolderType ===
                        'video'
                          ? 'bg-white text-neutral-900 shadow-xs dark:bg-neutral-800 dark:text-white'
                          : 'text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300'}"
                        onclick={() => handleFolderTypeChange("video")}
                        >视频目录</button
                      >
                      <button
                        class="flex-1 rounded-md py-1.5 text-center text-xs font-semibold transition-all {saveFolderType ===
                        'download'
                          ? 'bg-white text-neutral-900 shadow-xs dark:bg-neutral-800 dark:text-white'
                          : 'text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300'}"
                        onclick={() => handleFolderTypeChange("download")}
                        >下载目录</button
                      >
                      <button
                        class="flex-1 rounded-md py-1.5 text-center text-xs font-semibold transition-all {saveFolderType ===
                        'desktop'
                          ? 'bg-white text-neutral-900 shadow-xs dark:bg-neutral-800 dark:text-white'
                          : 'text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300'}"
                        onclick={() => handleFolderTypeChange("desktop")}
                        >桌面</button
                      >
                      <button
                        class="flex-1 rounded-md py-1.5 text-center text-xs font-semibold transition-all {saveFolderType ===
                        'custom'
                          ? 'bg-white text-neutral-900 shadow-xs dark:bg-neutral-800 dark:text-white'
                          : 'text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300'}"
                        onclick={() => handleFolderTypeChange("custom")}
                        >其他位置</button
                      >
                    </div>

                    {#if saveFolderType === "custom" && customSaveFolder}
                      <div
                        class="flex items-center justify-between rounded-lg border border-neutral-200/30 bg-neutral-100/50 px-3 py-1.5 text-[10px] text-neutral-600 dark:border-neutral-800/30 dark:bg-neutral-950/20 dark:text-neutral-400"
                      >
                        <span
                          class="truncate font-mono"
                          title={customSaveFolder}>{customSaveFolder}</span
                        >
                        <button
                          class="ml-2 shrink-0 cursor-pointer font-semibold text-red-600 hover:text-red-500"
                          onclick={() => handleFolderTypeChange("custom")}
                          >修改</button
                        >
                      </div>
                    {/if}
                  </div>
                </div>
              </div>
            </AppScrollArea>

            <!-- 启动录像按钮 (固定在卡片底部) -->
            <button
              class="mt-4 flex flex-shrink-0 items-center justify-center gap-2 rounded-xl bg-red-600 py-3 text-sm font-semibold text-white shadow-xs transition-all hover:bg-red-500 active:scale-95"
              onclick={handleStartRecord}
            >
              <span class="size-2 animate-pulse rounded-full bg-white"></span>
              开始屏幕录制
            </button>
          </div>
        </Tabs.Content>

        <!-- 2. 历史录像列表界面 -->
        <Tabs.Content
          value="history"
          class="flex h-full justify-center overflow-hidden"
        >
          <div class="flex h-full w-[640px] flex-col">
            {#if isLoading}
              <div
                class="flex flex-grow flex-col items-center justify-center gap-3 text-xs text-neutral-500"
              >
                <Spinner class="size-5 animate-spin text-red-500" />
                <span>正在检索视频文件...</span>
              </div>
            {:else if videos.length === 0}
              <div
                class="flex flex-grow flex-col items-center justify-center rounded-2xl border border-dashed border-neutral-200/80 bg-white/20 p-10 text-neutral-400 dark:border-neutral-800/80 dark:bg-neutral-900/10 dark:text-neutral-500"
              >
                <VideoCamera
                  class="mb-3 size-9 text-neutral-300 dark:text-neutral-700"
                />
                <p class="text-xs font-semibold">当前没有录像记录</p>
                <span class="text-[10px] font-medium text-neutral-400/80"
                  >点击顶部的“开始录屏”开启您的首次录制</span
                >
              </div>
            {:else}
              <!-- 统一自定义滚动条，限制高度为 flex-1 min-h-0 w-full 以启用内部滚动 -->
              <AppScrollArea
                class="min-h-0 w-full flex-1 pr-1"
                viewportClass="h-full w-full"
              >
                <div
                  class="flex flex-col divide-y divide-neutral-200/40 pr-3 pb-4 dark:divide-neutral-800/25"
                >
                  {#each videos as video}
                    <div
                      class="group flex items-center rounded-lg px-3.5 py-2.5 transition-all duration-150 hover:bg-neutral-100/50 dark:hover:bg-neutral-800/30"
                    >
                      <!-- svelte-ignore a11y_click_events_have_key_events -->
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div
                        class="mr-3.5 flex size-8.5 flex-shrink-0 cursor-pointer items-center justify-center rounded-full bg-red-500/10 text-red-500 transition-all hover:bg-red-500 hover:text-white"
                        onclick={() => handlePlay(video)}
                        title="点击播放"
                      >
                        <Play class="size-3.5 fill-current" />
                      </div>
                      <!-- svelte-ignore a11y_click_events_have_key_events -->
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div
                        class="flex min-w-0 flex-grow cursor-pointer flex-col gap-0.5"
                        onclick={() => handlePlay(video)}
                      >
                        <span
                          class="truncate text-xs font-semibold text-neutral-800 dark:text-neutral-100"
                          title={video.name}>{video.name}</span
                        >
                        <div
                          class="flex gap-4.5 font-mono text-[9px] text-neutral-400 dark:text-neutral-500"
                        >
                          <span>时间 {formatDateTime(video.createdTime)}</span>
                          <span>大小 {formatFileSize(video.sizeBytes)}</span>
                        </div>
                      </div>
                      <div
                        class="ml-3 flex-shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
                      >
                        <button
                          class="flex size-6.5 items-center justify-center rounded-md text-neutral-400 transition-colors hover:bg-red-500/10 hover:text-red-500"
                          onclick={() => handleDelete(video)}
                          title="删除视频"
                        >
                          <Trash class="size-3.5" />
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              </AppScrollArea>
            {/if}
          </div>
        </Tabs.Content>
      </div>
    </Tabs.Root>
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
