<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Toaster, toast } from "svelte-sonner";
  import type { ToastOverlayKind } from "$lib/utils/toastOverlay";

  interface ToastOverlayPayload {
    message: string;
    kind?: ToastOverlayKind;
    duration?: number;
  }

  const TOAST_ID = "toast-overlay";
  const SONNER_AUTO_DISMISS_MS = 60_000;
  let hideTimer: number | undefined;

  function showToast(payload: ToastOverlayPayload) {
    window.clearTimeout(hideTimer);

    const options = {
      id: TOAST_ID,
      duration: Math.max(SONNER_AUTO_DISMISS_MS, payload.duration || 1400),
    };
    switch (payload.kind) {
      case "success":
        toast.success(payload.message, options);
        break;
      case "error":
        toast.error(payload.message, options);
        break;
      case "warning":
        toast.warning(payload.message, options);
        break;
      case "info":
        toast.info(payload.message, options);
        break;
      default:
        toast(payload.message, options);
        break;
    }

    hideTimer = window.setTimeout(
      () => {
        toast.dismiss(TOAST_ID);
        getCurrentWindow()
          .hide()
          .catch(() => {});
      },
      (payload.duration || 1400) + 420,
    );
  }

  onMount(() => {
    let unlisten: UnlistenFn | undefined;

    const params = new URLSearchParams(window.location.search);
    const initialMessage = params.get("message");
    if (initialMessage) {
      getCurrentWindow()
        .show()
        .catch(() => {})
        .finally(() => {
          window.setTimeout(() => {
            showToast({
              message: initialMessage,
              kind: (params.get("kind") as ToastOverlayKind) || "success",
              duration: Number(params.get("duration")) || 1400,
            });
          }, 16);
        });
    }

    listen<ToastOverlayPayload>("toast_overlay_show", (event) => {
      getCurrentWindow()
        .show()
        .catch(() => {})
        .finally(() => {
          showToast(event.payload);
        });
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      window.clearTimeout(hideTimer);
      toast.dismiss(TOAST_ID);
      unlisten?.();
    };
  });
</script>

<Toaster richColors position="top-center" />

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent;
    user-select: none;
  }
</style>
