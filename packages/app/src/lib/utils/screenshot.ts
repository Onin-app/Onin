import { invoke } from "@tauri-apps/api/core";
import { showToastOverlay } from "$lib/utils/toastOverlay";

/**
 * Runs the native screenshot command and reports its outcome independently of
 * the main launcher window. This is shared by launcher and shortcut entry
 * points so both paths have identical behavior.
 */
export async function takeScreenshot(): Promise<boolean> {
  try {
    await invoke("take_screenshot");
    void showToastOverlay("截图已复制到剪贴板", { kind: "success" }).catch(
      (error) =>
        console.error("[Screenshot] Failed to show success toast", error),
    );
    return true;
  } catch (error) {
    console.error("[Screenshot] Capture failed", error);
    void showToastOverlay("截图失败，请重试", { kind: "error" }).catch(
      (toastError) =>
        console.error("[Screenshot] Failed to show error toast", toastError),
    );
    return false;
  }
}
