<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Switch } from "$lib/components/ui/switch";
  import { Card } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { toast } from "svelte-sonner";
  import {
    CloudArrowUp,
    CloudArrowDown,
    CheckCircle,
    XCircle,
    Spinner,
    ArrowsClockwise,
  } from "phosphor-svelte";
  import type { AppConfig, WebDavConfig } from "$lib/type";
  import SetItem from "./SetItem.svelte";
  import PasswordInput from "$lib/components/PasswordInput.svelte";

  interface LastSyncInfo {
    last_sync_time: string;
    device_id: string;
  }

  let enabled = $state<boolean>(false);
  let baseUrl = $state<string>("");
  let username = $state<string>("");
  let password = $state<string>("");
  let folderName = $state<string>("");
  let syncOnStartup = $state<boolean>(false);
  let syncOnExit = $state<boolean>(false);

  let testingConnection = $state<boolean>(false);
  let testSuccess = $state<boolean | null>(null);
  let syncing = $state<boolean>(false);
  let syncMode = $state<"backup" | "restore" | null>(null);
  let cloudBackupInfo = $state<LastSyncInfo | null>(null);
  let checkingBackup = $state<boolean>(false);

  const formatTime = (timeStr?: string) => {
    if (!timeStr) return "暂无同步";
    try {
      const date = new Date(timeStr);
      return date.toLocaleString();
    } catch {
      return timeStr;
    }
  };

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

  const updateConfig = async () => {
    try {
      const config = await invoke<AppConfig>("get_app_config");
      config.webdav = getCurrentWebDavConfig();
      await invoke("update_app_config", { config });
    } catch (error) {
      console.error("保存 WebDAV 配置失败:", error);
      toast.error("保存配置失败");
    }
  };

  const testConnection = async () => {
    if (!baseUrl || !username || !password) {
      toast.error("请先填写完整的服务器地址、账号和密码");
      return;
    }

    testingConnection = true;
    testSuccess = null;

    try {
      await updateConfig();
      await invoke("test_webdav_connection");
      testSuccess = true;
      toast.success("WebDAV 连接成功！");
      checkCloudBackup();
    } catch (error) {
      console.error("测试 WebDAV 连接失败:", error);
      testSuccess = false;
      toast.error("连接失败: " + String(error));
    } finally {
      testingConnection = false;
    }
  };

  const checkCloudBackup = async () => {
    if (!baseUrl || !username || !password) return;

    checkingBackup = true;
    try {
      const info = await invoke<LastSyncInfo | null>(
        "check_cloud_backup_metadata",
      );
      cloudBackupInfo = info;
    } catch (error) {
      console.error("检测云端备份元数据失败:", error);
    } finally {
      checkingBackup = false;
    }
  };

  const executeSync = async (mode: "backup" | "restore") => {
    if (!baseUrl || !username || !password) {
      toast.error("请先完善 WebDAV 连接配置");
      return;
    }

    syncing = true;
    syncMode = mode;

    try {
      await updateConfig();

      if (mode === "backup") {
        toast.info("正在将本地数据打包备份到云端...");
        await invoke("sync_backup_to_webdav");
        toast.success("云端备份成功！");
        await checkCloudBackup();
      } else {
        toast.info("正在从云端拉取配置并恢复...");
        await invoke("sync_restore_from_webdav");
        toast.success("配置恢复成功！部分改动可能需要重启应用后生效。");
      }
    } catch (error) {
      console.error(`WebDAV 同步失败 (${mode}):`, error);
      toast.error(`同步失败: ` + String(error));
    } finally {
      syncing = false;
      syncMode = null;
    }
  };

  onMount(() => {
    loadWebDavConfig().then(() => {
      if (enabled && baseUrl && username && password) {
        checkCloudBackup();
      }
    });
  });
</script>

