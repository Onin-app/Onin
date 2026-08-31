<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { platform } from "@tauri-apps/plugin-os";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Switch } from "$lib/components/ui/switch";
  import { Badge } from "$lib/components/ui/badge";
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
      toast.error("加载文件搜索配置失败");
    } finally {
      loading = false;
    }
  }

  async function refreshStatus() {
    try {
      status = await invoke<FileSearchStatus>("get_file_search_status");
    } catch (error) {
      console.error("Failed to refresh file search status:", error);
    }
  }

  async function setEverythingInstallCloseLock(acquire: boolean) {
    if (acquire === everythingInstallCloseLockHeld) {
      return;
    }

    try {
      if (acquire) {
        await invoke("acquire_window_close_lock");
        everythingInstallCloseLockHeld = true;
      } else {
        await invoke("release_window_close_lock");
        everythingInstallCloseLockHeld = false;
      }
    } catch (error) {
      console.error(
        "Failed to update window close lock for Everything install:",
        error,
      );
    }
  }

  async function saveConfig(nextConfig: AppConfig) {
    saving = true;
    try {
      await invoke("update_app_config", {
        config: nextConfig,
      });
      config = nextConfig;
      toast.success("文件搜索配置已保存");
      await refreshStatus();
    } catch (error) {
      console.error("Failed to save file search config:", error);
      toast.error("保存文件搜索配置失败");
    } finally {
      saving = false;
    }
  }

  async function addExcludedPath(pathToAdd: string) {
    const trimmed = pathToAdd.trim();
    if (!trimmed || !config) return;

    const currentPaths = config.file_search_excluded_paths ?? [];
    if (currentPaths.includes(trimmed)) {
      toast.info("该路径已在排除列表中");
      excludeInput = "";
      return;
    }

    const nextPaths = [...currentPaths, trimmed];
    excludeInput = "";
    await saveConfig({
      ...config,
      file_search_excluded_paths: nextPaths,
    });
  }

  async function removeExcludedPath(pathToRemove: string) {
    if (!config) return;

    const nextPaths = (config.file_search_excluded_paths ?? []).filter(
      (path) => path !== pathToRemove,
    );

    await saveConfig({
      ...config,
      file_search_excluded_paths: nextPaths,
    });
  }

  async function installEverything() {
    if (installingEverything) return;

    installingEverything = true;
    try {
      await invoke("install_everything");
      installEverythingDialogOpen = false;
      toast.success(
        "已发起 Everything 安装，请按系统提示完成安装并启动 Everything",
      );
      await refreshStatus();
    } catch (error) {
      console.error("Failed to install Everything:", error);
      toast.error("安装 Everything 失败: " + String(error));
    } finally {
      installingEverything = false;
    }
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
  <div class="text-muted-foreground py-6 text-center text-xs">
    正在加载文件搜索设置...
  </div>
{:else if config}
  <div class="flex flex-col gap-6">
    <section class="border-border/50 border-t pt-4">
      <div class="mb-3 flex items-center justify-between gap-4">
        <div>
          <h4 class="text-foreground text-sm font-semibold tracking-tight">
            排除路径
          </h4>
          <p class="text-muted-foreground/75 mt-0.5 text-xs leading-normal">
            匹配这些路径的文件和目录不会出现在搜索结果中。
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          class="h-8 cursor-pointer gap-1.5 rounded-xl text-xs font-medium transition-[transform,background-color] duration-120 active:scale-95"
          disabled={choosingDirectory}
          onclick={chooseDirectory}
        >
          <PhosphorIcon icon="folderPlus" class="h-4 w-4" />
          选择目录
        </Button>
      </div>

      <div class="mb-3 flex gap-2">
        <Input
          placeholder="例如 C:\Users\name\Downloads\tmp"
          bind:value={excludeInput}
          onkeydown={(event) => {
            if (event.key === "Enter") addExcludedPath(excludeInput);
          }}
          class="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring h-9 rounded-xl border text-xs transition-all focus:ring-1 focus:outline-none"
        />
        <Button
          disabled={!excludeInput.trim()}
          onclick={() => addExcludedPath(excludeInput)}
          class="h-9 cursor-pointer rounded-xl px-4 text-xs font-semibold transition-[transform,background-color] duration-120 active:scale-95"
        >
          添加
        </Button>
      </div>

      <div class="flex flex-col gap-1.5">
        {#each excludedPaths as path (path)}
          <div
            class="bg-muted/40 border-border/50 hover:border-border group flex items-center gap-2.5 rounded-xl border px-3 py-2 text-xs shadow-2xs transition-[border-color,background-color] duration-120"
          >
            <PhosphorIcon
              icon="prohibit"
              class="text-muted-foreground h-3.5 w-3.5 shrink-0"
            />
            <span
              class="text-foreground min-w-0 flex-1 truncate font-mono text-[11px]"
            >
              {path}
            </span>
            <Button
              variant="ghost"
              size="icon"
              class="text-muted-foreground hover:text-destructive h-6 w-6 cursor-pointer rounded-md opacity-0 transition-[opacity,transform] duration-120 group-hover:opacity-100 active:scale-90"
              title="移除"
              onclick={() => removeExcludedPath(path)}
            >
              <PhosphorIcon icon="trash" class="h-3.5 w-3.5" />
            </Button>
          </div>
        {:else}
          <div class="text-muted-foreground/75 py-2 text-xs">
            暂无额外排除路径
          </div>
        {/each}
      </div>
    </section>

    <section
      class="border-border/50 flex items-center justify-between gap-4 border-t pt-4"
    >
      <div>
        <h4 class="text-foreground text-sm font-semibold tracking-tight">
          隐藏文件
        </h4>
        <p class="text-muted-foreground/75 mt-0.5 text-xs leading-normal">
          开启后会显示以点号开头的文件和目录。
        </p>
      </div>
      <Switch
        checked={config.file_search_include_hidden ?? false}
        onCheckedChange={updateIncludeHidden}
      />
    </section>

    <section class="border-border/50 border-t pt-4">
      <div>
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <h4 class="text-foreground text-sm font-semibold tracking-tight">
              搜索后端
            </h4>
            <p class="text-muted-foreground/75 mt-0.5 text-xs leading-normal">
              {backendDescription}
            </p>
          </div>
          {#if isWindows && status.everything_install_required}
            <Button
              size="sm"
              class="h-8 shrink-0 cursor-pointer gap-1.5 rounded-xl text-xs font-semibold transition-[transform,background-color] duration-120 active:scale-95"
              disabled={installingEverything}
              onclick={() => (installEverythingDialogOpen = true)}
            >
              <PhosphorIcon icon="download" class="h-4 w-4" />
              安装 Everything
            </Button>
          {/if}
        </div>

        <div class="mt-3 flex flex-wrap items-center gap-2">
          <Badge
            variant="secondary"
            class="border-border/40 gap-1.5 rounded-lg border px-2.5 py-1 text-xs font-normal"
          >
            <span
              class="h-1.5 w-1.5 rounded-full {status.is_searching
                ? 'animate-pulse bg-amber-500'
                : status.available
                  ? 'bg-emerald-500'
                  : 'bg-destructive'}"
            ></span>
            <span>
              当前：{status.backend || "系统搜索"} · {status.available
                ? "可用"
                : "不可用"}
            </span>
          </Badge>
          {#if isWindows}
            <Badge
              variant="outline"
              class="border-border/40 gap-1.5 rounded-lg border px-2.5 py-1 text-xs font-normal"
            >
              <span
                class="h-1.5 w-1.5 rounded-full {status.everything_installed
                  ? status.everything_ipc_available
                    ? 'bg-emerald-500'
                    : 'bg-amber-500'
                  : 'bg-muted-foreground'}"
              ></span>
              <span>
                Everything：{status.everything_installed
                  ? status.everything_ipc_available
                    ? "已连接"
                    : "已安装，等待后台启动"
                  : "未安装"}
              </span>
            </Badge>
          {/if}
        </div>
        {#if isWindows && status.everything_install_required}
          <p class="text-muted-foreground/75 mt-2 text-xs">
            当前会继续使用 Windows Search。安装 Everything
            后可获得更快的全盘文件名搜索。
          </p>
        {/if}
        {#if status.last_error}
          <p class="text-destructive mt-2 text-xs font-medium">
            {status.last_error}
          </p>
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
