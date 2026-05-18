<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { marked } from "marked";
  import { X, ArrowCircleUp, CloudArrowDown, Warning } from "phosphor-svelte";

  interface Props {
    open: boolean;
    currentVersion: string;
    latestVersion: string;
    releaseNotes: string;
    downloadUrl: string;
    onClose: () => void;
  }

  let {
    open = $bindable(false),
    currentVersion,
    latestVersion,
    releaseNotes,
    downloadUrl,
    onClose,
  }: Props = $props();

  interface ProgressPayload {
    downloaded: number;
    total: number | null;
    percent: number | null;
  }

  let downloading = $state(false);
  let percent = $state(0);
  let downloadedBytes = $state(0);
  let totalBytes = $state<number | null>(null);
  let errorMessage = $state("");
  let unlistenProgress = $state<UnlistenFn | null>(null);
  let unlistenFinished = $state<UnlistenFn | null>(null);

  // 格式化字节
  function formatBytes(bytes: number, decimals = 1) {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i];
  }

  // 极简的原生 DOMParser HTML 安全消毒函数，彻底防御 XSS 攻击
  function sanitizeHtml(html: string): string {
    if (typeof window === "undefined") return html;
    try {
      const parser = new DOMParser();
      const doc = parser.parseFromString(html, "text/html");

      // 深度优先递归清理节点和属性
      const clean = (node: Node) => {
        if (node.nodeType === 1) {
          const el = node as Element;
          const tagName = el.tagName.toLowerCase();

          // 移除任何危险或不安全标签
          if (
            [
              "script",
              "iframe",
              "object",
              "embed",
              "form",
              "style",
              "meta",
              "link",
            ].includes(tagName)
          ) {
            el.remove();
            return;
          }

          // 移除所有 inline 的事件监听属性 (on*) 以及 javascript: 伪协议
          const attrs = Array.from(el.attributes);
          for (const attr of attrs) {
            const attrName = attr.name.toLowerCase();
            if (attrName.startsWith("on")) {
              el.removeAttribute(attr.name);
            }
            if (
              ["src", "href", "data"].includes(attrName) &&
              (attr.value.toLowerCase().trim().startsWith("javascript:") ||
                attr.value.toLowerCase().trim().startsWith("data:"))
            ) {
              el.removeAttribute(attr.name);
            }
          }
        }

        // 使用 Array.from 浅拷贝快照进行迭代，完美避开由于子节点被移除导致的索引偏移问题
        const children = Array.from(node.childNodes);
        for (const child of children) {
          clean(child);
        }
      };

      if (doc.body) {
        clean(doc.body);
        return doc.body.innerHTML;
      }
      return html;
    } catch (e) {
      console.error("HTML 消毒失败:", e);
      return html;
    }
  }

  // 渲染 Markdown 并进行 XSS 消毒
  let renderedNotes = $derived.by(() => {
    try {
      const rawHtml = marked.parse(releaseNotes || "无详细更新说明。");
      return sanitizeHtml(rawHtml as string);
    } catch (e) {
      return releaseNotes || "无详细更新说明。";
    }
  });

  async function handleStartUpdate() {
    if (downloading) return;
    downloading = true;
    errorMessage = "";
    percent = 0;
    downloadedBytes = 0;
    totalBytes = null;

    try {
      // 监听进度事件
      unlistenProgress = await listen<ProgressPayload>(
        "update-progress",
        (event) => {
          const payload = event.payload;
          downloadedBytes = payload.downloaded;
          totalBytes = payload.total;
          if (payload.percent !== null) {
            percent = Math.round(payload.percent * 10) / 10;
          }
        },
      );

      unlistenFinished = await listen("update-downloaded", () => {
        downloading = false;
        cleanupListeners();
      });

      // 启动更新
      await invoke("download_and_install_update", { url: downloadUrl });
    } catch (err: any) {
      if (String(err).includes("下载已被用户取消")) {
        // 用户主动取消的升级，属于预期行为，无需报错和重置
        return;
      }
      console.error("更新出错:", err);
      errorMessage = String(err) || "下载更新失败，请重试";
      downloading = false;
      cleanupListeners();
    }
  }

  function cleanupListeners() {
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
    if (unlistenFinished) {
      unlistenFinished();
      unlistenFinished = null;
    }
  }

  async function handleCancel() {
    if (downloading) {
      try {
        await invoke("cancel_update");
      } catch (err) {
        console.error("取消更新出错:", err);
      }
    }
    open = false;
    onClose();
  }

  onDestroy(() => {
    cleanupListeners();
  });
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
        class="absolute top-4 right-4 rounded-lg p-1.5 text-neutral-400 transition-colors hover:bg-neutral-100 hover:text-neutral-600 dark:hover:bg-neutral-800 dark:hover:text-neutral-200"
        aria-label="关闭"
      >
        <X size={18} />
      </button>

      <!-- 头部信息 -->
      <div class="flex items-start gap-4 pr-6">
        <div
          class="flex size-12 shrink-0 items-center justify-center rounded-xl bg-violet-100 text-violet-600 dark:bg-violet-950/50 dark:text-violet-400"
        >
          {#if downloading}
            <CloudArrowDown size={28} class="animate-bounce" />
          {:else}
            <ArrowCircleUp size={28} />
          {/if}
        </div>
        <div class="flex-1">
          <h3 class="text-lg font-bold text-neutral-900 dark:text-neutral-50">
            {downloading ? "正在下载更新..." : "发现新版本！"}
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
        {#if !downloading}
          <div
            class="border-neutral-150 max-h-56 overflow-y-auto rounded-xl border bg-neutral-50/50 p-4 text-sm text-neutral-600 dark:border-neutral-800/60 dark:bg-neutral-950/40 dark:text-neutral-300"
          >
            <div
              class="prose prose-xs dark:prose-invert prose-p:my-1 max-w-none"
            >
              <!-- eslint-disable-next-line svelte/no-at-html-tags -->
              {@html renderedNotes}
            </div>
          </div>
        {:else}
          <!-- 下载进度展示 -->
          <div class="flex flex-col gap-3 py-6">
            <div
              class="flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400"
            >
              <span class="font-medium">
                {#if totalBytes}
                  已下载 {formatBytes(downloadedBytes)} / {formatBytes(
                    totalBytes,
                  )}
                {:else}
                  已下载 {formatBytes(downloadedBytes)}
                {/if}
              </span>
              <span
                class="font-mono font-bold text-violet-600 dark:text-violet-400"
              >
                {percent}%
              </span>
            </div>

            <!-- 进度条轨道 -->
            <div
              class="h-3 w-full overflow-hidden rounded-full bg-neutral-100 dark:bg-neutral-800"
            >
              <!-- 炫酷流水渐变进度条 -->
              <div
                class="h-full rounded-full bg-gradient-to-r from-violet-500 to-indigo-500 transition-all duration-150 ease-out"
                style="width: {percent}%"
              ></div>
            </div>
            <p
              class="text-center text-[10px] text-neutral-400 dark:text-neutral-500"
            >
              下载完成后系统将自动覆盖升级，在此期间请勿关闭应用。
            </p>
          </div>
        {/if}

        <!-- 错误提示 -->
        {#if errorMessage}
          <div
            class="mt-3 flex items-start gap-2.5 rounded-lg bg-red-50 p-3 text-xs text-red-700 dark:bg-red-950/20 dark:text-red-400"
          >
            <Warning size={16} class="mt-0.5 shrink-0" />
            <div class="flex-1">
              <span class="font-semibold">升级失败:</span>
              {errorMessage}
            </div>
          </div>
        {/if}
      </div>

      <!-- 底部控制按钮 -->
      {#if !downloading}
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
