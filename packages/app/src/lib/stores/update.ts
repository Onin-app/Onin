import { writable, get } from "svelte/store";
import { getVersion } from "@tauri-apps/api/app";
import { platform } from "@tauri-apps/plugin-os";
import { UPDATE_CONFIG } from "$lib/constants";
import { toast } from "svelte-sonner";

// 全局响应式状态
export const checkingUpdate = writable(false);
export const updateDialogOpen = writable(false);
export const hasNewVersion = writable(false);
export const latestVersion = writable("");
export const releaseNotes = writable("");
export const downloadUrl = writable("");
export const appVersion = writable("未知");

const CACHE_KEY = "onin_last_notified_version";

// 初始化当前应用版本号
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

// 简单的语义化版本号对比算法
function isNewerVersion(current: string, latest: string) {
  const cur = current.replace(/^v/, "").split(".").map(Number);
  const lat = latest.replace(/^v/, "").split(".").map(Number);

  for (let i = 0; i < Math.max(cur.length, lat.length); i++) {
    const c = cur[i] || 0;
    const l = lat[i] || 0;
    if (l > c) return true;
    if (l < c) return false;
  }
  return false;
}

/**
 * 核心检查更新方法
 * @param silent 是否为静默检测（启动/轮询时）。如果为 false（手动触发），将直接拉起模态下载弹窗。
 */
export async function checkUpdate(silent: boolean = false) {
  if (get(checkingUpdate)) return;
  checkingUpdate.set(true);

  try {
    let currentVer = get(appVersion);
    if (currentVer === "未知") {
      currentVer = await initVersion();
    }

    const response = await fetch(UPDATE_CONFIG.LATEST_RELEASE_URL);
    if (!response.ok) {
      throw new Error("网络连接失败");
    }
    const data = await response.json();
    const tagName = data.tag_name || "";

    if (isNewerVersion(currentVer, tagName)) {
      let osPlatform = "windows";
      try {
        osPlatform = await platform();
      } catch (e) {
        console.error("Failed to get platform", e);
      }

      // 根据平台匹配正确的安装包资源 (加装防守链，彻底防御 assets 未定义引发崩溃的隐患)
      const assets = data.assets || [];
      let matchedAsset = null;
      if (osPlatform === "windows") {
        matchedAsset = assets.find((asset: any) => asset.name.endsWith(".msi"));
      } else if (osPlatform === "linux") {
        matchedAsset =
          assets.find((asset: any) => asset.name.endsWith(".AppImage")) ||
          assets.find((asset: any) => asset.name.endsWith(".deb"));
      } else if (osPlatform === "macos") {
        matchedAsset = assets.find((asset: any) => asset.name.endsWith(".dmg"));
      }

      if (matchedAsset) {
        const cleanLatestVersion = tagName.replace(/^v/, "");
        latestVersion.set(cleanLatestVersion);
        releaseNotes.set(data.body || "");
        downloadUrl.set(matchedAsset.browser_download_url);
        hasNewVersion.set(true);

        if (silent) {
          // 静默检测发现新版本，执行防打扰 Toast 提示逻辑
          const lastNotified = localStorage.getItem(CACHE_KEY);
          if (lastNotified !== cleanLatestVersion) {
            // 用 svelte-sonner 弹出一个支持交互并且持久挂载的 Toast 提示
            toast.info(`发现新版本 v${cleanLatestVersion}！`, {
              duration: 10000, // 提示展示 10 秒
              action: {
                label: "立即查看",
                onClick: () => {
                  updateDialogOpen.set(true);
                },
              },
            });
            // 写入缓存，防止相同版本频繁弹 Toast 干扰操作
            localStorage.setItem(CACHE_KEY, cleanLatestVersion);
          }
        } else {
          // 手动点击检查，直接展开精美的详细下载升级弹窗
          updateDialogOpen.set(true);
        }
      } else if (!silent) {
        toast.warning(
          `检测到新版本 ${tagName}，但未找到适用于您平台的安装包。`,
        );
      }
    } else {
      // 当前是最新版本，若自动检查则悄无声息重置状态，手动检查则温馨提示
      hasNewVersion.set(false);
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
 * 关闭并清理模态弹窗状态
 */
export function closeUpdateDialog() {
  updateDialogOpen.set(false);
}

// 自动执行一次初始化
initVersion();
