<script lang="ts">
  import { Toaster, toast } from "svelte-sonner";

  /**
   * 暴露给外部调用的 show 方法
   * 兼容旧版 __ONIN_SHOW_TOAST__ 接口
   */
  export function show(payload: { message: string; kind?: string; duration?: number }) {
    const { message, kind = "default", duration = 4000 } = payload;

    const opts = { duration };

    switch (kind) {
      case "success":
        toast.success(message, opts);
        break;
      case "error":
        toast.error(message, opts);
        break;
      case "warning":
        toast.warning(message, opts);
        break;
      case "info":
        toast.info(message, opts);
        break;
      default:
        toast.message(message, opts);
        break;
    }
  }
</script>

<Toaster
  position="top-center"
  richColors
  toastOptions={{
    style: "font-family: -apple-system, system-ui, sans-serif;",
  }}
/>
