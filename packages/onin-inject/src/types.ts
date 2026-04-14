/**
 * Toast payload 类型定义
 * 与 Rust 端 ToastPayload 和 SDK ToastType 对齐
 */
export interface ToastPayload {
  message: string;
  kind: "default" | "success" | "error" | "warning" | "info";
  duration?: number;
}

declare global {
  interface Window {
    __ONIN_RUNTIME__?: {
      mode: "inline" | "window";
      pluginId: string;
      version: string;
      mainWindowLabel: string;
    };
    __ONIN_SHOW_TOAST__?: (payload: ToastPayload) => void;
    __ONIN_TOAST_INJECTED__?: boolean;
    __PLUGIN_ID__?: string;
  }
}
