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
    Clock,
    Plus,
    Trash,
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
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";

  interface Message {
    role: "user" | "assistant";
    contentText: string;
  }

  interface ChatSessionMeta {
    id: string;
    title: string;
    provider_name: string;
    model_name: string;
    created_at: number;
    updated_at: number;
  }

  // 状态变量
  let messages = $state<Message[]>([]);
  let inputValue = $state("");
  let isGenerating = $state(false);
  let isConfigured = $state<boolean | null>(null); // null = 检查中, true = 已配置, false = 未配置
  let activeProviderName = $state("");
  let activeModelName = $state("");
  let activeEventId = $state("");
  let currentCleanup = $state<(() => void) | null>(null);

  // 历史记录状态
  let sessions = $state<ChatSessionMeta[]>([]);
  let currentSessionId = $state<string>("");
  let currentSessionTitle = $state<string>("");
  let currentSessionCreatedAt = $state<number>(0);
  let showHistory = $state(false);

  // 确认对话框状态
  let confirmDialogOpen = $state(false);
  let confirmDialogTitle = $state("");
  let confirmDialogDescription = $state("");
  let pendingAction = $state<(() => void | Promise<void>) | null>(null);

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

  // 辅助函数：生成简易 UUID
  function generateUUID() {
    if (
      typeof crypto !== "undefined" &&
      typeof crypto.randomUUID === "function"
    ) {
      return crypto.randomUUID();
    }
    return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(
      /[xy]/g,
      function (c) {
        const r = (Math.random() * 16) | 0;
        const v = c === "x" ? r : (r & 0x3) | 0x8;
        return v.toString(16);
      },
    );
  }

  // 辅助函数：格式化时间戳
  function formatTime(timestamp: number): string {
    const now = Date.now();
    const diff = now - timestamp;
    if (diff < 60000) return "刚刚";
    const minutes = Math.floor(diff / 60000);
    if (minutes < 60) return `${minutes} 分钟前`;
    const hours = Math.floor(diff / 3600000);
    if (hours < 24) return `${hours} 小时前`;
    const days = Math.floor(diff / 86400000);
    if (days === 1) return "昨天";
    if (days < 7) return `${days} 天前`;
    const date = new Date(timestamp);
    return `${date.getMonth() + 1}/${date.getDate()}`;
  }

  // 接收从 URL 传来的 query 参数
  onMount(async () => {
    // 检查 AI 提供商配置
    await checkAIConfig();

    // 如果已配置，加载历史记录
    if (isConfigured) {
      await loadSessions();

      const params = new URLSearchParams(window.location.search);
      const queryText = params.get("q");
      if (queryText) {
        initNewSession();
        inputValue = queryText;
        await handleSend();
      } else {
        // 如果有历史会话，默认加载最新一条会话以保持状态
        if (sessions.length > 0) {
          await selectSession(sessions[0].id);
        } else {
          initNewSession();
        }
      }
    }
  });

  onDestroy(() => {
    if (currentCleanup) {
      currentCleanup();
    }
    activeEventId = "";
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

  // 加载会话元数据列表
  async function loadSessions() {
    try {
      sessions = await invoke<ChatSessionMeta[]>("get_ai_sessions_index");
    } catch (e) {
      console.error("[AI Extension] Failed to load sessions:", e);
    }
  }

  // 初始化一个全新的会话
  function initNewSession() {
    if (isGenerating) return;
    currentSessionId = generateUUID();
    currentSessionTitle = "";
    currentSessionCreatedAt = Date.now();
    messages = [];
    showHistory = false;
  }

  // 选择并读取历史会话
  async function selectSession(id: string) {
    if (isGenerating) {
      toast.error("AI 正在思考中，请先等待回答结束");
      return;
    }
    try {
      const session = await invoke<any>("get_ai_session", { id });
      currentSessionId = session.id;
      currentSessionTitle = session.title;
      currentSessionCreatedAt = session.created_at;
      messages = session.messages.map((m: any) => ({
        role: m.role,
        contentText: m.content,
      }));
      showHistory = false;
    } catch (e) {
      console.error("[AI Extension] Failed to load session:", e);
      toast.error("加载对话历史失败");
    }
  }

  // 保存当前会话到本地文件
  async function saveCurrentSession(reloadIndex = true) {
    if (!currentSessionId || messages.length === 0) return;

    if (!currentSessionTitle) {
      const firstUserMsg = messages.find((m) => m.role === "user");
      if (firstUserMsg) {
        const text = firstUserMsg.contentText.trim();
        currentSessionTitle =
          text.slice(0, 15) + (text.length > 15 ? "..." : "");
      } else {
        currentSessionTitle = "新对话";
      }
    }

    try {
      const session = {
        id: currentSessionId,
        title: currentSessionTitle,
        provider_name: activeProviderName,
        model_name: activeModelName,
        created_at: currentSessionCreatedAt || Date.now(),
        updated_at: Date.now(),
        messages: messages.map((m) => ({
          role: m.role,
          content: m.contentText,
        })),
      };
      await invoke("save_ai_session", { session });
      if (reloadIndex) {
        await loadSessions();
      }
    } catch (e) {
      console.error("[AI Extension] Failed to save session:", e);
    }
  }

  // 删除单条会话
  function deleteSession(id: string, event: Event) {
    event.stopPropagation();
    if (isGenerating && id === currentSessionId) {
      toast.error("当前会话正在回答中，无法删除");
      return;
    }

    const session = sessions.find((s) => s.id === id);
    const sessionTitle = session?.title || "未命名对话";

    confirmDialogTitle = "确认删除对话";
    confirmDialogDescription = `确定要删除对话“${sessionTitle}”吗？此操作无法撤销。`;
    pendingAction = async () => {
      try {
        await invoke("delete_ai_session", { id });
        toast.success("会话已删除");
        await loadSessions();

        if (id === currentSessionId) {
          if (sessions.length > 0) {
            await selectSession(sessions[0].id);
          } else {
            initNewSession();
          }
        }
      } catch (e) {
        console.error("[AI Extension] Failed to delete session:", e);
        toast.error("删除会话失败");
      }
    };
    confirmDialogOpen = true;
  }

  // 一键清空所有历史记录
  function clearAllSessions() {
    if (isGenerating) {
      toast.error("AI 正在思考中，无法清空历史");
      return;
    }
    confirmDialogTitle = "确认清空历史记录";
    confirmDialogDescription =
      "确定要清空所有的历史对话记录吗？此操作无法撤销。";
    pendingAction = async () => {
      try {
        await invoke("clear_all_ai_sessions");
        toast.success("所有会话历史已清空");
        sessions = [];
        initNewSession();
        showHistory = false;
      } catch (e) {
        console.error("[AI Extension] Failed to clear all sessions:", e);
        toast.error("清空历史记录失败");
      }
    };
    confirmDialogOpen = true;
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

    // 先触发一次本地紧急保存，确保即使中途退出也能保存用户的问题
    await saveCurrentSession(false);

    // 生成流式事件 ID并记录
    const eventId = `ai-stream-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
    activeEventId = eventId;

    // 格式化为后端 AIManager 能解析 of ChatMessage[]
    const chatMessages = messages.slice(0, aiMsgIndex).map((msg) => ({
      role: msg.role,
      content: [{ type: "text", text: msg.contentText }],
    }));

    // 注册流式区块监听
    const unlistenChunk = await listen<string>(eventId, (event) => {
      messages[aiMsgIndex].contentText += event.payload;
      messages = [...messages]; // 触发 Svelte 响应式更新
    });

    // 注册完成监听
    const unlistenDone = await listen(eventId + "_done", async () => {
      if (currentCleanup) currentCleanup();
      isGenerating = false;
      await saveCurrentSession(); // 流生成结束，进行最终完整持久化
    });

    // 注册错误监听
    const unlistenError = await listen<string>(
      eventId + "_error",
      async (event) => {
        messages[aiMsgIndex].contentText +=
          `\n\n⚠️ **流传输中断**: ${event.payload}`;
        messages = [...messages];
        if (currentCleanup) currentCleanup();
        isGenerating = false;
        toast.error(`AI 响应错误: ${event.payload}`);
        await saveCurrentSession(); // 报错也保存状态
      },
    );

    currentCleanup = () => {
      unlistenChunk();
      unlistenDone();
      unlistenError();
      currentCleanup = null;
      activeEventId = "";
    };

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
      if (currentCleanup) currentCleanup();
      isGenerating = false;
      toast.error(`请求失败: ${err}`);
      await saveCurrentSession();
    }
  }

  // 中止当前 AI 流式生成任务
  async function handleStopGeneration() {
    if (!isGenerating || !activeEventId) return;

    try {
      // 调用后端终止接口
      await invoke("abort_ai_stream", { eventId: activeEventId });

      // 🐞 竞态检查：流可能已自然结束
      if (!isGenerating) return;

      // 手动触发前端监听清理
      if (currentCleanup) {
        currentCleanup();
      }

      // 在最后一条消息追加中止说明
      const lastMsgIndex = messages.length - 1;
      if (lastMsgIndex >= 0 && messages[lastMsgIndex].role === "assistant") {
        messages[lastMsgIndex].contentText +=
          "\n\n⚠️ **流传输已由用户手动中止**";
        messages = [...messages];
      }

      isGenerating = false;
      toast.success("已中止生成");
      await saveCurrentSession();
    } catch (e) {
      console.error("[AI Extension] Failed to abort AI stream:", e);
      // ⚠️ 保证在异常抛出时重置状态，避免永久卡死生成态
      if (currentCleanup) {
        currentCleanup();
      }
      isGenerating = false;
      toast.error("中止生成失败");
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
      <!-- 历史记录按钮 -->
      <Button.Root
        class="flex h-8 w-8 items-center justify-center rounded-lg border border-neutral-200 bg-white text-neutral-600 transition-colors hover:bg-neutral-100 active:scale-95 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800 {showHistory
          ? 'border-neutral-300 bg-neutral-100 dark:border-neutral-700 dark:bg-neutral-800'
          : ''}"
        onclick={() => (showHistory = !showHistory)}
        aria-label="历史对话"
      >
        <Clock size={15} />
      </Button.Root>

      <!-- 新建对话按钮 -->
      <Button.Root
        class="flex h-8 items-center gap-1 rounded-lg border border-neutral-200 bg-white px-2.5 text-xs text-neutral-600 transition-colors hover:bg-neutral-100 active:scale-95 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800"
        onclick={initNewSession}
        disabled={isGenerating}
      >
        <Plus size={13} weight="bold" />
        新对话
      </Button.Root>

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
    <!-- 历史记录侧边栏 -->
    <!-- 遮罩层 -->
    <button
      class="absolute inset-0 z-30 cursor-default border-none bg-neutral-950/15 backdrop-blur-[2px] transition-all duration-300 outline-none {showHistory
        ? 'pointer-events-auto opacity-100'
        : 'pointer-events-none opacity-0'}"
      onclick={() => (showHistory = false)}
      aria-label="关闭历史记录"
    ></button>

    <!-- 侧滑栏：采用毛玻璃特效和微光边框 -->
    <aside
      class="absolute top-0 bottom-0 left-0 z-40 flex w-72 flex-col border-r border-neutral-200/80 bg-white/95 shadow-2xl transition-all duration-300 ease-[cubic-bezier(0.16,1,0.3,1)] dark:border-neutral-800/80 dark:bg-neutral-950/95 {showHistory
        ? 'translate-x-0'
        : '-translate-x-full'}"
    >
      <!-- 侧边栏头部 -->
      <div
        class="flex items-center justify-between border-b border-neutral-200/50 p-4 dark:border-neutral-800/50"
      >
        <span
          class="text-xs font-semibold text-neutral-500 dark:text-neutral-400"
          >历史对话记录</span
        >
        {#if sessions.length > 0}
          <button
            class="flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] text-neutral-400 hover:bg-neutral-100 hover:text-red-500 dark:hover:bg-neutral-900"
            onclick={clearAllSessions}
          >
            <Trash size={10} />
            全部清空
          </button>
        {/if}
      </div>

      <!-- 历史会话列表 -->
      <div class="scrollbar-thin flex-1 space-y-1 overflow-y-auto p-2">
        {#if sessions.length === 0}
          <div
            class="flex h-40 flex-col items-center justify-center text-center text-xs text-neutral-400 select-none"
          >
            <Clock size={20} class="mb-2 text-neutral-300" />
            暂无历史对话记录
          </div>
        {:else}
          {#each sessions as session (session.id)}
            <div
              role="button"
              tabindex="0"
              class="group relative flex w-full cursor-pointer flex-col gap-1 rounded-lg px-3 py-2.5 text-left transition-all hover:bg-neutral-100 active:scale-98 dark:hover:bg-neutral-900/60 {currentSessionId ===
              session.id
                ? 'border-l-2 border-neutral-900 bg-neutral-100/80 dark:border-neutral-100 dark:bg-neutral-900/80'
                : ''}"
              onclick={() => selectSession(session.id)}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  selectSession(session.id);
                }
              }}
            >
              <!-- 标题与删除按钮 -->
              <div class="flex items-start justify-between gap-2">
                <span
                  class="line-clamp-1 text-xs font-medium text-neutral-800 group-hover:pr-6 dark:text-neutral-200"
                >
                  {session.title || "未命名对话"}
                </span>
                <!-- 悬浮删除键 -->
                <button
                  class="absolute top-2.5 right-2 hidden rounded p-1 text-neutral-400 group-hover:block hover:bg-neutral-200 hover:text-red-500 dark:hover:bg-neutral-800"
                  onclick={(e) => deleteSession(session.id, e)}
                  aria-label="删除会话"
                >
                  <Trash size={12} />
                </button>
              </div>
              <!-- 详情小标 -->
              <div
                class="flex items-center gap-1.5 text-[9px] text-neutral-400"
              >
                <span
                  class="py-0.2 rounded bg-neutral-200/50 px-1 font-mono dark:bg-neutral-800"
                  >{session.model_name}</span
                >
                <span>•</span>
                <span>{formatTime(session.updated_at)}</span>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </aside>

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
          <!-- 采用 bits-ui Button.Root 构建发送提交键 / 中止键 -->
          {#if isGenerating}
            <Button.Root
              type="button"
              onclick={handleStopGeneration}
              class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-red-500 text-white shadow-sm transition-all hover:bg-red-600 active:scale-95 dark:bg-red-600 dark:hover:bg-red-700"
              aria-label="停止生成"
            >
              <div class="h-3 w-3 rounded-xs bg-white"></div>
            </Button.Root>
          {:else}
            <Button.Root
              type="submit"
              disabled={!inputValue.trim()}
              class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-neutral-900 text-neutral-50 shadow-sm transition-all hover:bg-neutral-900/90 active:scale-95 disabled:cursor-not-allowed disabled:bg-neutral-100 disabled:text-neutral-400 dark:bg-neutral-50 dark:text-neutral-900 dark:hover:bg-neutral-50/90 dark:disabled:bg-neutral-800 dark:disabled:text-neutral-600"
              aria-label="发送消息"
            >
              <PaperPlaneTilt size={16} weight="bold" />
            </Button.Root>
          {/if}
        </form>
      </div>
    </div>
  {/if}
</div>

<ConfirmDialog
  bind:open={confirmDialogOpen}
  title={confirmDialogTitle}
  description={confirmDialogDescription}
  onConfirm={async () => {
    if (pendingAction) {
      await pendingAction();
      pendingAction = null;
    }
  }}
  onCancel={() => {
    pendingAction = null;
  }}
/>

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

  /* 自定义侧边栏滚动条以显得极为精致 */
  .scrollbar-thin::-webkit-scrollbar {
    width: 4px;
  }
  .scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
  }
  .scrollbar-thin::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.15);
    border-radius: 99px;
  }
  :global(.dark) .scrollbar-thin::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.12);
  }
</style>
