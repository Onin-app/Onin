import { writable, get } from "svelte/store";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { toast } from "svelte-sonner";
import { trackEvent } from "$lib/tracking";

// 响应式状态
export const checkingUpdate = writable(false);
export const updateDialogOpen = writable(false);
export const hasNewVersion = writable(false);
export const latestVersion = writable("");
export const releaseNotes = writable("");
export const appVersion = writable("未知");

// 下载与安装状态
export const downloading = writable(false);
export const installing = writable(false);
export const isLongInstalling = writable(false);
export const downloadPercent = writable(0);
export const downloadedBytes = writable(0);
export const totalBytes = writable<number | null>(null);
export const downloadError = writable("");

// 缓存当前可用的更新对象
let currentUpdate: Update | null = null;
let installTimeoutTimer: ReturnType<typeof setTimeout> | null = null;

const CACHE_KEY_NOTIFIED = "onin_last_notified_version";

/**
 * 临时调整主窗口置顶状态，避免在 macOS 下更新时遮挡系统管理员密码输入弹窗
 */
async function setWindowAlwaysOnTop(val: boolean) {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().setAlwaysOnTop(val);
  } catch (e) {
    console.warn("更新流程设置窗口置顶失败:", e);
  }
}

/**
 * 打开浏览器手动下载最新版本
 */
export async function openManualDownload() {
  const { openBrowserUrl } = await import("$lib/utils/link");
  await openBrowserUrl("https://github.com/Onin-app/Onin/releases/latest");
}

// 极简的原生 DOMParser HTML 安全消毒函数，彻底防御 XSS 攻击
function sanitizeHtml(html: string): string {
  if (typeof window === "undefined") return html;
  try {
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");

    const clean = (node: Node) => {
      if (node.nodeType === 1) {
        const el = node as Element;
        const tagName = el.tagName.toLowerCase();

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

        const attrs = Array.from(el.attributes || []);
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

      const children = Array.from(node.childNodes || []);
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

async function initVersion() {
  try {
    const version = await getVersion();
    appVersion.set(version);
    return version;
  } catch (e) {
    console.error("Failed to get app version:", e);
    return "未知";
  }
}

/**
 * 检查更新
 */
export async function checkUpdate(silent: boolean = false) {
  if (get(checkingUpdate)) return;
  checkingUpdate.set(true);
  downloadError.set(""); // 重置上次可能残留的错误状态

  try {
    let currentVer = get(appVersion);
    if (currentVer === "未知") {
      currentVer = await initVersion();
    }

    const update = await check();

    if (update?.available) {
      currentUpdate = update;
      const cleanVersion = update.version.replace(/^v/, "");

      latestVersion.set(cleanVersion);

      // 按需动态加载 marked 进行 Markdown 解析，提升首屏渲染性能
      try {
        const { marked } = await import("marked");
        const rawHtml = await marked.parse(update.body || "无详细更新说明。");
        const cleanHtml = sanitizeHtml(rawHtml as string);
        releaseNotes.set(cleanHtml);
      } catch (err) {
        console.error("解析 Release Notes 失败:", err);
        releaseNotes.set(update.body || "无详细更新说明。");
      }

      hasNewVersion.set(true);

      trackEvent("update_found", {
        current_version: currentVer,
        latest_version: cleanVersion,
      });

      const lastNotified = localStorage.getItem(CACHE_KEY_NOTIFIED);

      if (silent) {
        if (lastNotified !== cleanVersion) {
          toast.info(`发现新版本 v${cleanVersion}！`, {
            duration: 10000,
            action: {
              label: "立即查看",
              onClick: () => updateDialogOpen.set(true),
            },
          });
          localStorage.setItem(CACHE_KEY_NOTIFIED, cleanVersion);
        }
      } else {
        updateDialogOpen.set(true);
        localStorage.setItem(CACHE_KEY_NOTIFIED, cleanVersion);
      }
    } else {
      hasNewVersion.set(false);
      currentUpdate = null;
      if (!silent) {
        toast.success("当前已是最新版本");
      }
    }
  } catch (e) {
    console.error("检查更新失败", e);
    if (!silent) {
      toast.error("检查更新失败，请稍后重试");
    }
  } finally {
    checkingUpdate.set(false);
  }
}

/**
 * 下载并安装更新
 */
export async function startUpdate() {
  if (!currentUpdate || get(downloading) || get(installing)) return;

  downloading.set(true);
  installing.set(false);
  isLongInstalling.set(false);
  downloadError.set("");
  downloadPercent.set(0);
  downloadedBytes.set(0);
  totalBytes.set(null);

  // 开始更新流程时，解除置顶，防止 macOS 系统管理员授权弹窗被无边框置顶窗口遮挡
  await setWindowAlwaysOnTop(false);

  const currentVer = get(appVersion);
  const targetVer = get(latestVersion);

  trackEvent("update_started", {
    current_version: currentVer,
    latest_version: targetVer,
  });

  try {
    let downloaded = 0;

    await currentUpdate.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          totalBytes.set(event.data.contentLength ?? null);
          break;
        case "Progress":
          // 因官方 v2 API 回调 payload 仅包含 chunkLength 增量值，故在此进行累加
          downloaded += event.data.chunkLength;
          downloadedBytes.set(downloaded);
          const total = get(totalBytes);
          if (total && total > 0) {
            downloadPercent.set(Math.round((downloaded / total) * 1000) / 10);
          }
          break;
        case "Finished":
          downloadPercent.set(100);
          downloading.set(false);
          installing.set(true); // 转为正在安装中
          // 启动长时间未完成提示计时器（12秒）
          if (installTimeoutTimer) clearTimeout(installTimeoutTimer);
          installTimeoutTimer = setTimeout(() => {
            if (get(installing)) {
              isLongInstalling.set(true);
            }
          }, 12000);
          trackEvent("update_downloaded", {
            current_version: currentVer,
            latest_version: targetVer,
          });
          break;
      }
    });

    if (installTimeoutTimer) {
      clearTimeout(installTimeoutTimer);
      installTimeoutTimer = null;
    }

    // 下载安装完成，重启应用
    await relaunch();
  } catch (e) {
    if (installTimeoutTimer) {
      clearTimeout(installTimeoutTimer);
      installTimeoutTimer = null;
    }
    console.error("更新失败:", e);

    // 失败时恢复置顶
    await setWindowAlwaysOnTop(true);

    trackEvent("update_failed", {
      current_version: currentVer,
      latest_version: targetVer,
      error: String(e) || "unknown",
    });

    downloadError.set(String(e) || "下载更新失败，请重试");
    downloading.set(false);
    installing.set(false);
    isLongInstalling.set(false);
  }
}

export function closeUpdateDialog() {
  if (installTimeoutTimer) {
    clearTimeout(installTimeoutTimer);
    installTimeoutTimer = null;
  }
  updateDialogOpen.set(false);
  downloadError.set(""); // 关闭弹窗时重置错误，防止下次残留
  downloading.set(false);
  installing.set(false);
  isLongInstalling.set(false);
  // 恢复置顶
  setWindowAlwaysOnTop(true);
}

// 初始化版本号
initVersion();
