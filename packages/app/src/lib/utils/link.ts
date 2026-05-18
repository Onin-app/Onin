import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

/**
 * 安全地在系统默认浏览器中打开外部链接，并临时锁定窗口防止由于失去焦点导致窗口自动隐藏。
 *
 * 保持了 HTML 语义化与 A 标签的可访问性（如屏幕阅读器、右键菜单、键盘导航、辅助键点击等）。
 *
 * @param href 外部链接地址
 * @param event 鼠标点击事件
 */
export async function openExternalLink(href: string, event: MouseEvent) {
  if (!href) return;

  // 仅拦截普通的左键点击，允许中键、右键或辅助键（Ctrl/Cmd/Shift等）的原生行为（在支持的外壳中）
  if (
    event.button !== 0 ||
    event.ctrlKey ||
    event.metaKey ||
    event.shiftKey ||
    event.altKey
  ) {
    return;
  }

  event.preventDefault();

  try {
    // 锁定窗口关闭，防止点击超链接失焦导致窗口自动隐藏
    await invoke("acquire_window_close_lock");
    await openUrl(href);
    // 延时 500ms 后释放锁，确保焦点成功切换至系统浏览器，避免由于并发竞争导致窗口被隐藏
    setTimeout(async () => {
      try {
        await invoke("release_window_close_lock");
      } catch (err) {
        console.error("释放窗口锁失败:", err);
      }
    }, 500);
  } catch (e) {
    console.error("无法打开外部链接:", e);
    try {
      await invoke("release_window_close_lock");
    } catch (err) {
      console.error("释放窗口锁失败:", err);
    }
  }
}
