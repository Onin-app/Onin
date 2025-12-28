<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { Minus, Square, CornersIn, X, ArrowsIn } from "phosphor-svelte";

  import "../../index.css";

  // 获取当前窗口
  const currentWindow = getCurrentWebviewWindow();
  const windowLabel = currentWindow.label;

  // 从 URL 参数获取 plugin_id
  let pluginId = $state<string>("");
  let pluginName = $state<string>("");
  let pluginUrl = $state<string>("");
  let isMaximized = $state<boolean>(false);
  let pluginIframeElement = $state<HTMLIFrameElement | null>(null);

  // 解析 URL 参数
  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    pluginId = params.get("plugin_id") || "";

    if (!pluginId) {
      console.error("[PluginWindow] No plugin_id provided");
      return;
    }

    // 获取插件信息和 URL
    (async () => {
      try {
        const plugin = await invoke<any>("get_plugin_with_schema", {
          pluginId,
        });
        console.log("[PluginWindow] Plugin data:", plugin);
        console.log(
          "[PluginWindow] devMode:",
          plugin.devMode,
          "devServer:",
          plugin.devServer,
        );

        // LoadedPlugin 使用了 flatten，所以字段在顶层
        pluginName = plugin.name || "Unknown Plugin";

        // 根据开发模式决定使用哪个 URL
        if (plugin.devMode && plugin.devServer) {
          // 开发模式：使用开发服务器，添加 plugin_id 参数
          const url = new URL(plugin.devServer);
          url.searchParams.set("plugin_id", pluginId);
          pluginUrl = url.toString();
          console.log(
            "[PluginWindow] Loading plugin from dev server:",
            pluginName,
            pluginUrl,
          );
        } else {
          // 生产模式：使用插件服务器
          const port = await invoke<number>("get_plugin_server_port");
          pluginUrl = `http://127.0.0.1:${port}/plugin/${plugin.dir_name}/${plugin.entry}?mode=window&plugin_id=${pluginId}`;
          console.log("[PluginWindow] Loading plugin:", pluginName, pluginUrl);
        }
      } catch (error) {
        console.error("[PluginWindow] Failed to load plugin info:", error);
      }

      // 检查初始最大化状态
      isMaximized = await currentWindow.isMaximized();

      // BUG FIX: 移除 onResized 监听器，避免无限循环
      // 不再监听 resize 事件来更新 isMaximized 状态
      // 改为在按钮点击时手动更新状态

      // 监听来自 iframe 的消息（Tauri API 调用）
      window.addEventListener("message", handlePluginMessage);

      // 监听来自后端的窗口可见性事件，转发给 iframe 中的插件
      const { listen } = await import("@tauri-apps/api/event");
      const unlistenVisibility = await listen<boolean>(
        "window_visibility",
        (event) => {
          const iframe = document.querySelector("iframe");
          if (!iframe?.contentWindow || !pluginUrl) return;

          const eventType = event.payload ? "show" : "hide";
          try {
            const targetOrigin = new URL(pluginUrl).origin;
            iframe.contentWindow.postMessage(
              { type: "plugin-lifecycle-event", event: eventType },
              targetOrigin,
            );
          } catch (error) {
            console.error(
              "[PluginWindow] Failed to send lifecycle event:",
              error,
            );
          }
        },
      );

      return () => {
        unlistenVisibility();
        window.removeEventListener("message", handlePluginMessage);
      };
    })();
  });

  // 监听 iframe 加载完成，注入 plugin ID
  const handleIframeLoad = () => {
    const iframe = pluginIframeElement;
    if (!iframe?.contentWindow) return;

    // Send plugin ID to iframe via postMessage
    // Use specific origin for better security
    try {
      const targetOrigin = new URL(pluginUrl).origin;
      iframe.contentWindow.postMessage(
        { type: "set-plugin-id", pluginId },
        targetOrigin,
      );
    } catch (error) {
      console.error("[PluginWindow] Failed to send plugin ID:", error);
    }
  };

  // 处理来自插件 iframe 的 Tauri API 调用
  const handlePluginMessage = async (event: MessageEvent) => {
    if (event.data?.type !== "plugin-tauri-call") return;

    const { messageId, command, args } = event.data;
    const iframe = pluginIframeElement;
    if (!iframe?.contentWindow) return;

    try {
      let result;
      if (command === "invoke") {
        result = await invoke(args[0], args[1] || {});
      } else if (command === "emit") {
        // 暂不支持 emit
        throw new Error("emit not supported in window mode yet");
      } else if (command === "listen") {
        // 暂不支持 listen
        throw new Error("listen not supported in window mode yet");
      }

      iframe.contentWindow.postMessage({ messageId, result }, "*");
    } catch (error) {
      iframe.contentWindow.postMessage(
        {
          messageId,
          error: error instanceof Error ? error.message : String(error),
        },
        "*",
      );
    }
  };

  // 窗口控制函数
  const handleClose = async () => {
    await invoke("plugin_close_window", { label: windowLabel });
  };

  const handleMinimize = async () => {
    await invoke("plugin_minimize_window", { label: windowLabel });
  };

  // BUG FIX: 手动管理 isMaximized 状态，避免 onResized 无限循环
  const handleMaximize = async () => {
    if (isMaximized) {
      await invoke("plugin_unmaximize_window", { label: windowLabel });
      isMaximized = false;  // 手动更新状态
    } else {
      await invoke("plugin_maximize_window", { label: windowLabel });
      isMaximized = true;   // 手动更新状态
    }
  };

  // BUG FIX: 使用 currentWindow.startDragging() 而不是 invoke
  function handleTitlebarMouseDown(event: MouseEvent) {
    // 只在左键点击时拖动
    if (event.button !== 0) return;
    
    // 如果点击的是按钮或其他交互元素，不拖动
    const target = event.target as HTMLElement;
    if (target.closest('button') || target.closest('input') || target.closest('select')) {
      return;
    }
    
    // 使用 Tauri API 开始拖动（需要 core:window:allow-start-dragging 权限）
    currentWindow.startDragging();
  }

  const handleBackToInline = async () => {
    try {
      console.log("[PluginWindow] Switching back to inline mode");

      // 1. 先切换插件的 auto_detach 为 false
      await invoke("toggle_plugin_auto_detach", {
        pluginId,
        autoDetach: false,
      });
      console.log("[PluginWindow] Plugin auto_detach set to false");

      // 2. 获取主窗口并显示
      const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const mainWindow = await WebviewWindow.getByLabel("main");

      if (mainWindow) {
        console.log("[PluginWindow] Showing main window");
        await mainWindow.show();
        await mainWindow.setFocus();
      } else {
        console.error("[PluginWindow] Main window not found");
      }

      // 3. 在主窗口中以 inline 模式执行插件
      console.log("[PluginWindow] Executing plugin in inline mode");
      await invoke("execute_plugin_entry", { pluginId });

      // 4. 等待一下确保插件已经在主窗口中显示
      await new Promise((resolve) => setTimeout(resolve, 200));

      // 5. 最后关闭当前独立窗口
      console.log("[PluginWindow] Closing plugin window");
      await invoke("plugin_close_window", { label: windowLabel });
    } catch (error) {
      console.error("[PluginWindow] Failed to switch to inline mode:", error);
    }
  };
