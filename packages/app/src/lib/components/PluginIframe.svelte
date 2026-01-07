<script lang="ts">
  /**
   * PluginIframe Component
   *
   * 插件 iframe 容器组件
   * 负责加载和管理插件的 iframe
   */
  interface Props {
    url: string;
    onLoad?: () => void;
  }

  let { url, onLoad }: Props = $props();

  let iframeElement = $state<HTMLIFrameElement | null>(null);

  // 暴露 iframe 元素给父组件
  export function getElement(): HTMLIFrameElement | null {
    return iframeElement;
  }

  /**
   * 发送生命周期事件给插件
   */
  export function sendLifecycleEvent(event: "show" | "hide") {
    if (iframeElement?.contentWindow) {
      iframeElement.contentWindow.postMessage(
        {
          type: "plugin-lifecycle-event",
          event,
        },
        "*",
      );
    }
  }
</script>

<iframe
  bind:this={iframeElement}
  src={url}
  class="h-full w-full border-0"
  title="Plugin"
  allow="clipboard-read; clipboard-write"
  onload={() => {
    // 加载完成后发送 show 事件
    sendLifecycleEvent("show");
    onLoad?.();
  }}
></iframe>
