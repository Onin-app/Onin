<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { platform } from "@tauri-apps/plugin-os";
  import { Switch } from "bits-ui";
  import { toast } from "svelte-sonner";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  import type { AppConfig } from "$lib/type";

  interface FileSearchStatus {
    is_searching: boolean;
    last_result_count: number;
    backend: string;
    everything_installed: boolean;
    everything_ipc_available: boolean;
    everything_install_required: boolean;
    available: boolean;
    last_error?: string | null;
  }

  let config = $state<AppConfig | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let choosingDirectory = $state(false);
  let installingEverything = $state(false);
  let installEverythingDialogOpen = $state(false);
  let everythingInstallCloseLockHeld = false;
  let excludeInput = $state("");
  let currentPlatform = $state("");
  let status = $state<FileSearchStatus>({
    is_searching: false,
    last_result_count: 0,
    backend: "",
    everything_installed: false,
    everything_ipc_available: false,
    everything_install_required: false,
    available: true,
    last_error: null,
  });
  const excludedPaths = $derived(config?.file_search_excluded_paths ?? []);
  const isWindows = $derived(currentPlatform === "windows");
  const backendDescription = $derived(
    getBackendDescription(currentPlatform, status.backend),
  );

  function getBackendDescription(platformName: string, backend: string) {
    if (platformName === "windows") {
      return "Windows 可安装 Everything 获取实时索引；未安装或不可用时使用 Windows Search。";
    }

    if (platformName === "macos") {
      return "macOS 使用 Spotlight 索引进行文件名搜索。";
    }

    if (platformName === "linux") {
      return "Linux 使用 locate/plocate 数据库进行文件名搜索。";
    }

    return backend
      ? `当前平台使用 ${backend} 进行文件名搜索。`
      : "当前平台使用可用的系统文件搜索后端。";
  }

  function loadPlatform() {
    try {
      currentPlatform = platform();
    } catch (error) {
      console.error("Failed to detect platform:", error);
      currentPlatform = "";
    }
  }

  async function loadConfig() {
    loading = true;
    try {
      config = await invoke<AppConfig>("get_app_config");
    } catch (error) {
      console.error("Failed to load file search config:", error);
      toast.error("加载文件搜索设置失败");
    } finally {
      loading = false;
    }
  }

  async function refreshStatus() {
    try {
      status = await invoke<FileSearchStatus>("get_file_search_status");
    } catch (error) {
      console.error("Failed to load file search status:", error);
    }
  }

  async function setEverythingInstallCloseLock(locked: boolean) {
    if (locked === everythingInstallCloseLockHeld) return;

    everythingInstallCloseLockHeld = locked;
    await invoke(
      locked ? "acquire_window_close_lock" : "release_window_close_lock",
    ).catch((error) => {
      everythingInstallCloseLockHeld = !locked;
      console.error("Failed to update Everything install window lock:", error);
    });
  }

  async function installEverything() {
    if (installingEverything) return;

    installingEverything = true;
    const toastId = toast.loading("正在安装 Everything...");
    try {
      await invoke("install_file_search_everything");
      await refreshStatus();
      installEverythingDialogOpen = false;
      toast.success("Everything 已安装，文件搜索将优先使用 Everything", {
        id: toastId,
      });
    } catch (error) {
      console.error("Failed to install Everything:", error);
      installEverythingDialogOpen = false;
      toast.error(String(error), { id: toastId });
    } finally {
      installingEverything = false;
    }
  }

  async function saveConfig(nextConfig: AppConfig) {
    if (saving) return;

    saving = true;
    try {
      await invoke("update_app_config", { config: nextConfig });
      config = nextConfig;
      await refreshStatus();
      toast.success("文件搜索设置已保存");
    } catch (error) {
      console.error("Failed to save file search config:", error);
      toast.error("保存文件搜索设置失败");
    } finally {
      saving = false;
    }
  }

  function normalizedList(paths: string[]) {
    return Array.from(
      new Set(paths.map((path) => path.trim()).filter(Boolean)),
    );
  }

  async function addExcludedPath(path: string) {
    if (!config) return;
    const nextExcludedPaths = normalizedList([...excludedPaths, path]);
    await saveConfig({
      ...config,
      file_search_excluded_paths: nextExcludedPaths,
    });
    excludeInput = "";
  }

  async function removeExcludedPath(path: string) {
    if (!config) return;
    await saveConfig({
      ...config,
      file_search_excluded_paths: excludedPaths.filter(
        (excludedPath) => excludedPath !== path,
      ),
    });
  }

  async function chooseDirectory() {
    if (choosingDirectory) return;

    choosingDirectory = true;
    await invoke("acquire_window_close_lock");
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (!selected || Array.isArray(selected)) {
        return;
      }

      await addExcludedPath(selected);
    } catch (error) {
      console.error("Failed to choose file search directory:", error);
      toast.error("选择目录失败");
    } finally {
      choosingDirectory = false;
      await invoke("release_window_close_lock");
    }
  }

  async function updateIncludeHidden(includeHidden: boolean) {
    if (!config) return;
    await saveConfig({
      ...config,
      file_search_include_hidden: includeHidden,
    });
  }

  onMount(async () => {
    loadPlatform();
    await Promise.all([loadConfig(), refreshStatus()]);
  });

  $effect(() => {
    void setEverythingInstallCloseLock(
      installEverythingDialogOpen || installingEverything,
    );
  });