</script>

<svelte:head>
  <style>
    html,
    body {
      margin: 0;
      padding: 0;
      width: 100%;
      height: 100%;
      overflow: hidden;
    }
  </style>
</svelte:head>

<div class="plugin-window-container">
  <!-- 自定义顶栏 -->
  <div class="titlebar" onmousedown={handleTitlebarMouseDown}>
    <!-- 插件标题 -->
    <div class="titlebar-title">
      {pluginName || "Plugin"}
    </div>

    <!-- 窗口控制按钮 -->
    <div class="titlebar-controls">
      <button
        onclick={handleBackToInline}
        class="titlebar-button titlebar-button-inline"
        type="button"
        title="切换到主窗口模式"
        aria-label="切换到主窗口模式"
      >
        <ArrowsIn size={16} weight="bold" />
      </button>

      <div class="titlebar-separator"></div>

      <button
        onclick={handleMinimize}
        class="titlebar-button"
        type="button"
        title="最小化"
        aria-label="最小化"
      >
        <Minus size={16} weight="bold" />
      </button>

      <button
        onclick={handleMaximize}
        class="titlebar-button"
        type="button"
        title={isMaximized ? "还原" : "最大化"}
        aria-label={isMaximized ? "还原" : "最大化"}
      >
        {#if isMaximized}
          <CornersIn size={16} weight="bold" />
        {:else}
          <Square size={16} weight="bold" />
        {/if}
      </button>

      <button
        onclick={handleClose}
        class="titlebar-button titlebar-button-close"
        type="button"
        title="关闭"
        aria-label="关闭"
      >
        <X size={16} weight="bold" />
      </button>
    </div>
  </div>

  <!-- 插件内容区域 (iframe) -->
  <div class="plugin-content">
    {#if pluginUrl}
      <iframe
        bind:this={pluginIframeElement}
        src={pluginUrl}
        title={pluginName}
        allow="clipboard-read; clipboard-write"
        onload={handleIframeLoad}
      ></iframe>
    {:else}
      <div class="loading-container">
        <div class="loading-spinner"></div>
        <span>Loading plugin...</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .plugin-window-container {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: hsl(var(--background));
  }

  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 0 16px;
    background: hsl(var(--background));
    border-bottom: 1px solid hsl(var(--border) / 0.5);
    flex-shrink: 0;
    backdrop-filter: blur(10px);
  }

  .titlebar-title {
    flex: 1;
    font-size: 12px;
    font-weight: 600;
    color: hsl(var(--foreground) / 0.9);
    letter-spacing: 0.01em;
    user-select: none;
    -webkit-user-select: none;
  }

  .titlebar-controls {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .titlebar-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: hsl(var(--muted-foreground) / 0.7);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.12s cubic-bezier(0.4, 0, 0.2, 1);
    padding: 0;
  }

  .titlebar-button:hover {
    background: rgba(128, 128, 128, 0.2);
    color: hsl(var(--foreground));
  }

  .titlebar-button:active {
    transform: scale(0.95);
    background: rgba(128, 128, 128, 0.3);
  }

  .titlebar-button-close:hover {
    background: hsl(0 84% 60%) !important;
    color: white !important;
  }

  .titlebar-button-close:active {
    background: hsl(0 84% 50%) !important;
  }

  .titlebar-button-inline:hover {
    background: hsl(var(--primary) / 0.2);
    color: hsl(var(--primary));
  }

  .titlebar-button-inline:active {
    background: hsl(var(--primary) / 0.3);
  }

  .titlebar-separator {
    width: 1px;
    height: 16px;
    background: hsl(var(--border) / 0.3);
    margin: 0 4px;
  }

  .plugin-content {
    flex: 1;
    overflow: hidden;
    background: hsl(var(--background));
  }

  .plugin-content iframe {
    width: 100%;
    height: 100%;
    border: none;
    display: block;
  }

  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid hsl(var(--primary) / 0.2);
    border-top-color: hsl(var(--primary));
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading-container span {
    font-size: 14px;
    color: hsl(var(--muted-foreground));
  }
</style>
