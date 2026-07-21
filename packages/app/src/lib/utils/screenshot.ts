import { invoke } from "@tauri-apps/api/core";
import { showToastOverlay } from "$lib/utils/toastOverlay";

/**
 * Starts the native region-selection flow. This is shared by launcher and
 * shortcut entry points so both paths have identical behavior.
 */
export async function takeScreenshot(): Promise<boolean> {
  try {
    await invoke("start_screenshot_selection");
    return true;
  } catch (error) {
    console.error("[Screenshot] Capture failed", error);
    void showToastOverlay("无法启动截图，请重试", { kind: "error" }).catch(
      (toastError) =>
        console.error("[Screenshot] Failed to show error toast", toastError),
    );
    return false;
  }
}
