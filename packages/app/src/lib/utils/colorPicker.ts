import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast } from "svelte-sonner";
import type { ColorConversion } from "$lib/type";

let isPicking = false;

interface StartColorPickerFlowOptions {
  beforeStart?: () => void;
  onCancel?: () => void;
  onSuccess?: (color: ColorConversion) => void | Promise<void>;
  closeOnSuccess?: boolean;
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

export async function startColorPickerFlow({
  beforeStart,
  onCancel,
  onSuccess,
  closeOnSuccess = true,
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
          toast.success(`${event.payload.hex} 已复制`);
        } catch (error) {
          console.error("[ColorPicker] Failed to copy picked color:", error);
          toast.error(`${event.payload.hex} 已取色，复制失败`);
        }

        await onSuccess?.(event.payload);

        if (closeOnSuccess) {
          invoke("close_main_window");
        }
      },
    );

    await invoke("start_color_picker");
  } catch (error) {
    isPicking = false;
    cleanupListener();
    toast.error(getErrorMessage(error, "取色失败"));
  }
}
