<script lang="ts">
  /**
   * PluginIframe Component
   *
   * 插件 iframe 容器组件
   * 负责加载和管理插件的 iframe，并处理生命周期事件
   */
  interface Props {
    url: string;
    pluginId?: string;
    onLoad?: () => void;
  }

  let { url, pluginId = "", onLoad }: Props = $props();

  let iframeElement = $state<HTMLIFrameElement | null>(null);

  /**
   * 计算带有 mode 和 plugin_id 参数的 iframe URL
   * 这样 SDK 可以在初始化时立即从 URL 获取运行模式信息
   */
  const iframeSrc = $derived.by(() => {
    try {
      const urlObj = new URL(url);
      urlObj.searchParams.set("mode", "inline");
      if (pluginId) {
        urlObj.searchParams.set("plugin_id", pluginId);
      }
      return urlObj.toString();
    } catch {
      // 如果 URL 解析失败，添加查询参数的简单方式
      const separator = url.includes("?") ? "&" : "?";
      const params = `mode=inline${pluginId ? `&plugin_id=${encodeURIComponent(pluginId)}` : ""}`;
      return `${url}${separator}${params}`;
    }
  });

  // 暴露 iframe 元素给父组件
  export function getElement(): HTMLIFrameElement | null {
    return iframeElement;
  }

  /**
   * 发送运行时初始化信息给插件
   */
  function sendRuntimeInit() {
    if (!iframeElement?.contentWindow) return;

    const runtimeInit = {
      type: "plugin-runtime-init",
      runtime: {
        mode: "inline" as const,
        pluginId: pluginId || "unknown",
        version: "0.1.0",
        mainWindowLabel: "main",
      },
    };

    iframeElement.contentWindow.postMessage(runtimeInit, "*");
    console.log("[PluginIframe] Sent runtime init:", runtimeInit.runtime);
  }

  /**
   * 发送生命周期事件给插件
   */
  export function sendLifecycleEvent(
    event: "show" | "hide" | "focus" | "blur",
  ) {
    if (iframeElement?.contentWindow) {
      iframeElement.contentWindow.postMessage(
        {
          type: "plugin-lifecycle-event",
          event,
        },
        "*",
      );
      console.log("[PluginIframe] Sent lifecycle event:", event);
    }
  }

  /**
   * 处理 iframe 加载完成
   */
  function handleLoad() {
    // 发送运行时初始化
    sendRuntimeInit();
    // 发送初始 show 事件
    sendLifecycleEvent("show");
    // 通知父组件
    onLoad?.();
  }
</script>

<iframe
  bind:this={iframeElement}
  src={iframeSrc}
  class="h-full w-full border-0"
  title="Plugin"
  allow="clipboard-read; clipboard-write"
  onload={handleLoad}
></iframe>
