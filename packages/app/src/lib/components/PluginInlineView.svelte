<script lang="ts">
  /**
   * PluginInlineView Component
   *
   * 插件内联视图组件 - Native Webview 版本
   * 使用 div 占位符，并通过 Tauri 2 API 控制原生 Child Webview 的位置和大小
   */
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event"; // [Fix] Import UnlistenFn

  interface Props {
    url: string;
    pluginId?: string;
    onLoad?: () => void;
  }

  let { url, pluginId = "", onLoad }: Props = $props();

  let containerElement = $state<HTMLDivElement | null>(null);
  let resizeObserver: ResizeObserver | null = null;
  let isMounted = false;
  let unlistenLoaded: UnlistenFn | null = null; // [Fix] Declare at top level

  /**
   * 计算带有 mode 和 plugin_id 参数的 URL
   */
  const targetUrl = $derived.by(() => {
    try {
      if (!url) return "";
      const urlObj = new URL(url);
      urlObj.searchParams.set("mode", "inline");
      if (pluginId) {
        urlObj.searchParams.set("plugin_id", pluginId);
      }
      return urlObj.toString();
    } catch {
      if (typeof url !== "string") return "";
      const separator = url.includes("?") ? "&" : "?";
      const params = `mode=inline${pluginId ? `&plugin_id=${encodeURIComponent(pluginId)}` : ""}`;
      return `${url}${separator}${params}`;
    }
  });

  // 暴露元素给父组件 (保持接口兼容，当前为 div 占位符)
  // 父组件可能用它来获取焦点等，对于 div 也可以
  export function getElement(): HTMLDivElement | null {
    return containerElement;
  }

  /**
   * 获取物理像素矩形
   */
  function getPhysicalRect(element: HTMLElement) {
    const rect = element.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    return {
      x: rect.x * dpr,
      y: rect.y * dpr,
      width: rect.width * dpr,
      height: rect.height * dpr,
    };
  }

  /**
   * 更新 Webview 位置和大小
   */
  async function updateBounds() {
    if (!containerElement || !isMounted) return;
    const rect = getPhysicalRect(containerElement);
    try {
      await invoke("update_inline_plugin_bounds", { rect });
    } catch (error) {
      console.error("[PluginInlineView] Failed to update bounds:", error);
    }
  }

  /**
   * 显示 Webview
   */
  async function showWebview() {
    if (!containerElement) {
      console.error("[PluginInlineView] Container element not found");
      return;
    }

    // Revert to strict logic
    const rect = getPhysicalRect(containerElement);

    // Access targetUrl safely
    let safeUrl = "";
    try {
      safeUrl = targetUrl;
    } catch (e) {
      console.error(`Failed to read targetUrl: ${e}`);
      throw e;
    }

    try {
      await invoke("show_inline_plugin", {
        url: safeUrl,
        pluginId: pluginId,
        rect,
      });

      // 发送运行时初始化数据
      // 必须在 show_inline_plugin 之后发送，确保 webview 已经存在
      sendRuntimeInit();

      // [Fix] onLoad 应该等待 backend 的 on_page_load 事件，而不是立即触发
      // onLoad?.();
    } catch (error) {
      console.error("[PluginInlineView] Failed to show webview:", error);
    }
  }

  /**
   * 发送生命周期事件给插件
   */
  export async function sendLifecycleEvent(
    event: "show" | "hide" | "focus" | "blur",
  ) {
    try {
      // 使用后端命令发送消息到 Native Webview
      await invoke("send_inline_plugin_message", {
        message: {
          type: "plugin-lifecycle-event",
          event,
        },
      });
      console.log("[PluginInlineView] Sent lifecycle event:", event);
    } catch (err) {
      console.error("[PluginInlineView] Failed to send lifecycle event:", err);
    }
  }

  // 监听 URL 变化
  $effect(() => {
    if (isMounted && targetUrl) {
      showWebview();
    }
  });

  let rectCheckLoop: number | null = null;
  let lastRectJson = "";

  onMount(() => {
    isMounted = true;

    // 监听 Native Webview 加载完成事件
    listen("plugin-inline-loaded", () => {
      console.log("[PluginInlineView] Native webview loaded");
      onLoad?.();
      // Also send init again just in case? No, showWebview sent it.
      // But if load happened, maybe context cleared?
      // Yes, on load, JS context is reset. So we MUST send init AFTER load.
      sendRuntimeInit();
    }).then((u) => (unlistenLoaded = u));

    // 初始化显示
    // 使用 requestAnimationFrame 确保布局已完成
    requestAnimationFrame(() => {
      try {
        showWebview().catch((e) => {
          console.error(`showWebview promise rejected: ${e}`);
        });

        // 初始化后发送 runtime-init (虽然 native bridge 可能会自己处理?)
        // 运行时初始化改由宿主消息完成，这里不再依赖旧桥接路径。
        // 所以我们需要通过 send_inline_plugin_message 发送 init data
        // Moved to showWebview to ensure it sends on every load
        // sendRuntimeInit();
      } catch (e) {
        console.error(`Error in RAF callback: ${e}`);
      }
    });

    // 监听大小变化
    if (containerElement) {
      resizeObserver = new ResizeObserver(() => {
        updateBounds();
      });
      resizeObserver.observe(containerElement);
    }

    // 轮询检查位置变化 (ResizeObserver 无法检测仅仅位置改变的情况)
    const checkLoop = () => {
      if (!isMounted) return;
      updateBoundsIfChanged();
      rectCheckLoop = requestAnimationFrame(checkLoop);
    };
    rectCheckLoop = requestAnimationFrame(checkLoop);
  });

  onDestroy(() => {
    isMounted = false;
    resizeObserver?.disconnect();
    if (rectCheckLoop) cancelAnimationFrame(rectCheckLoop);

    if (unlistenLoaded) unlistenLoaded();

    // 组件卸载时仅隐藏，避免误销毁后台保活的插件实例
    invoke("hide_inline_plugin").catch(console.error);
  });

  async function updateBoundsIfChanged() {
    if (!containerElement) return;
    const rect = getPhysicalRect(containerElement);
    const rectJson = JSON.stringify(rect);
    if (rectJson !== lastRectJson) {
      lastRectJson = rectJson;
      updateBounds();
    }
  }

  function sendRuntimeInit() {
    const runtimeInit = {
      type: "plugin-runtime-init",
      runtime: {
        mode: "inline" as const,
        pluginId: pluginId || "unknown",
        version: "0.1.0",
        mainWindowLabel: "main",
      },
    };
    invoke("send_inline_plugin_message", { message: runtimeInit }).catch(
      console.error,
    );
  }
</script>

<div
  bind:this={containerElement}
  class="relative h-full w-full bg-transparent"
  role="none"
></div>
