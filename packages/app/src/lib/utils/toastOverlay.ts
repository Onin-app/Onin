import { invoke } from "@tauri-apps/api/core";

export type ToastOverlayKind =
  | "default"
  | "success"
  | "error"
  | "warning"
  | "info";

export interface ToastOverlayOptions {
  kind?: ToastOverlayKind;
  duration?: number;
}

export async function showToastOverlay(
  message: string,
  { kind = "default", duration = 1400 }: ToastOverlayOptions = {},
) {
  await invoke("show_toast_overlay", {
    message,
    kind,
    duration,
  });
}
