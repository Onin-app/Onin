import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast } from "svelte-sonner";
import type { ColorConversion } from "$lib/type";
import { showToastOverlay } from "$lib/utils/toastOverlay";

let isPicking = false;

interface StartColorPickerFlowOptions {
  beforeStart?: () => void;
  onCancel?: () => void;
  onSuccess?: (color: ColorConversion) => void | Promise<void>;
  closeOnSuccess?: boolean;
  restoreMainWindow?: boolean;
  useToastOverlay?: boolean;
}

function getErrorMessage(error: unknown, fallback: string) {
  if (error instanceof Error && error.message) return error.message;
  if (typeof error === "string" && error) return error;
  if (
    error &&
    typeof error === "object" &&
    "message" in error &&
    typeof error.message === "string" &&
    error.message
  ) {
    return error.message;
  }
  return fallback;
}

function wait(ms: number) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

export async function startColorPickerFlow({
  beforeStart,
  onCancel,
  onSuccess,
  closeOnSuccess = true,
  restoreMainWindow = true,
  useToastOverlay = false,
}: StartColorPickerFlowOptions = {}) {
  if (isPicking) return;

  isPicking = true;
  beforeStart?.();

  let unlisten: (() => void) | undefined;
  const cleanupListener = () => {
    const stopListening = unlisten;
    unlisten = undefined;
    stopListening?.();
  };

  try {
    unlisten = await listen<ColorConversion | null>(
      "color_picker_result",
      async (event) => {
        if (!isPicking) return;

        isPicking = false;
        cleanupListener();

        if (!event.payload) {
          toast.info("已取消取色");
          onCancel?.();
          return;
        }

        try {
          await invoke("plugin_clipboard_write_text", {
            options: { text: event.payload.hex },
          });
          const message = `${event.payload.hex} 已复制`;
          if (useToastOverlay) {
            await wait(140);
            await showToastOverlay(message, {
              kind: "success",
              duration: 1400,
            }).catch((error) => {
              console.error(
                "[ColorPicker] Failed to show toast overlay:",
                error,
              );
            });
          } else {
            toast.success(message);
          }
        } catch (error) {
          console.error("[ColorPicker] Failed to copy picked color:", error);
          const message = `${event.payload.hex} 已取色，复制失败`;
          if (useToastOverlay) {
            await wait(140);
            await showToastOverlay(message, {
              kind: "error",
              duration: 1800,
            }).catch(() => toast.error(message));
          } else {
            toast.error(message);
          }
        }

        await onSuccess?.(event.payload);

        if (closeOnSuccess) {
          invoke("close_main_window");
        }
      },
    );

    await invoke("start_color_picker", { restoreMainWindow });
  } catch (error) {
    isPicking = false;
    cleanupListener();
    toast.error(getErrorMessage(error, "取色失败"));
  }
}

// Exported for testing only
export function resetColorPickerState() {
  isPicking = false;
}
