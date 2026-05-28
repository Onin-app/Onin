<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";
  import { Button } from "bits-ui";
  import {
    PaperPlaneTilt,
    Sparkle,
    ArrowLeft,
    Gear,
    Warning,
    Spinner,
    ArrowCounterClockwise,
  } from "phosphor-svelte";
  import {
    IncremarkContent,
    ThemeProvider,
    AutoScrollContainer,
  } from "@incremark/svelte";
  import "@incremark/theme/styles.css";
  import { get } from "svelte/store";
  import { theme as globalTheme, getTheme } from "$lib/utils/theme";
  import { Theme } from "$lib/type";

  interface Message {
    role: "user" | "assistant";
    contentText: string;
  }

  // 状态变量
  let messages = $state<Message[]>([]);
  let inputValue = $state("");
  let isGenerating = $state(false);
  let isConfigured = $state<boolean | null>(null); // null = 检查中, true = 已配置, false = 未配置
  let activeProviderName = $state("");
  let activeModelName = $state("");

  // 动态确定 ThemeProvider 主题 (与 globalTheme 强力同步，杜绝延迟)
  const currentGlobalTheme = get(globalTheme);
  const resolvedInitialTheme = getTheme(currentGlobalTheme);
  let resolvedTheme = $state<"dark" | "default">(
    resolvedInitialTheme === Theme.DARK ? "dark" : "default",
  );

  $effect(() => {
    const unsubscribe = globalTheme.subscribe((value) => {
      const resolved = getTheme(value);
      resolvedTheme = resolved === Theme.DARK ? "dark" : "default";
    });
    return unsubscribe;
  });

  // 接收从 URL 传来的 query 参数
  onMount(async () => {
    // 检查 AI 提供商配置
    await checkAIConfig();

    // 如果已配置，检查 URL 参数并进行首轮自动提问
    if (isConfigured) {
      const params = new URLSearchParams(window.location.search);
      const queryText = params.get("q");
      if (queryText) {
        inputValue = queryText;
        await handleSend();
      }
    }
  });

  // 检查 AI 配置
  async function checkAIConfig() {
    try {
      const config = await invoke<any>("get_ai_config");
      const activeId = config.active_provider_id;
      const providers = config.providers || [];

      if (!activeId || providers.length === 0) {
        isConfigured = false;
        return;
      }

      const activeProvider = providers.find((p: any) => p.id === activeId);
      if (!activeProvider) {
        isConfigured = false;
      } else {
        isConfigured = true;
        activeProviderName =
          activeProvider.display_name ||
          activeProvider.name ||
          "OpenAI Compatible";
        activeModelName = activeProvider.default_model || "未指定模型";
      }
    } catch (e) {
      console.error("[AI Extension] Failed to check AI config:", e);
      isConfigured = false;
    }
  }

  // 一键清空对话
  function clearChat() {
    if (isGenerating) return;
    messages = [];
  }

  // 发送消息与流式响应逻辑
  async function handleSend() {
    if (!inputValue.trim() || isGenerating || isConfigured === false) return;

    const userPrompt = inputValue.trim();
    inputValue = "";

    // 添加用户消息
    messages = [...messages, { role: "user", contentText: userPrompt }];

    // 预留 AI 的回答槽位
    const aiMsgIndex = messages.length;
    messages = [...messages, { role: "assistant", contentText: "" }];
    isGenerating = true;

    // 生成流式事件 ID
    const eventId = `ai-stream-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;

    // 格式化为后端 AIManager 能解析 of ChatMessage[]
    const chatMessages = messages.slice(0, aiMsgIndex).map((msg) => ({
      role: msg.role,
      content: [{ type: "text", text: msg.contentText }],
    }));

    // 注册流式区块监听
    const unlistenChunk = await listen<string>(eventId, (event) => {
      messages[aiMsgIndex].contentText += event.payload;
      messages = [...messages]; // 触发响应式更新
    });

    // 注册完成监听
    const unlistenDone = await listen(eventId + "_done", () => {
      isGenerating = false;
      cleanup();
    });

    // 注册错误监听
    const unlistenError = await listen<string>(eventId + "_error", (event) => {
      messages[aiMsgIndex].contentText +=
        `\n\n⚠️ **流传输中断**: ${event.payload}`;
      messages = [...messages];
      isGenerating = false;
      cleanup();
      toast.error(`AI 响应错误: ${event.payload}`);
    });

    function cleanup() {
      unlistenChunk();
      unlistenDone();
      unlistenError();
    }

    try {
      const request = {
        model: null, // 后端默认使用 active provider 的 default_model
        messages: chatMessages,
        stream: true,
      };

      await invoke("plugin_ai_stream", { request, eventId });
    } catch (err: any) {
      messages[aiMsgIndex].contentText = `⚠️ **请求发起失败**: ${err}`;
      messages = [...messages];
      isGenerating = false;
      cleanup();
      toast.error(`请求失败: ${err}`);
    }
  }

  function handleBack() {
    goto("/");
  }

  function handleToConfig() {
    goto("/settings?tab=ai");
  }
</script>

<!-- 主容器：继承系统目前风格的卡片底板，摒弃花里胡哨，极简大气 -->
<div class="flex h-full w-full flex-col overflow-hidden" data-tauri-drag-region>
  <!-- 头部栏：Shadcn 风格极简头栏 -->
  <header
    class="z-10 flex h-14 shrink-0 items-center justify-between border-b border-neutral-200/80 bg-white/40 px-4 backdrop-blur-md dark:border-neutral-800 dark:bg-neutral-950/20"
    data-tauri-drag-region
  >
    <div class="flex items-center gap-3">
      <!-- 采用 bits-ui Button.Root 构建返回键 -->
      <Button.Root
        class="flex h-9 w-9 items-center justify-center rounded-lg text-neutral-600 transition-colors hover:bg-neutral-200 active:scale-95 dark:text-neutral-400 dark:hover:bg-neutral-800"
        onclick={handleBack}
        aria-label="返回主界面 (ESC)"
      >
        <ArrowLeft size={18} weight="bold" />
      </Button.Root>
      <div class="flex items-center gap-2">
        <span
          class="text-sm font-semibold text-neutral-900 dark:text-neutral-100"
          >AI 助手</span
        >
        {#if isConfigured}
          <span
            class="rounded-md border border-neutral-200 bg-neutral-100 px-2 py-0.5 text-[10px] font-medium text-neutral-600 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-400"
          >
            {activeProviderName}
          </span>
        {/if}
      </div>
    </div>

    <!-- 顶栏操作区 -->
    <div class="flex items-center gap-2">
      {#if messages.length > 0}
        <!-- 采用 bits-ui Button.Root 构建清空对话键 -->
        <Button.Root
          class="flex h-8 items-center gap-1.5 rounded-lg border border-neutral-200 bg-white px-2.5 text-xs text-neutral-600 transition-colors hover:bg-neutral-100 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800"
          onclick={clearChat}
          disabled={isGenerating}
        >
          <ArrowCounterClockwise size={13} />
          清空对话
        </Button.Root>
      {/if}
      <!-- 采用 bits-ui Button.Root 构建设置键 -->
      <Button.Root
        class="flex h-8 w-8 items-center justify-center rounded-lg border border-neutral-200 bg-white text-neutral-600 transition-colors hover:bg-neutral-100 active:scale-95 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800"
        onclick={handleToConfig}
        aria-label="AI 配置"
      >
        <Gear size={16} />
      </Button.Root>
    </div>
  </header>

  <!-- 主体内容 -->
  <div class="relative flex-1 overflow-hidden">
    {#if isConfigured === null}
      <!-- 检查 AI 配置中 -->
      <div
        class="flex h-full w-full flex-col items-center justify-center gap-3"
      >
        <Spinner class="h-5 w-5 animate-spin text-neutral-500" />
        <span class="text-xs text-neutral-500">检测 AI 能力中...</span>
      </div>
    {:else if isConfigured === false}
      <!-- 智海迷航：未配置 Provider 引导状态 -->
      <div
        class="flex h-full w-full flex-col items-center justify-center px-6 text-center"
      >
        <div
          class="flex max-w-sm flex-col items-center rounded-2xl border border-neutral-200 bg-neutral-50/50 p-8 shadow-xs dark:border-neutral-800 dark:bg-neutral-900/30"
        >
          <div
            class="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-neutral-200/80 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
          >
            <Warning size={24} />
          </div>
          <h3
            class="mb-2 text-sm font-semibold text-neutral-900 dark:text-neutral-100"
          >
            未检测到 AI 模型
          </h3>
          <p class="mb-6 text-xs leading-relaxed text-neutral-500">
            您需要先在设置中配置一个支持 OpenAI 协议的提供商（如
            DeepSeek、OpenAI 或本地 Ollama），才能开启强大的 AI 问答指令。
          </p>
          <!-- 采用 bits-ui Button.Root 构建配置跳转键 -->
          <Button.Root
            class="inline-flex h-9 items-center justify-center gap-2 rounded-lg bg-neutral-900 px-4 text-xs font-semibold text-neutral-50 shadow-xs transition-colors hover:bg-neutral-900/90 active:scale-95 dark:bg-neutral-50 dark:text-neutral-900 dark:hover:bg-neutral-50/90"
            onclick={handleToConfig}
          >
            <Gear size={14} />
            配置 AI 能力
          </Button.Root>
        </div>
      </div>
    {:else}
      <!-- 采用 Incremark 官方高级自动滚动容器，带用户向上滚屏暂停等极致交互 -->
      <AutoScrollContainer
        enabled={isGenerating}
        class="ai-scroll-viewport h-full w-full overflow-y-auto px-4 py-4"
      >
        <div class="mx-auto max-w-3xl space-y-6 pb-24">
          {#if messages.length === 0}
            <!-- 欢迎引导 -->
            <div
              class="flex flex-col items-center justify-center py-16 text-center select-none"
            >
              <div
                class="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-neutral-200/60 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
              >
                <Sparkle weight="bold" size={20} />
              </div>
              <h2
                class="mb-1 text-sm font-semibold text-neutral-800 dark:text-neutral-200"
              >
                我是您的 AI 随身随心助手
              </h2>
              <p class="max-w-xs text-xs text-neutral-500">
                当前模型：<code
                  class="rounded border border-neutral-300/30 bg-neutral-200/50 px-1 py-0.5 font-mono text-[10px] text-neutral-700 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300"
                  >{activeModelName}</code
                >。<br />
                支持编写代码、解答问题、整理文本。
              </p>
            </div>
          {/if}

          <!-- 对话消息列表 -->
          {#each messages as msg, index}
            <div
              class="flex w-full flex-col {msg.role === 'user'
                ? 'items-end'
                : 'items-start'}"
            >
              <span
                class="mb-1 px-2 text-[10px] font-medium text-neutral-400 select-none"
              >
                {msg.role === "user" ? "您" : activeProviderName}
              </span>

              {#if msg.role === "user"}
                <!-- 用户对话气泡：Shadcn 经典黑白卡片 -->
                <div
                  class="rounded-2xl rounded-tr-none bg-neutral-900 px-4 py-2 text-sm leading-relaxed whitespace-pre-wrap text-neutral-50 shadow-xs select-text dark:bg-neutral-100 dark:text-neutral-900"
                >
                  {msg.contentText}
                </div>
              {:else}
                <!-- AI 对话气泡：极简灰框灰底卡片 -->
                <div
                  class="w-full max-w-[90%] rounded-2xl rounded-tl-none border border-neutral-200 bg-neutral-50/70 px-4 py-3 text-sm text-neutral-800 select-text dark:border-neutral-800 dark:bg-neutral-900/40 dark:text-neutral-200"
                >
                  {#if msg.contentText === "" && isGenerating && index === messages.length - 1}
                    <div
                      class="flex items-center gap-1.5 py-1 text-neutral-400"
                    >
                      <Spinner class="h-4 w-4 animate-spin text-neutral-500" />
                      <span class="font-mono text-xs">思考中...</span>
                    </div>
                  {:else}
                    <div class="incremark-container space-y-4 leading-relaxed">
                      <ThemeProvider theme={resolvedTheme}>
                        <IncremarkContent
                          content={msg.contentText}
                          isFinished={!isGenerating ||
                            index < messages.length - 1}
                        />
                      </ThemeProvider>
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
          <div id="chat-bottom" class="h-px"></div>
        </div>
      </AutoScrollContainer>
    {/if}
  </div>

  <!-- 底部悬浮输入区：转为标准的 flex 底栏流，100% 自然平铺，告别左下角缩水 Bug -->
  {#if isConfigured}
    <div
      class="shrink-0 border-t border-neutral-200/80 bg-white/50 p-4 backdrop-blur-md dark:border-neutral-800 dark:bg-neutral-950/20"
    >
      <div class="mx-auto max-w-3xl">
        <form
          class="relative flex items-center overflow-hidden rounded-xl border border-neutral-200 bg-white p-1.5 shadow-xs transition-all focus-within:border-neutral-400 dark:border-neutral-800 dark:bg-neutral-900 dark:focus-within:border-neutral-600"
          onsubmit={(e) => {
            e.preventDefault();
            handleSend();
          }}
        >
          <input
            type="text"
            bind:value={inputValue}
            placeholder={isGenerating
              ? "AI 正在思考中..."
              : "输入问题，回车发送..."}
            disabled={isGenerating}
            class="h-9 flex-1 border-none bg-transparent px-3 text-sm text-neutral-900 outline-none placeholder:text-neutral-400 focus:ring-0 focus:outline-none focus-visible:outline-none active:outline-none disabled:cursor-not-allowed disabled:opacity-50 dark:text-neutral-100 dark:placeholder:text-neutral-500"
          />
          <!-- 采用 bits-ui Button.Root 构建发送提交键 -->
          <Button.Root
            type="submit"
            disabled={!inputValue.trim() || isGenerating}
            class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-neutral-900 text-neutral-50 shadow-sm transition-all hover:bg-neutral-900/90 active:scale-95 disabled:cursor-not-allowed disabled:bg-neutral-100 disabled:text-neutral-400 dark:bg-neutral-50 dark:text-neutral-900 dark:hover:bg-neutral-50/90 dark:disabled:bg-neutral-800 dark:disabled:text-neutral-600"
          >
            {#if isGenerating}
              <Spinner class="h-4 w-4 animate-spin" />
            {:else}
              <PaperPlaneTilt size={16} weight="bold" />
            {/if}
          </Button.Root>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  /* 隐藏滚动条但保留滚动功能 */
  :global(.app-scroll-area) {
    scrollbar-width: none;
  }
  :global(.app-scroll-area::-webkit-scrollbar) {
    display: none;
  }

  /* 强行让 Incremark 代码块在任何主题模式下都保持极致好看的深色高亮背景（与官方示例及主流 AI 助手一致） */
  :global(.incremark-code) {
    --incremark-color-code-block-background: #0f172a !important; /* 经典深蓝 slate-900 */
    --incremark-color-code-header-background: #1e293b !important; /* slate-800 */
    --incremark-color-code-block-text: #f8fafc !important; /* slate-50 */
    --incremark-color-border-strong: #334155 !important;
    --incremark-color-text-tertiary: #94a3b8 !important;
  }
  :global(.incremark-code .code-btn) {
    color: #f8fafc !important;
  }
  :global(.incremark-code .language) {
    color: #94a3b8 !important;
  }
</style>