</script>

{#if loading}
  <div class="py-3 text-sm text-neutral-500 dark:text-neutral-400">
    正在加载文件搜索设置...
  </div>
{:else if config}
  <div class="flex flex-col gap-5">
    <section class="border-t border-neutral-100 pt-4 dark:border-neutral-800">
      <div class="mb-3 flex items-center justify-between gap-4">
        <div>
          <h4
            class="text-sm font-semibold text-neutral-950 dark:text-neutral-50"
          >
            排除路径
          </h4>
          <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
            匹配这些路径的文件和目录不会出现在搜索结果中。
          </p>
        </div>
        <button
          class="inline-flex items-center gap-1.5 rounded-md border border-neutral-200 px-2.5 py-1.5 text-xs font-medium text-neutral-700 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:text-neutral-200 dark:hover:bg-neutral-800"
          disabled={choosingDirectory}
          onclick={chooseDirectory}
        >
          <PhosphorIcon icon="folderPlus" class="h-4 w-4" />
          选择目录
        </button>
      </div>

      <div class="mb-3 flex gap-2">
        <input
          class="min-w-0 flex-1 rounded-md border border-neutral-200 bg-white px-3 py-2 text-sm outline-none focus:border-neutral-400 dark:border-neutral-700 dark:bg-neutral-950 dark:focus:border-neutral-500"
          placeholder="C:\Users\name\Downloads\tmp"
          bind:value={excludeInput}
          onkeydown={(event) => {
            if (event.key === "Enter") addExcludedPath(excludeInput);
          }}
        />
        <button
          class="rounded-md bg-neutral-900 px-3 py-2 text-sm font-medium text-white disabled:cursor-not-allowed disabled:opacity-50 dark:bg-neutral-100 dark:text-neutral-900"
          disabled={!excludeInput.trim()}
          onclick={() => addExcludedPath(excludeInput)}
        >
          添加
        </button>
      </div>

      <div class="flex flex-col gap-2">
        {#each excludedPaths as path (path)}
          <div
            class="flex items-center gap-2 rounded-md bg-neutral-100 px-3 py-2 text-sm dark:bg-neutral-800"
          >
            <PhosphorIcon icon="prohibit" class="h-4 w-4 shrink-0" />
            <span class="min-w-0 flex-1 truncate font-mono text-xs">
              {path}
            </span>
            <button
              class="rounded p-1 text-neutral-500 hover:bg-neutral-200 hover:text-neutral-900 dark:hover:bg-neutral-700 dark:hover:text-neutral-100"
              title="移除"
              onclick={() => removeExcludedPath(path)}
            >
              <PhosphorIcon icon="trash" class="h-4 w-4" />
            </button>
          </div>
        {:else}
          <div class="text-sm text-neutral-500 dark:text-neutral-400">
            暂无额外排除路径
          </div>
        {/each}
      </div>
    </section>

    <section
      class="flex items-center justify-between gap-4 border-t border-neutral-100 pt-4 dark:border-neutral-800"
    >
      <div>
        <h4 class="text-sm font-semibold text-neutral-950 dark:text-neutral-50">
          隐藏文件
        </h4>
        <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
          开启后会显示以点号开头的文件和目录。
        </p>
      </div>
      <Switch.Root
        checked={config.file_search_include_hidden ?? false}
        onCheckedChange={updateIncludeHidden}
        class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2 focus-visible:outline-hidden data-[state=checked]:bg-neutral-900 data-[state=unchecked]:bg-neutral-200 dark:focus-visible:ring-neutral-300 dark:data-[state=checked]:bg-neutral-50 dark:data-[state=unchecked]:bg-neutral-700"
      >
        <Switch.Thumb
          class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0 dark:bg-neutral-950"
        />
      </Switch.Root>
    </section>

    <section class="border-t border-neutral-100 pt-4 dark:border-neutral-800">
      <div>
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <h4
              class="text-sm font-semibold text-neutral-950 dark:text-neutral-50"
            >
              搜索后端
            </h4>
            <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
              {backendDescription}
            </p>
          </div>
          {#if isWindows && status.everything_install_required}
            <button
              class="inline-flex shrink-0 items-center gap-1.5 rounded-md bg-neutral-900 px-2.5 py-1.5 text-xs font-medium text-white transition-colors hover:bg-neutral-700 disabled:cursor-not-allowed disabled:opacity-60 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-300"
              disabled={installingEverything}
              onclick={() => (installEverythingDialogOpen = true)}
            >
              <PhosphorIcon icon="download" class="h-4 w-4" />
              安装 Everything
            </button>
          {/if}
        </div>

        <div class="mt-3 flex flex-wrap items-center gap-2">
          <div
            class="inline-flex items-center gap-1.5 rounded-md border border-neutral-200 bg-neutral-50 px-2 py-1 text-xs text-neutral-600 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300"
          >
            <span
              class="h-1.5 w-1.5 rounded-full {status.is_searching
                ? 'animate-pulse bg-amber-500'
                : status.available
                  ? 'bg-emerald-500'
                  : 'bg-red-500'}"
            ></span>
            <span>
              当前：{status.backend || "系统搜索"} · {status.available
                ? "可用"
                : "不可用"}
            </span>
          </div>
          {#if isWindows}
            <div
              class="inline-flex items-center gap-1.5 rounded-md border border-neutral-200 bg-white px-2 py-1 text-xs text-neutral-600 dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-300"
            >
              <span
                class="h-1.5 w-1.5 rounded-full {status.everything_installed
                  ? status.everything_ipc_available
                    ? 'bg-emerald-500'
                    : 'bg-amber-500'
                  : 'bg-neutral-300 dark:bg-neutral-600'}"
              ></span>
              <span>
                Everything：{status.everything_installed
                  ? status.everything_ipc_available
                    ? "已连接"
                    : "已安装，等待后台启动"
                  : "未安装"}
              </span>
            </div>
          {/if}
        </div>
        {#if isWindows && status.everything_install_required}
          <p class="mt-2 text-xs text-neutral-500 dark:text-neutral-400">
            当前会继续使用 Windows Search。安装 Everything
            后可获得更快的全盘文件名搜索。
          </p>
        {/if}
        {#if status.last_error}
          <p class="mt-2 text-xs text-red-500">{status.last_error}</p>
        {/if}
      </div>
    </section>
  </div>
{/if}

{#if isWindows}
  <ConfirmDialog
    bind:open={installEverythingDialogOpen}
    title="安装 Everything 加速文件搜索"
    description="Onin 会通过 winget 安装 Everything，并优先使用 Everything IPC 获取实时文件搜索结果。未安装时仍会继续使用 Windows Search。"
    confirmLabel="安装"
    cancelLabel="暂不安装"
    variant="default"
    loading={installingEverything}
    closeOnConfirm={false}
    onConfirm={installEverything}
    onCancel={() => {
      if (!installingEverything) {
        installEverythingDialogOpen = false;
      }
    }}
  />
{/if}
