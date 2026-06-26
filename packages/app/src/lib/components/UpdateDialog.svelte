<script lang="ts">
  import { openExternalLink } from "$lib/utils/link";
  import { X, ArrowCircleUp, CloudArrowDown, Warning } from "phosphor-svelte";
  import {
    downloading,
    installing,
    downloadPercent,
    downloadedBytes,
    totalBytes,
    downloadError,
    startUpdate,
  } from "$lib/stores/update";

  interface Props {
    open: boolean;
    currentVersion: string;
    latestVersion: string;
    releaseNotes: string; // 此时传入的 releaseNotes 已经是经过 marked 渲染并进行 XSS 消毒后的 HTML 字符串
    onClose: () => void;
  }

  let {
    open = $bindable(false),
    currentVersion,
    latestVersion,
    releaseNotes,
    onClose,
  }: Props = $props();

  // 格式化字节
  function formatBytes(bytes: number, decimals = 1) {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i];
  }

  // 处理更新日志中的链接点击，在系统默认浏览器中打开
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
    if ($installing) return; // 安装阶段禁止取消
    open = false;
    onClose();
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-md transition-opacity duration-300"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="relative flex w-[480px] max-w-[90vw] flex-col overflow-hidden rounded-2xl border border-neutral-200/80 bg-white/95 p-6 shadow-2xl transition-all duration-300 dark:border-neutral-800/80 dark:bg-neutral-900/95"
    >
      <!-- 关闭按钮 -->
      <button
        onclick={handleCancel}
        disabled={$installing}
        class="absolute top-4 right-4 rounded-lg p-1.5 text-neutral-400 transition-colors hover:bg-neutral-100 hover:text-neutral-600 disabled:pointer-events-none disabled:opacity-30 dark:hover:bg-neutral-800 dark:hover:text-neutral-200"
        aria-label="关闭"
      >
        <X size={18} />
      </button>

      <!-- 头部信息 -->
      <div class="flex items-start gap-4 pr-6">
        <div
          class="flex size-12 shrink-0 items-center justify-center rounded-xl bg-violet-100 text-violet-600 dark:bg-violet-950/50 dark:text-violet-400"
        >
          {#if $downloading || $installing}
            <CloudArrowDown size={28} class="animate-bounce" />
          {:else}
            <ArrowCircleUp size={28} />
          {/if}
        </div>
        <div class="flex-1">
          <h3 class="text-lg font-bold text-neutral-900 dark:text-neutral-50">
            {#if $installing}
              正在安装更新...
            {:else if $downloading}
              正在下载更新...
            {:else}
              发现新版本！
            {/if}
          </h3>
          <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
            最新版本: <span
              class="font-mono font-semibold text-violet-600 dark:text-violet-400"
              >v{latestVersion}</span
            >
            (当前版本: v{currentVersion})
          </p>
        </div>
      </div>

      <!-- 中间内容：更新日志 / 进度条 -->
      <div class="my-5 min-h-0 flex-1">
        {#if !$downloading && !$installing}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="border-neutral-150 max-h-56 overflow-y-auto rounded-xl border bg-neutral-50/50 p-4 text-sm text-neutral-600 dark:border-neutral-800/60 dark:bg-neutral-950/40 dark:text-neutral-300"
            onclick={handleNotesClick}
          >
            <div
              class="prose prose-xs dark:prose-invert prose-p:my-1 max-w-none"
            >
              <!-- eslint-disable-next-line svelte/no-at-html-tags -->
              {@html releaseNotes}
            </div>
          </div>
        {:else}
          <!-- 下载与安装进度展示 -->
          <div class="flex flex-col gap-3 py-6">
            <div
              class="flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400"
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
              <span
                class="font-mono font-bold text-violet-600 dark:text-violet-400"
              >
                {$installing ? "100%" : `${$downloadPercent}%`}
              </span>
            </div>

            <!-- 进度条轨道 -->
            <div
              class="h-3 w-full overflow-hidden rounded-full bg-neutral-100 dark:bg-neutral-800"
            >
              <!-- 炫酷流水渐变进度条 -->
              <div
                class="h-full rounded-full bg-gradient-to-r from-violet-500 to-indigo-500 transition-all duration-150 ease-out"
                style="width: {$installing ? 100 : $downloadPercent}%"
              ></div>
            </div>
            <p
              class="text-center text-[10px] text-neutral-400 dark:text-neutral-500"
            >
              {#if $installing}
                正在执行安装，完成后应用将自动重启，在此期间请勿关闭应用。
              {:else}
                下载完成后系统将自动覆盖升级，在此期间请勿关闭应用。
              {/if}
            </p>
          </div>
        {/if}

        <!-- 错误提示 -->
        {#if $downloadError}
          <div
            class="mt-3 flex items-start gap-2.5 rounded-lg bg-red-50 p-3 text-xs text-red-700 dark:bg-red-950/20 dark:text-red-400"
          >
            <Warning size={16} class="mt-0.5 shrink-0" />
            <div class="flex-1">
              <span class="font-semibold">升级失败:</span>
              {$downloadError}
            </div>
          </div>
        {/if}
      </div>

      <!-- 底部控制按钮 -->
      {#if !$downloading && !$installing}
        <div
          class="flex justify-end gap-3 border-t border-neutral-100 pt-4 dark:border-neutral-800"
        >
          <button
            onclick={handleCancel}
            class="rounded-xl border border-neutral-200 px-4 py-2 text-xs font-semibold text-neutral-600 transition-all hover:bg-neutral-50 dark:border-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            稍后提醒
          </button>
          <button
            onclick={handleStartUpdate}
            class="flex items-center justify-center gap-1.5 rounded-xl bg-gradient-to-r from-violet-600 to-indigo-600 px-5 py-2 text-xs font-semibold text-white shadow-md shadow-violet-500/10 transition-all hover:from-violet-500 hover:to-indigo-500 hover:shadow-lg hover:shadow-violet-500/20 focus:outline-none"
          >
            立即升级
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
