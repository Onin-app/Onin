<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Switch, Button } from "bits-ui";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import { toast } from "svelte-sonner";
  import {
    CloudArrowUp,
    CloudArrowDown,
    CheckCircle,
    XCircle,
    Spinner,
    ArrowsClockwise,
    Cloud,
  } from "phosphor-svelte";
  import type { AppConfig, WebDavConfig } from "$lib/type";
  import SetItem from "./SetItem.svelte";
  import PasswordInput from "$lib/components/PasswordInput.svelte";

  // 云端备份元数据接口
  interface LastSyncInfo {
    last_sync_time: string;
    device_id: string;
  }

  // 状态管理，Svelte 5 状态绑定
  let enabled = $state<boolean>(false);
  let baseUrl = $state<string>("");
  let username = $state<string>("");
  let password = $state<string>("");
  let folderName = $state<string>("");
  let syncOnStartup = $state<boolean>(false);
  let syncOnExit = $state<boolean>(false);

  // 交互状态
  let testingConnection = $state<boolean>(false);
  let testSuccess = $state<boolean | null>(null);
  let syncing = $state<boolean>(false);
  let syncMode = $state<"backup" | "restore" | null>(null);
  let cloudBackupInfo = $state<LastSyncInfo | null>(null);
  let checkingBackup = $state<boolean>(false);

  // 格式化时间戳
  const formatTime = (timeStr?: string) => {
    if (!timeStr) return "暂无同步";
    try {
      const date = new Date(timeStr);
      return date.toLocaleString();
    } catch {
      return timeStr;
    }
  };

  // 加载配置
  const loadWebDavConfig = async () => {
    try {
      const config = await invoke<AppConfig>("get_app_config");
      if (config.webdav) {
        enabled = config.webdav.enabled;
        baseUrl = config.webdav.base_url;
        username = config.webdav.username;
        password = config.webdav.password;
        folderName = config.webdav.folder_name || "";
        syncOnStartup = config.webdav.sync_on_startup;
        syncOnExit = config.webdav.sync_on_exit;
      }
    } catch (e) {
      console.error("加载 WebDAV 配置失败:", e);
      toast.error("加载 WebDAV 配置失败");
    }
  };

  // 组合当前的配置对象
  const getCurrentWebDavConfig = (): WebDavConfig => {
    return {
      enabled,
      base_url: baseUrl,
      username,
      password,
      sync_on_startup: syncOnStartup,
      sync_on_exit: syncOnExit,
      folder_name: folderName,
    };
  };

  // 更新配置并持久化保存
  const updateConfig = async () => {
    try {
      // 获取当前最新的整个应用配置
      const config = await invoke<AppConfig>("get_app_config");
      // 覆写其中的 webdav 选项
      config.webdav = getCurrentWebDavConfig();

      await invoke("update_app_config", { config });
    } catch (error) {
      console.error("保存 WebDAV 配置失败:", error);
      toast.error("保存配置失败");
    }
  };

  // 测试 WebDAV 连接
  const testConnection = async () => {
    if (testingConnection) return;
    if (!baseUrl || !username || !password) {
      toast.warning("请填写完整的 WebDAV 服务器配置");
      return;
    }

    testingConnection = true;
    testSuccess = null;

    try {
      const config = getCurrentWebDavConfig();
      await invoke("test_webdav_connection", { config });
      testSuccess = true;
      toast.success("WebDAV 连接测试成功");
      // 成功后自动拉取一次云端备份信息
      checkCloudBackup();
    } catch (e) {
      console.error("WebDAV 连接测试失败:", e);
      testSuccess = false;
      toast.error(`连接测试失败: ${e}`);
    } finally {
      testingConnection = false;
    }
  };

  // 检查云端备份
  const checkCloudBackup = async () => {
    if (checkingBackup) return;
    if (!baseUrl || !username || !password) return;

    checkingBackup = true;
    try {
      const config = getCurrentWebDavConfig();
      const backupInfo = await invoke<LastSyncInfo | null>(
        "check_cloud_backup",
        { config },
      );
      cloudBackupInfo = backupInfo;
    } catch (e) {
      console.error("检查云端备份失败:", e);
    } finally {
      checkingBackup = false;
    }
  };

  // 执行同步 (备份或恢复)
  const executeSync = async (mode: "backup" | "restore") => {
    if (syncing) return;
    if (!baseUrl || !username || !password) {
      toast.warning("请先填写并保存 WebDAV 配置");
      return;
    }

    // 在同步前，先确保当前最新配置已保存到本地
    await updateConfig();

    syncing = true;
    syncMode = mode;

    const actionText = mode === "backup" ? "备份" : "恢复";
    const loadingToastId = toast.loading(`正在执行数据${actionText}...`);

    try {
      const result = await invoke<LastSyncInfo | null>("trigger_webdav_sync", {
        mode,
      });
      if (result) {
        cloudBackupInfo = result;
      }
      toast.dismiss(loadingToastId);
      toast.success(`数据${actionText}成功！`);

      // 如果是恢复，拉取本地最新配置，这样前端UI能立刻显示恢复后的状态
      if (mode === "restore") {
        await loadWebDavConfig();
      }
    } catch (e) {
      console.error(`数据${actionText}失败:`, e);
      toast.dismiss(loadingToastId);
      toast.error(`数据${actionText}失败: ${e}`);
    } finally {
      syncing = false;
      syncMode = null;
    }
  };

  // 初始化拉取
  onMount(async () => {
    await loadWebDavConfig();
    if (enabled) {
      checkCloudBackup();
    }
  });