<ScrollArea class="h-full w-full" viewportClass="h-full w-full">
  <main class="h-full w-full pr-2 pb-8">
    <!-- 启用同步 -->
    <section class="mb-6">
      <h2
        class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
      >
        数据同步
      </h2>
      <Card class="px-4 py-1">
        <SetItem
          title="启用 WebDAV 数据同步"
          description="将您的本地数据打包上传至 WebDAV 云盘，实现跨设备多端配置同步"
        >
          {#snippet content()}
            <Switch bind:checked={enabled} onCheckedChange={updateConfig} />
          {/snippet}
        </SetItem>
      </Card>
    </section>

    {#if enabled}
      <!-- WebDAV 服务器配置 -->
      <section class="mb-6">
        <h2
          class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
        >
          服务器配置
        </h2>
        <Card class="px-4 py-1">
          <!-- 地址 -->
          <SetItem
            title="WebDAV 服务器地址"
            description="例如坚果云: https://dav.jianguoyun.com/dav/"
          >
            {#snippet content()}
              <Input
                type="text"
                bind:value={baseUrl}
                onchange={updateConfig}
                placeholder="https://..."
                class="h-8 w-80 text-sm"
              />
            {/snippet}
          </SetItem>

          <!-- 用户名 -->
          <SetItem title="账号 / 用户名">
            {#snippet content()}
              <Input
                type="text"
                bind:value={username}
                onchange={updateConfig}
                placeholder="用户名/邮箱"
                class="h-8 w-64 text-sm"
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
                class="h-8 w-64 bg-transparent"
              />
            {/snippet}
          </SetItem>

          <!-- 云端同步目录 -->
          <SetItem
            title="云端同步目录"
            description="支持自定义单级目录名称（如 onin-work, onin-home）实现不同电脑配置隔离，默认使用 onin"
          >
            {#snippet content()}
              <Input
                type="text"
                bind:value={folderName}
                onchange={updateConfig}
                placeholder="onin"
                class="h-8 w-64 text-sm"
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
                    class="text-destructive flex items-center gap-1 text-xs"
                  >
                    <XCircle class="h-4 w-4" /> 连接失败
                  </span>
                {/if}
                <Button
                  variant="outline"
                  size="sm"
                  onclick={testConnection}
                  disabled={testingConnection}
                >
                  {#if testingConnection}
                    <Spinner class="mr-1 h-3.5 w-3.5 animate-spin" /> 测试中...
                  {:else}
                    测试连接
                  {/if}
                </Button>
              </div>
            {/snippet}
          </SetItem>
        </Card>
      </section>

      <!-- 同步策略配置 -->
      <section class="mb-6">
        <h2
          class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
        >
          自动同步策略
        </h2>
        <Card class="px-4 py-1">
          <SetItem
            title="开机启动时自动下载同步"
            description="应用启动时将自动检测云端更新并拉取同步最新配置"
          >
            {#snippet content()}
              <Switch
                bind:checked={syncOnStartup}
                onCheckedChange={updateConfig}
              />
            {/snippet}
          </SetItem>

          <SetItem
            title="退出应用时自动上传备份"
            description="应用退出或关机前，会自动把最新的应用配置打包备份到云端"
          >
            {#snippet content()}
              <Switch
                bind:checked={syncOnExit}
                onCheckedChange={updateConfig}
              />
            {/snippet}
          </SetItem>
        </Card>
      </section>

      <!-- 备份与恢复执行面板 -->
      <section class="mb-6">
        <h2
          class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
        >
          数据手动同步
        </h2>
        <Card class="px-4 py-1">
          <!-- 上次同步状态显示 -->
          <SetItem title="云端备份状态">
            {#snippet content()}
              <div class="flex items-center gap-3">
                <div class="text-right text-xs">
                  {#if checkingBackup}
                    <span
                      class="text-muted-foreground flex items-center justify-end gap-1"
                    >
                      <Spinner class="h-3 w-3 animate-spin" /> 检测云端数据...
                    </span>
                  {:else if cloudBackupInfo}
                    <div class="text-muted-foreground">
                      <span class="text-foreground font-medium">上次备份:</span>
                      {formatTime(cloudBackupInfo.last_sync_time)}
                    </div>
                    <div class="text-muted-foreground/70 text-[10px]">
                      <span>设备:</span>
                      <span class="font-mono"
                        >{cloudBackupInfo.device_id || "未知"}</span
                      >
                    </div>
                  {:else}
                    <span class="text-muted-foreground"
                      >云端未检测到备份文件</span
                    >
                  {/if}
                </div>
                <Button
                  variant="outline"
                  size="icon"
                  class="h-8 w-8"
                  onclick={checkCloudBackup}
                  disabled={checkingBackup}
                  title="刷新云端备份状态"
                >
                  <ArrowsClockwise
                    class="h-3.5 w-3.5 {checkingBackup ? 'animate-spin' : ''}"
                  />
                </Button>
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
                <Button
                  variant="default"
                  size="sm"
                  onclick={() => executeSync("backup")}
                  disabled={syncing}
                >
                  {#if syncing && syncMode === "backup"}
                    <Spinner class="mr-1 h-3.5 w-3.5 animate-spin" /> 备份中...
                  {:else}
                    <CloudArrowUp class="mr-1 h-3.5 w-3.5" /> 立即备份 (上传)
                  {/if}
                </Button>

                <!-- 立即恢复 -->
                <Button
                  variant="outline"
                  size="sm"
                  onclick={() => executeSync("restore")}
                  disabled={syncing}
                >
                  {#if syncing && syncMode === "restore"}
                    <Spinner class="mr-1 h-3.5 w-3.5 animate-spin" /> 恢复中...
                  {:else}
                    <CloudArrowDown class="mr-1 h-3.5 w-3.5" /> 立即同步 (下载)
                  {/if}
                </Button>
              </div>
            {/snippet}
          </SetItem>
        </Card>
      </section>
    {/if}
  </main>
</ScrollArea>
