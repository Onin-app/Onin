<script lang="ts">
  import { openExternalLink } from "$lib/utils/link";
  import {
    X,
    ArrowCircleUp,
    CloudArrowDown,
    Warning,
    ArrowSquareOut,
  } from "phosphor-svelte";
  import {
    downloading,
    installing,
    isLongInstalling,
    downloadPercent,
    downloadedBytes,
    totalBytes,
    downloadError,
    startUpdate,
    openManualDownload,
    closeUpdateDialog,
  } from "$lib/stores/update";
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
    DialogFooter,
  } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";

  interface Props {
    open: boolean;
    currentVersion: string;
    latestVersion: string;
    releaseNotes: string;
    onClose: () => void;
  }

  let {
    open = $bindable(false),
    currentVersion,
    latestVersion,
    releaseNotes,
    onClose,
  }: Props = $props();

  function formatBytes(bytes: number, decimals = 1) {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i];
  }

  async function handleNotesClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const anchor = target.closest("a");
    if (anchor) {
      const href = anchor.getAttribute("href");
      if (href && (href.startsWith("http://") || href.startsWith("https://"))) {
        await openExternalLink(href, event);
      }
    }
  }

  async function handleStartUpdate() {
    await startUpdate();
  }

  function handleCancel() {
    if ($installing && !$isLongInstalling) return;
    open = false;
    closeUpdateDialog();
    onClose();
  }
</script>

<Dialog
  bind:open
  onOpenChange={(v) => {
    if (!v) handleCancel();
  }}
>
  <DialogContent class="max-w-lg p-6">
    <!-- 头部信息 -->
    <div class="flex items-start gap-4">
      <div
        class="bg-primary/10 text-primary flex size-12 shrink-0 items-center justify-center rounded-xl"
      >
        {#if $downloading || $installing}
          <CloudArrowDown size={26} class="animate-bounce" />
        {:else}
          <ArrowCircleUp size={26} />
        {/if}
      </div>
      <div class="flex-1">
        <DialogTitle class="text-lg font-bold">
          {#if $installing}
            正在安装更新...
          {:else if $downloading}
            正在下载更新...
          {:else}
            发现新版本！
          {/if}
        </DialogTitle>
        <DialogDescription class="mt-1 text-xs">
          最新版本: <span class="text-primary font-mono font-semibold"
            >v{latestVersion}</span
          >
          (当前版本: v{currentVersion})
        </DialogDescription>
      </div>
    </div>

    <!-- 中间内容：更新日志 / 进度条 -->
    <div class="my-4 min-h-0 flex-1">
      {#if !$downloading && !$installing}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <ScrollArea
          class="bg-muted/30 max-h-56 rounded-xl border p-4 text-xs"
          onclick={handleNotesClick}
        >
          <div
            class="prose prose-xs dark:prose-invert prose-p:my-1 text-muted-foreground max-w-none"
          >
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html releaseNotes}
          </div>
        </ScrollArea>
      {:else}
        <!-- 下载与安装进度展示 -->
        <div class="flex flex-col gap-3 py-4">
          <div
            class="text-muted-foreground flex items-center justify-between text-xs"
          >
            <span class="font-medium">
              {#if $installing}
                正在解包并覆盖旧版本...
              {:else if $totalBytes}
                已下载 {formatBytes($downloadedBytes)} / {formatBytes(
                  $totalBytes,
                )}
              {:else}
                已下载 {formatBytes($downloadedBytes)}
              {/if}
            </span>
            <span class="text-primary font-mono font-bold">
              {$installing ? "100%" : `${$downloadPercent}%`}
            </span>
          </div>

          <!-- 进度条轨道 -->
          <div class="bg-secondary h-2.5 w-full overflow-hidden rounded-full">
            <div
              class="bg-primary h-full rounded-full transition-all duration-150 ease-out"
              style="width: {$installing ? 100 : $downloadPercent}%"
            ></div>
          </div>

          <div class="space-y-1 text-center">
            <p class="text-muted-foreground/80 text-[11px]">
              {#if $installing}
                正在执行覆盖安装，完成后将自动重启。
              {:else}
                下载完成后系统将自动覆盖升级，在此期间请勿关闭应用。
              {/if}
            </p>
            {#if $installing}
              <p class="text-muted-foreground/60 text-[10px]">
                提示：若弹出 macOS 管理员授权弹窗，请输入密码以允许写入。
              </p>
            {/if}
          </div>

          <!-- 长时间解包/安装兜底提示 -->
          {#if $isLongInstalling}
            <div
              class="mt-2 flex flex-col gap-2 rounded-lg bg-amber-500/10 p-3 text-xs text-amber-600 dark:text-amber-400"
            >
              <div class="flex items-start gap-2">
                <Warning size={16} class="mt-0.5 shrink-0" />
                <span class="leading-relaxed">
                  解包安装耗时较长。若系统权限受限或长时间未响应，可直接手动下载安装包覆盖。
                </span>
              </div>
              <div class="flex justify-end gap-2 pt-1">
                <Button
                  variant="outline"
                  size="sm"
                  class="h-7 text-xs"
                  onclick={handleCancel}
                >
                  取消
                </Button>
                <Button
                  size="sm"
                  class="h-7 text-xs"
                  onclick={openManualDownload}
                >
                  <ArrowSquareOut size={14} class="mr-1" />
                  前往下载安装包
                </Button>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      <!-- 错误提示 -->
      {#if $downloadError}
        <div
          class="bg-destructive/10 text-destructive mt-3 flex flex-col gap-2 rounded-lg p-3 text-xs"
        >
          <div class="flex items-start gap-2.5">
            <Warning size={16} class="mt-0.5 shrink-0" />
            <div class="flex-1">
              <span class="font-semibold">升级失败:</span>
              {$downloadError}
            </div>
          </div>
          <div class="flex justify-end pt-1">
            <Button
              variant="outline"
              size="sm"
              class="h-7 text-xs"
              onclick={openManualDownload}
            >
              <ArrowSquareOut size={14} class="mr-1" />
              手动下载最新版本
            </Button>
          </div>
        </div>
      {/if}
    </div>

    <!-- 底部控制按钮 -->
    {#if !$downloading && !$installing}
      <DialogFooter class="gap-2 sm:gap-2">
        <Button variant="outline" size="sm" onclick={handleCancel}>
          稍后提醒
        </Button>
        <Button size="sm" onclick={handleStartUpdate}>立即升级</Button>
      </DialogFooter>
    {/if}
  </DialogContent>
</Dialog>