</script>

<AppScrollArea class="h-full w-full" viewportClass="h-full w-full">
  <main class="h-full w-full pr-2 pb-8">
    <!-- 启用同步 -->
    <section class="mb-6">
      <h2
        class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
      >
        数据同步
      </h2>
      <div
        class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
      >
        <SetItem
          title="启用 WebDAV 数据同步"
          description="将您的本地数据打包上传至 WebDAV 云盘，实现跨设备多端配置同步"
        >
          {#snippet content()}
            <Switch.Root
              bind:checked={enabled}
              onCheckedChange={updateConfig}
              class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-neutral-900 data-[state=unchecked]:bg-neutral-200 dark:focus-visible:ring-neutral-300 dark:data-[state=checked]:bg-neutral-50 dark:data-[state=unchecked]:bg-neutral-700"
            >
              <Switch.Thumb
                class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0 dark:bg-neutral-950"
              />
            </Switch.Root>
          {/snippet}
        </SetItem>
      </div>
    </section>

    {#if enabled}
      <!-- WebDAV 服务器配置 -->
      <section class="mb-6">
        <h2
          class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
        >
          服务器配置
        </h2>
        <div
          class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <!-- 地址 -->
          <SetItem
            title="WebDAV 服务器地址"
            description="例如坚果云: https://dav.jianguoyun.com/dav/"
          >
            {#snippet content()}
              <input
                type="text"
                bind:value={baseUrl}
                onchange={updateConfig}
                placeholder="https://..."
                class="h-8 w-80 rounded-md border border-neutral-200 bg-transparent px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:focus:border-neutral-100"
              />
            {/snippet}
          </SetItem>

          <!-- 用户名 -->
          <SetItem title="账号 / 用户名">
            {#snippet content()}
              <input
                type="text"
                bind:value={username}
                onchange={updateConfig}
                placeholder="用户名/邮箱"
                class="h-8 w-64 rounded-md border border-neutral-200 bg-transparent px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:focus:border-neutral-100"
              />
            {/snippet}
          </SetItem>

          <!-- 密码 -->
          <SetItem
            title="应用密码 / 授权密钥"
            description="出于安全考虑，推荐使用网盘生成的应用独立密码"
          >
            {#snippet content()}
              <PasswordInput
                bind:value={password}
                onchange={updateConfig}
                placeholder="应用授权密钥"
                class="h-8 w-64 rounded-md bg-transparent"
              />
            {/snippet}
          </SetItem>

          <!-- 云端同步目录 -->
          <SetItem
            title="云端同步目录"
            description="支持自定义单级目录名称（如 onin-work, onin-home）实现不同电脑配置隔离，默认使用 onin"
          >
            {#snippet content()}
              <input
                type="text"
                bind:value={folderName}
                onchange={updateConfig}
                placeholder="onin"
                class="h-8 w-64 rounded-md border border-neutral-200 bg-transparent px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:focus:border-neutral-100"
              />
            {/snippet}
          </SetItem>

          <!-- 连接测试 -->
          <SetItem title="连接状态">
            {#snippet content()}
              <div class="flex items-center gap-3">
                {#if testSuccess === true}
                  <span
                    class="flex items-center gap-1 text-xs text-green-600 dark:text-green-400"
                  >
                    <CheckCircle class="h-4 w-4" /> 连接成功
                  </span>
                {:else if testSuccess === false}
                  <span
                    class="flex items-center gap-1 text-xs text-red-600 dark:text-red-400"
                  >
                    <XCircle class="h-4 w-4" /> 连接失败
                  </span>
                {/if}
                <Button.Root
                  class="inline-flex h-8 items-center justify-center rounded-md border border-neutral-200 bg-white px-3 text-xs font-semibold text-neutral-900 shadow-sm transition-colors hover:bg-neutral-100 hover:text-neutral-900 focus-visible:ring-1 focus-visible:ring-neutral-950 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-50 dark:hover:bg-neutral-800 dark:hover:text-neutral-50 dark:focus-visible:ring-neutral-300"
                  onclick={testConnection}
                  disabled={testingConnection}
                >
                  {#if testingConnection}
                    <Spinner class="mr-1 h-3.5 w-3.5 animate-spin" /> 测试中...
                  {:else}
                    测试连接
                  {/if}
                </Button.Root>
              </div>
            {/snippet}
          </SetItem>
        </div>
      </section>

      <!-- 同步策略配置 -->
      <section class="mb-6">
        <h2
          class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
        >
          自动同步策略
        </h2>
        <div
          class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <SetItem
            title="开机启动时自动下载同步"
            description="应用启动时将自动检测云端更新并拉取同步最新配置"
          >
            {#snippet content()}
              <Switch.Root
                bind:checked={syncOnStartup}
                onCheckedChange={updateConfig}
                class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-neutral-900 data-[state=unchecked]:bg-neutral-200 dark:focus-visible:ring-neutral-300 dark:data-[state=checked]:bg-neutral-50 dark:data-[state=unchecked]:bg-neutral-700"
              >
                <Switch.Thumb
                  class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0 dark:bg-neutral-950"
                />
              </Switch.Root>
            {/snippet}
          </SetItem>

          <SetItem
            title="退出应用时自动上传备份"
            description="应用退出或关机前，会自动把最新的应用配置打包备份到云端"
          >
            {#snippet content()}
              <Switch.Root
                bind:checked={syncOnExit}
                onCheckedChange={updateConfig}
                class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-neutral-900 data-[state=unchecked]:bg-neutral-200 dark:focus-visible:ring-neutral-300 dark:data-[state=checked]:bg-neutral-50 dark:data-[state=unchecked]:bg-neutral-700"
              >
                <Switch.Thumb
                  class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0 dark:bg-neutral-950"
                />
              </Switch.Root>
            {/snippet}
          </SetItem>
        </div>
      </section>

      <!-- 备份与恢复执行面板 -->
      <section class="mb-6">
        <h2
          class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
        >
          数据手动同步
        </h2>
        <div
          class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <!-- 上次同步状态显示 -->
          <SetItem title="云端备份状态">
            {#snippet content()}
              <div class="flex items-center gap-3">
                <div class="text-right text-xs">
                  {#if checkingBackup}
                    <span
                      class="flex items-center justify-end gap-1 text-neutral-400"
                    >
                      <Spinner class="h-3 w-3 animate-spin" /> 检测云端数据...
                    </span>
                  {:else if cloudBackupInfo}
                    <div class="text-neutral-600 dark:text-neutral-300">
                      <span
                        class="font-medium text-neutral-800 dark:text-neutral-200"
                        >上次备份:</span
                      >
                      {formatTime(cloudBackupInfo.last_sync_time)}
                    </div>
                    <div class="text-[10px] text-neutral-400">
                      <span>设备:</span>
                      <span class="font-mono"
                        >{cloudBackupInfo.device_id || "未知"}</span
                      >
                    </div>
                  {:else}
                    <span class="text-neutral-400">云端未检测到备份文件</span>
                  {/if}
                </div>
                <Button.Root
                  class="inline-flex h-8 items-center justify-center rounded-md border border-neutral-200 bg-white px-2.5 text-xs font-semibold text-neutral-900 shadow-sm transition-colors hover:bg-neutral-100 hover:text-neutral-900 focus-visible:ring-1 focus-visible:ring-neutral-950 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-50 dark:hover:bg-neutral-800 dark:hover:text-neutral-50 dark:focus-visible:ring-neutral-300"
                  onclick={checkCloudBackup}
                  disabled={checkingBackup}
                  title="刷新云端备份状态"
                >
                  <ArrowsClockwise
                    class="h-3.5 w-3.5 {checkingBackup ? 'animate-spin' : ''}"
                  />
                </Button.Root>
              </div>
            {/snippet}
          </SetItem>

          <!-- 触发按钮 -->
          <SetItem
            title="手动同步操作"
            description="手动触发云端备份上传，或从云端拉取覆盖本地数据"
          >
            {#snippet content()}
              <div class="flex gap-2">
                <!-- 立即备份 -->
                <Button.Root
                  class="inline-flex h-8 items-center justify-center rounded-md bg-neutral-900 px-3 text-xs font-semibold text-white shadow-sm transition-colors hover:bg-neutral-800 focus-visible:ring-1 focus-visible:ring-neutral-950 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50 dark:bg-neutral-50 dark:text-neutral-900 dark:hover:bg-neutral-200 dark:focus-visible:ring-neutral-300"
                  onclick={() => executeSync("backup")}
                  disabled={syncing}
                >
                  {#if syncing && syncMode === "backup"}
                    <Spinner class="mr-1 h-3.5 w-3.5 animate-spin" /> 备份中...
                  {:else}
                    <CloudArrowUp class="mr-1 h-3.5 w-3.5" /> 立即备份 (上传)
                  {/if}
                </Button.Root>

                <!-- 立即恢复 -->
                <Button.Root
                  class="inline-flex h-8 items-center justify-center rounded-md border border-neutral-200 bg-white px-3 text-xs font-semibold text-neutral-900 shadow-sm transition-colors hover:bg-neutral-100 hover:text-neutral-900 focus-visible:ring-1 focus-visible:ring-neutral-950 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-50 dark:hover:bg-neutral-800 dark:hover:text-neutral-50 dark:focus-visible:ring-neutral-300"
                  onclick={() => executeSync("restore")}
                  disabled={syncing}
                >
                  {#if syncing && syncMode === "restore"}
                    <Spinner class="mr-1 h-3.5 w-3.5 animate-spin" /> 恢复中...
                  {:else}
                    <CloudArrowDown class="mr-1 h-3.5 w-3.5" /> 立即同步 (下载)
                  {/if}
                </Button.Root>
              </div>
            {/snippet}
          </SetItem>
        </div>
      </section>
    {/if}
  </main>
</AppScrollArea>
