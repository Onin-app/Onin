<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Button, Combobox } from "bits-ui";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import { toast } from "svelte-sonner";
  import {
    Check,
    CaretUpDown,
    CaretDoubleUp,
    CaretDoubleDown,
    Plus,
    Sparkle,
    Cpu,
    Lightning,
  } from "phosphor-svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import MCPSettings from "$lib/components/settings/MCPSettings.svelte";
  import SkillsSettings from "$lib/components/settings/SkillsSettings.svelte";
  import PasswordInput from "$lib/components/PasswordInput.svelte";

  type TabId = "providers" | "mcp" | "skills";
  const tabs: { id: TabId; label: string; icon: any }[] = [
    { id: "providers", label: "提供商", icon: Sparkle },
    { id: "mcp", label: "MCP", icon: Cpu },
    { id: "skills", label: "Skills", icon: Lightning },
  ];
  let activeTab = $state<TabId>("providers");

  interface AIConfig {
    active_provider_id: string | null;
    providers: ProviderConfig[];
  }

  interface ProviderConfig {
    id: string;
    provider_type: string;
    name: string;
    display_name?: string | null;
    base_url: string;
    api_key: string | null;
    default_model: string | null;
    models?: ModelInfo[] | null;
  }

  interface ModelModalities {
    input: string[];
    output: string[];
  }

  interface ModelLimit {
    context?: number | null;
    output?: number | null;
  }

  interface RegistryModel {
    id: string;
    name: string;
    description?: string | null;
    attachment?: boolean;
    reasoning?: boolean;
    tool_call?: boolean;
    modalities?: ModelModalities;
    limit?: ModelLimit;
  }

  interface RegistryProvider {
    id: string;
    name: string;
    api?: string;
    doc?: string;
    models?: Record<string, RegistryModel> | RegistryModel[];
  }

  interface RemoteProvider {
    id: string;
    name: string;
    description: string;
    baseUrl: string;
    apiKeyUrl?: string;
    models: ModelInfo[];
  }

  interface ModelInfo {
    id: string;
    name: string;
    description?: string | null;
    context_window?: number | null;
    attachment?: boolean | null;
    reasoning?: boolean | null;
    tool_call?: boolean | null;
    modalities?: ModelModalities | null;
    limit?: ModelLimit | null;
  }

  let config = $state<AIConfig>({ active_provider_id: null, providers: [] });

  // Editing state
  let editingIndex = $state<number | null>(null); // null = not editing, -1 = adding new
  let editForm = $state<{
    id: string;
    provider_type: string;
    name: string;
    display_name: string | null;
    base_url: string;
    api_key: string | null;
    default_model: string | null;
    models: ModelInfo[] | null;
  }>({
    id: "",
    provider_type: "",
    name: "",
    display_name: null,
    base_url: "",
    api_key: null,
    default_model: null,
    models: null,
  });

  // Delete confirmation dialog state
  let deleteDialogOpen = $state(false);
  let pendingDeleteIndex = $state<number | null>(null);

  // Search states for comboboxes
  let providerSearch = $state("");
  let modelSearch = $state("");

  // Syncing states
  let isSyncingDirect = $state(false);

  function adaptRegistryData(
    data:
      | Record<string, RegistryProvider>
      | RegistryProvider[]
      | null
      | undefined,
  ): RemoteProvider[] {
    if (!data) return [];
    const rawProviders: RegistryProvider[] = Array.isArray(data)
      ? data
      : Object.values(data);
    try {
      const providersList: RemoteProvider[] = rawProviders.map((p) => {
        const rawModels = p.models || {};
        const modelList: RegistryModel[] = Array.isArray(rawModels)
          ? rawModels
          : Object.values(rawModels);
        const models = modelList.map((m) => {
          let cleanId = m.id;
          if (p.id !== "openrouter" && cleanId.includes("/")) {
            const parts = cleanId.split("/");
            cleanId = parts.slice(1).join("/");
          }
          return {
            id: cleanId,
            name: m.name,
            description: m.description || null,
            context_window: m.limit?.context || null,
            attachment: m.attachment ?? false,
            reasoning: m.reasoning ?? false,
            tool_call: m.tool_call ?? false,
            modalities: m.modalities ?? null,
            limit: m.limit ?? null,
          };
        });
        return {
          id: p.id,
          name: p.name,
          description: `来自 models.dev 的提供商`,
          baseUrl: p.api || "",
          apiKeyUrl: p.doc || "",
          models: models,
        };
      });

      // 收集所有模型的完整 ID，打平供 OpenRouter 使用
      const allOpenRouterModels: ModelInfo[] = [];
      for (const p of rawProviders) {
        if (p.models) {
          const modelList = Array.isArray(p.models)
            ? p.models
            : Object.values(p.models);
          for (const m of modelList) {
            allOpenRouterModels.push({
              id: m.id,
              name: `${p.name}: ${m.name}`,
              description: m.description || null,
              context_window: m.limit?.context || null,
              attachment: m.attachment ?? false,
              reasoning: m.reasoning ?? false,
              tool_call: m.tool_call ?? false,
              modalities: m.modalities ?? null,
              limit: m.limit ?? null,
            });
          }
        }
      }

      const existingOpenRouter = providersList.find(
        (p) => p.id === "openrouter",
      );
      if (existingOpenRouter) {
        existingOpenRouter.models = allOpenRouterModels;
        existingOpenRouter.description = "OpenRouter 路由聚合服务";
        existingOpenRouter.baseUrl = "https://openrouter.ai/api/v1";
        existingOpenRouter.apiKeyUrl = "https://openrouter.ai/settings/keys";
      } else {
        providersList.push({
          id: "openrouter",
          name: "OpenRouter",
          description: "OpenRouter 路由聚合服务",
          baseUrl: "https://openrouter.ai/api/v1",
          apiKeyUrl: "https://openrouter.ai/settings/keys",
          models: allOpenRouterModels,
        });
      }

      return providersList;
    } catch (err) {
      console.error("Failed to adapt models.dev registry data:", err);
      return [];
    }
  }

  let providersRegistry = $state<RemoteProvider[]>([]);
  let isSyncingRegistry = $state(false);
  let isRegistryLoading = $state(true);

  let availableProviders = $derived<RemoteProvider[]>(providersRegistry);

  let popularProviders = $derived(
    availableProviders.filter((p) => getProviderMeta(p.id).isPopular),
  );

  let otherProviders = $derived(
    availableProviders.filter((p) => !getProviderMeta(p.id).isPopular),
  );

  let selectedRemoteProvider = $derived(
    availableProviders.find((p) => p.id === editForm.provider_type),
  );

  let providerOptions = $derived(
    availableProviders.map((p) => ({ value: p.id, label: p.name })),
  );

  let filteredProviderOptions = $derived(
    providerSearch === ""
      ? providerOptions
      : providerOptions.filter((p) =>
          p.label.toLowerCase().includes(providerSearch.toLowerCase()),
        ),
  );

  interface ModelOption {
    value: string;
    label: string;
    model: ModelInfo;
  }

  let modelOptions = $derived.by(() => {
    // 1. 如果该配置实例已经有单独拉取并保存的 models，优先使用它
    if (editForm.models && editForm.models.length > 0) {
      return editForm.models.map(
        (m) =>
          ({
            value: m.id,
            label: m.name,
            model: m,
          }) satisfies ModelOption,
      );
    }

    // 2. 如果选定了服务提供商
    if (selectedRemoteProvider) {
      return selectedRemoteProvider.models.map(
        (m) =>
          ({
            value: m.id,
            label: m.name,
            model: m,
          }) satisfies ModelOption,
      );
    }

    return [];
  });

  let filteredModelOptions = $derived(
    modelSearch === ""
      ? modelOptions
      : modelOptions.filter((m) =>
          m.label.toLowerCase().includes(modelSearch.toLowerCase()),
        ),
  );

  interface ValidationResult {
    valid: boolean;
    message?: string;
    models_count?: number;
  }

  // 监听提供商类型变化以回填配置
  $effect(() => {
    const type = editForm.provider_type;
    if (type && editingIndex === -1) {
      const remote = availableProviders.find((p) => p.id === type);
      if (remote) {
        editForm.base_url = remote.baseUrl;
        editForm.name = remote.name.split(" ")[0];
        editForm.models = null;
        editForm.default_model =
          remote.models && remote.models.length > 0
            ? remote.models[0].id
            : null;
      }
    }
  });

  onMount(async () => {
    try {
      config = await invoke("get_ai_config");
    } catch (e) {
      console.error("Failed to load AI config", e);
      toast.error("Failed to load AI config");
    }

    try {
      const cachedRegistry = await invoke<any | null>("get_providers_registry");
      if (cachedRegistry) {
        providersRegistry = adaptRegistryData(cachedRegistry);
      }
    } catch (e) {
      console.error("Failed to load providers registry from backend", e);
    }

    // If still empty, auto-sync silently from online
    if (providersRegistry.length === 0) {
      await syncProvidersRegistry(false);
    }
    isRegistryLoading = false;
  });

  async function syncProvidersRegistry(showToast = true) {
    let toastId;
    if (showToast) {
      toastId = toast.loading("正在同步大模型商配置注册表...");
    }
    isSyncingRegistry = true;
    try {
      const latestRegistry = await invoke<any>("sync_providers_registry");
      providersRegistry = adaptRegistryData(latestRegistry);
      if (showToast) {
        toast.success("供应商注册表同步成功", { id: toastId });
      }
    } catch (e) {
      console.error(e);
      if (showToast) {
        toast.error(`配置注册表同步失败: ${e}`, { id: toastId });
      }
    } finally {
      isSyncingRegistry = false;
    }
  }

  function enrichModelsWithRegistry(fetchedModels: any[]): ModelInfo[] {
    const registryModelMap = new Map<string, ModelInfo>();

    for (const provider of providersRegistry) {
      for (const m of provider.models) {
        registryModelMap.set(m.id.toLowerCase(), m);
      }
    }

    return fetchedModels.map((m) => {
      const matchId = m.id.toLowerCase();
      const matched = registryModelMap.get(matchId);

      if (matched) {
        return {
          id: m.id,
          name: m.name || matched.name,
          description: m.description || matched.description || null,
          context_window:
            m.context_window ||
            matched.context_window ||
            matched.limit?.context ||
            null,
          attachment: m.attachment ?? matched.attachment ?? false,
          reasoning: m.reasoning ?? matched.reasoning ?? false,
          tool_call: m.tool_call ?? matched.tool_call ?? false,
          modalities: m.modalities ?? matched.modalities ?? null,
          limit: m.limit ?? matched.limit ?? null,
        };
      }

      return {
        id: m.id,
        name: m.name,
        description: m.description || null,
        context_window: m.context_window || null,
        attachment: m.attachment ?? false,
        reasoning: m.reasoning ?? false,
        tool_call: m.tool_call ?? false,
        modalities: m.modalities ?? null,
        limit: m.limit ?? null,
      };
    });
  }

  async function syncDirectModels() {
    if (!editForm.base_url) {
      toast.error("API 地址不能为空，无法拉取模型");
      return;
    }

    const toastId = toast.loading("正在获取提供商的模型列表...");
    isSyncingDirect = true;
    try {
      const fetched = await invoke<any[]>("fetch_ai_models_direct", {
        baseUrl: editForm.base_url,
        apiKey: editForm.api_key || null,
      });

      if (fetched && fetched.length > 0) {
        editForm.models = enrichModelsWithRegistry(fetched);
        toast.success(`成功拉取并缓存了 ${fetched.length} 个模型`, {
          id: toastId,
        });

        // 确保当前选中的 default_model 在拉取到的新列表中，如果不在，则自动重置为第一个
        if (fetched.length > 0) {
          const hasCurrentModel = fetched.some(
            (m) => m.id === editForm.default_model,
          );
          if (!hasCurrentModel) {
            editForm.default_model = fetched[0].id;
          }
        }
      } else {
        toast.error("未获取到任何可用模型", { id: toastId });
      }
    } catch (e) {
      console.error(e);
      toast.error(`拉取模型列表失败: ${e}`, { id: toastId });
    } finally {
      isSyncingDirect = false;
    }
  }

  function getModelInfo(provider: ProviderConfig): ModelInfo | undefined {
    if (provider.models && provider.models.length > 0) {
      const matched = provider.models.find(
        (m) => m.id === provider.default_model,
      );
      if (matched) return matched;
    }

    const remoteProvider = providersRegistry.find(
      (p) => p.id === provider.provider_type,
    );
    if (remoteProvider && remoteProvider.models) {
      const matched = remoteProvider.models.find(
        (m) => m.id === provider.default_model,
      );
      if (matched) return matched;
    }

    for (const p of providersRegistry) {
      if (p.models) {
        const matched = p.models.find((m) => m.id === provider.default_model);
        if (matched) return matched;
      }
    }

    return undefined;
  }

  function startAdd() {
    editingIndex = -1;
    providerSearch = "";
    modelSearch = "";
    editForm = {
      id: "",
      provider_type: "",
      name: "",
      display_name: null,
      base_url: "",
      api_key: null,
      default_model: null,
      models: null,
    };
  }

  interface ProviderMeta {
    description: string;
    isPopular: boolean;
    recommendTag?: string;
  }

  const PROVIDER_METAS: Record<string, ProviderMeta> = {
    deepseek: {
      description: "高性价比国产大模型服务，提供极佳的推理与通用能力",
      isPopular: true,
      recommendTag: "推荐",
    },
    openai: {
      description: "行业标杆，提供强大的 GPT 系列模型与多模态能力",
      isPopular: true,
      recommendTag: "推荐",
    },
    anthropic: {
      description: "业界顶尖的 Claude 系列模型，适合复杂推理与长文本",
      isPopular: true,
      recommendTag: "推荐",
    },
    google: {
      description: "Google Gemini 核心模型，多模态与长上下文优势明显",
      isPopular: true,
    },
    ollama: {
      description: "本地部署大模型服务的首选，完全免费，数据绝对安全隐私",
      isPopular: true,
    },
    openrouter: {
      description: "一站式 API 路由服务，可快速接入成百上千种前沿与开源模型",
      isPopular: true,
    },
    groq: {
      description: "超高速开源大模型推理引擎",
      isPopular: true,
    },
    together: {
      description: "大模型开源托管平台",
      isPopular: true,
    },
    mistral: {
      description: "欧洲主流的开源大模型先锋",
      isPopular: true,
    },
    cohere: {
      description: "企业级语言模型与重排服务",
      isPopular: true,
    },
    siliconflow: {
      description: "硅基流动高性能大模型服务平台",
      isPopular: true,
    },
    zhipu: {
      description: "智谱 AI 大模型开放平台",
      isPopular: true,
    },
    moonshot: {
      description: "月之暗面 (Kimi) 开放平台",
      isPopular: true,
    },
  };

  function getProviderMeta(id: string) {
    const key = id.toLowerCase();
    if (PROVIDER_METAS[key]) {
      return PROVIDER_METAS[key];
    }
    return {
      description: "通过标准 OpenAI 协议连接的其他 AI 大模型服务",
      isPopular: false,
    };
  }

  function connectRegistryProvider(remote: RemoteProvider) {
    editingIndex = -1;
    providerSearch = "";
    modelSearch = "";
    editForm = {
      id: "",
      provider_type: remote.id,
      name: remote.name,
      display_name: null,
      base_url: remote.baseUrl,
      api_key: null,
      default_model:
        remote.models && remote.models.length > 0 ? remote.models[0].id : null,
      models: null,
    };
  }

  function startEdit(index: number) {
    editingIndex = index;
    const provider = config.providers[index];
    editForm = {
      ...provider,
      display_name: provider.display_name ?? null,
      models: provider.models ?? null,
    };
  }

  function cancelEdit() {
    editingIndex = null;
    providerSearch = "";
    modelSearch = "";
  }

  // Generate unique ID for provider instance
  function generateProviderId(templateId: string): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 8);
    return `${templateId}_${timestamp}_${random}`;
  }

  async function testConnection() {
    if (!editForm.provider_type || !editForm.base_url) {
      toast.error("进行连接测试需要提供商类型和 API 地址");
      return;
    }

    const toastId = toast.loading("正在测试连接...");
    try {
      const validation = await invoke<ValidationResult>(
        "validate_ai_provider",
        {
          baseUrl: editForm.base_url,
          apiKey: editForm.api_key,
        },
      );

      if (validation.valid) {
        toast.success(
          `连接测试成功！已获取到 ${validation.models_count} 个模型。`,
          { id: toastId },
        );
      } else {
        toast.error(`连接测试失败: ${validation.message}`, {
          id: toastId,
        });
      }
    } catch (e) {
      toast.error(`测试连接时出错: ${e}`, { id: toastId });
    }
  }

  async function save() {
    // Validation
    if (!editForm.provider_type || !editForm.base_url) {
      toast.error("提供商类型和 API 地址是必填项");
      return;
    }

    const toastId = toast.loading("正在验证并保存...");

    try {
      const validation = await invoke<ValidationResult>(
        "validate_ai_provider",
        {
          baseUrl: editForm.base_url,
          apiKey: editForm.api_key,
        },
      );

      if (!validation.valid) {
        toast.error(`验证失败: ${validation.message}`, {
          id: toastId,
        });
        return;
      }

      toast.success("验证成功", { id: toastId });
    } catch (e) {
      console.error(e);
      toast.warning("无法验证连接，正在直接保存...", {
        id: toastId,
      });
    }

    // Generate unique ID for new providers, keep existing ID for edits
    const providerId =
      editingIndex === -1
        ? generateProviderId(editForm.provider_type)
        : config.providers[editingIndex!].id;

    // 根据 provider_type 决定默认的显示名称
    let defaultName = editForm.name;
    if (!defaultName) {
      const remote = availableProviders.find(
        (p) => p.id === editForm.provider_type,
      );
      if (remote) {
        defaultName = remote.name.split(" ")[0];
      } else {
        defaultName = "自定义直连";
      }
    }

    const newProvider: ProviderConfig = {
      id: providerId,
      provider_type: editForm.provider_type,
      name: defaultName,
      display_name: editForm.display_name || null,
      base_url: editForm.base_url,
      api_key: editForm.api_key || null,
      default_model: editForm.default_model || null,
      models: editForm.models || null,
    };

    if (editingIndex === -1) {
      // Adding new
      config.providers.push(newProvider);
    } else if (editingIndex !== null) {
      // Updating existing
      config.providers[editingIndex] = newProvider;
    }

    try {
      await invoke("update_ai_config", { config });
      toast.success("模型配置已保存");
      editingIndex = null;
      providerSearch = "";
      modelSearch = "";
    } catch (e) {
      console.error(e);
      toast.error("保存模型配置失败");
    }
  }

  async function deleteProvider(index: number) {
    const provider = config.providers[index];

    // Warn if deleting active provider - show dialog instead of system confirm
    if (provider.id === config.active_provider_id) {
      pendingDeleteIndex = index;
      deleteDialogOpen = true;
      return;
    }

    // Direct delete for non-active providers
    await performDelete(index);
  }

  async function performDelete(index: number) {
    const provider = config.providers[index];

    // Clear active provider if deleting the active one
    if (provider.id === config.active_provider_id) {
      config.active_provider_id = null;
    }

    config.providers.splice(index, 1);

    try {
      await invoke("update_ai_config", { config });
      toast.success("模型配置已删除");
    } catch (e) {
      console.error(e);
      toast.error("删除模型配置失败");
    }
  }

  function handleDeleteConfirm() {
    if (pendingDeleteIndex !== null) {
      performDelete(pendingDeleteIndex);
      pendingDeleteIndex = null;
    }
    deleteDialogOpen = false; // Close dialog after action
  }

  function handleDeleteCancel() {
    pendingDeleteIndex = null;
    deleteDialogOpen = false; // Close dialog
  }

  async function setActive(providerId: string) {
    config.active_provider_id = providerId;
    try {
      await invoke("update_ai_config", { config });
      toast.success("当前启用的模型已更新");
    } catch (e) {
      console.error(e);
      toast.error("更新启用模型失败");
    }
  }
</script>

<AppScrollArea class="h-full w-full" viewportClass="h-full w-full">
  <main class="h-full w-full pr-2 pb-8">
    <!-- Tab 导航 -->
    <div
      class="mb-6 flex gap-1 border-b border-neutral-200 dark:border-neutral-800"
    >
      {#each tabs as tab}
        {@const TabIcon = tab.icon}
        <button
          class="flex items-center gap-2 border-b-2 px-3 pb-2.5 text-sm font-medium transition-colors
              {activeTab === tab.id
            ? 'border-neutral-900 text-neutral-900 dark:border-neutral-100 dark:text-neutral-100'
            : 'border-transparent text-neutral-500 hover:text-neutral-700 dark:text-neutral-400 dark:hover:text-neutral-200'}"
          onclick={() => (activeTab = tab.id)}
        >
          <TabIcon size={15} />
          {tab.label}
        </button>
      {/each}
    </div>

    {#if activeTab === "mcp"}
      <MCPSettings />
    {:else if activeTab === "skills"}
      <SkillsSettings />
    {:else}
      {#if editingIndex === null}
        <div class="mb-6 flex items-center justify-between px-1">
          <div>
            <h2
              class="mb-1 text-sm font-semibold text-neutral-900 dark:text-neutral-100"
            >
              提供商
            </h2>
            <p class="text-xs text-neutral-500 dark:text-neutral-400">
              管理你的 AI 提供商与服务
            </p>
          </div>
          <div class="flex gap-2">
            <Button.Root
              class="inline-flex h-8 items-center justify-center gap-1.5 rounded-lg border border-neutral-200 bg-white px-3 text-xs font-medium text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
              disabled={isSyncingRegistry}
              onclick={() => syncProvidersRegistry(true)}
            >
              {isSyncingRegistry ? "正在同步..." : "同步模型列表"}
            </Button.Root>
          </div>
        </div>
      {/if}

      <!-- Provider List or Edit Form -->
      <div class="space-y-6">
        {#if editingIndex === null}
          <!-- 已连接的提供商分区 -->
          <div>
            <h3
              class="mb-3 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
            >
              已连接的提供商
            </h3>

            {#if config.providers.length === 0}
              <div
                class="rounded-xl border border-dashed border-neutral-200 bg-neutral-50/50 px-6 py-8 text-center dark:border-neutral-800 dark:bg-neutral-900/20"
              >
                <p class="text-xs text-neutral-400 dark:text-neutral-500">
                  暂无已连接的提供商
                </p>
              </div>
            {:else}
              <div
                class="divide-y divide-neutral-100 overflow-hidden rounded-xl border border-neutral-200 bg-white shadow-xs dark:divide-neutral-800/60 dark:border-neutral-800 dark:bg-neutral-900"
              >
                {#each config.providers as provider, index (provider.id)}
                  {@const defaultModelInfo = getModelInfo(provider)}
                  <div
                    class="group relative flex cursor-pointer items-center justify-between px-4 py-3 transition-colors hover:bg-neutral-50 dark:hover:bg-neutral-800/30"
                    onclick={() => setActive(provider.id)}
                  >
                    <!-- Left indicator & Info -->
                    <div class="flex min-w-0 items-center gap-2.5">
                      <!-- Active dot indicator -->
                      <div class="flex w-4 items-center justify-center">
                        {#if config.active_provider_id === provider.id}
                          <span class="relative flex h-2 w-2">
                            <span
                              class="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75"
                            ></span>
                            <span
                              class="relative inline-flex h-2 w-2 rounded-full bg-green-500"
                            ></span>
                          </span>
                        {:else}
                          <span
                            class="h-2 w-2 rounded-full bg-neutral-200 transition-colors group-hover:bg-neutral-300 dark:bg-neutral-700 dark:group-hover:bg-neutral-600"
                          ></span>
                        {/if}
                      </div>

                      <!-- Text details -->
                      <div class="min-w-0">
                        <div class="flex items-center gap-2">
                          <span
                            class="truncate text-sm font-medium text-neutral-900 dark:text-neutral-100"
                          >
                            {provider.display_name || provider.name}
                          </span>
                          <span
                            class="rounded border border-solid border-neutral-200/50 bg-neutral-100 px-1.5 py-0.5 text-[10px] font-medium text-neutral-500 dark:border-neutral-700/50 dark:bg-neutral-800 dark:text-neutral-400"
                          >
                            {provider.api_key ? "API 密钥" : "免密钥"}
                          </span>
                        </div>
                      </div>
                    </div>

                    <!-- Middle: Default Model display -->
                    <div class="hidden items-center gap-2 sm:flex">
                      {#if provider.default_model}
                        <code
                          class="rounded-md border border-neutral-200/60 bg-neutral-50 px-2 py-0.5 font-mono text-[11px] text-neutral-700 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-300"
                        >
                          {provider.default_model}
                        </code>
                      {/if}
                    </div>

                    <!-- Right actions -->
                    <div
                      class="flex items-center gap-3"
                      onclick={(e) => e.stopPropagation()}
                    >
                      <button
                        class="text-xs text-neutral-500 transition-colors hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-200"
                        onclick={() => startEdit(index)}
                      >
                        编辑
                      </button>
                      <span class="text-neutral-300 dark:text-neutral-800"
                        >|</span
                      >
                      <button
                        class="text-xs text-red-500 transition-colors hover:text-red-600 dark:text-red-400 dark:hover:text-red-300"
                        onclick={() => deleteProvider(index)}
                      >
                        断开连接
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- 热门提供商分区 -->
          <div class="space-y-2.5">
            <h3
              class="text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
            >
              热门提供商
            </h3>
            <div class="grid grid-cols-1 gap-2.5 sm:grid-cols-2 md:grid-cols-3">
              {#each popularProviders as rp (rp.id)}
                <div
                  class="flex items-center justify-between rounded-xl border border-neutral-200 bg-white p-2.5 shadow-xs transition-all hover:border-neutral-300 dark:border-neutral-800 dark:bg-neutral-900 dark:hover:border-neutral-700"
                >
                  <span
                    class="truncate pr-1 text-xs font-medium text-neutral-900 dark:text-neutral-100"
                  >
                    {rp.name}
                  </span>
                  <Button.Root
                    class="inline-flex h-7 shrink-0 items-center justify-center gap-0.5 rounded-lg border border-neutral-200 bg-white px-2.5 text-[11px] font-semibold text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                    onclick={() => connectRegistryProvider(rp)}
                  >
                    <Plus class="h-3 w-3" />
                    连接
                  </Button.Root>
                </div>
              {/each}
            </div>
          </div>

          <!-- 其他提供商分区 -->
          <div class="space-y-2.5 pt-2">
            <h3
              class="text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
            >
              其他可用提供商
            </h3>
            <div class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4">
              {#each otherProviders as rp (rp.id)}
                <div
                  class="flex items-center justify-between rounded-lg border border-neutral-200 bg-white p-2 shadow-2xs transition-all hover:border-neutral-300 dark:border-neutral-800 dark:bg-neutral-900 dark:hover:border-neutral-700"
                >
                  <span
                    class="truncate pr-1 text-xs font-medium text-neutral-800 dark:text-neutral-200"
                  >
                    {rp.name}
                  </span>
                  <Button.Root
                    class="shadow-3xs inline-flex h-6 shrink-0 items-center justify-center gap-0.5 rounded-md border border-neutral-200 bg-white px-2 text-[10px] font-semibold text-neutral-600 transition-colors hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-400 dark:hover:bg-neutral-700"
                    onclick={() => connectRegistryProvider(rp)}
                  >
                    <Plus class="h-2.5 w-2.5" />
                    连接
                  </Button.Root>
                </div>
              {/each}

              <!-- 手动连接自定义大模型 -->
              <div
                class="flex items-center justify-between rounded-lg border border-dashed border-neutral-200 bg-transparent p-2 transition-all hover:border-neutral-300 dark:border-neutral-800 dark:hover:border-neutral-700"
              >
                <span
                  class="truncate pr-1 text-xs font-medium text-neutral-800 dark:text-neutral-200"
                >
                  自定义直连服务
                </span>
                <Button.Root
                  class="shadow-3xs inline-flex h-6 shrink-0 items-center justify-center gap-0.5 rounded-md border border-neutral-200 bg-white px-2 text-[10px] font-semibold text-neutral-600 transition-colors hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-400 dark:hover:bg-neutral-700"
                  onclick={startAdd}
                >
                  <Plus class="h-2.5 w-2.5" />
                  连接
                </Button.Root>
              </div>
            </div>
          </div>
        {:else}
          <!-- Edit Form -->
          <div
            class="overflow-hidden rounded-xl border border-neutral-200 bg-white shadow-xs transition-all dark:border-neutral-800 dark:bg-neutral-900"
          >
            <!-- Header with name -->
            <div
              class="relative flex items-center justify-between border-b border-neutral-200/60 bg-neutral-50/50 px-4 py-3.5 dark:border-neutral-800/60 dark:bg-neutral-800/30"
            >
              <div class="flex items-center gap-2.5">
                <div>
                  <h3
                    class="text-sm font-semibold text-neutral-900 dark:text-neutral-100"
                  >
                    {editingIndex === -1 ? "连接新提供商" : "编辑提供商配置"}
                  </h3>
                  <p class="text-xs text-neutral-500 dark:text-neutral-400">
                    {editForm.provider_type
                      ? `正在配置 ${selectedRemoteProvider?.name || editForm.provider_type}`
                      : "选择一个大模型提供商开始连接"}
                  </p>
                </div>
              </div>

              <!-- 如果是新建且已经选了类型，允许用户清除重选 -->
              {#if editingIndex === -1 && editForm.provider_type}
                <button
                  type="button"
                  class="text-xs text-neutral-500 transition-colors hover:text-neutral-800 dark:text-neutral-400 dark:hover:text-neutral-200"
                  onclick={() => {
                    editForm.provider_type = "";
                    editForm.base_url = "";
                    editForm.name = "";
                    editForm.default_model = null;
                    editForm.models = null;
                  }}
                >
                  重新选择
                </button>
              {/if}
            </div>

            <div class="space-y-4 p-4">
              <!-- 1. 如果尚未选择提供商类型 (仅在新建且 provider_type 为空时显示) -->
              {#if !editForm.provider_type}
                <div>
                  <span
                    class="mb-1.5 block text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
                  >
                    服务提供商
                  </span>
                  <Combobox.Root
                    type="single"
                    name="provider"
                    inputValue={providerOptions.find(
                      (o) => o.value === editForm.provider_type,
                    )?.label || ""}
                    onOpenChange={(o) => {
                      if (!o) providerSearch = "";
                    }}
                    onValueChange={(v) => {
                      if (v) {
                        editForm.provider_type = v;
                        providerSearch = "";
                      }
                    }}
                  >
                    <div class="relative w-full">
                      <Combobox.Input
                        id="provider-type"
                        oninput={(e) =>
                          (providerSearch = e.currentTarget.value)}
                        class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm font-medium text-neutral-900 placeholder:text-neutral-500 focus:ring-2 focus:ring-neutral-950 focus:ring-offset-2 focus:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:ring-offset-neutral-950 dark:placeholder:text-neutral-400 dark:focus:ring-neutral-300"
                        placeholder="选择提供商"
                      />
                      <Combobox.Trigger
                        class="absolute top-1/2 right-3 -translate-y-1/2 text-neutral-400"
                      >
                        <CaretUpDown class="h-4 w-4" />
                      </Combobox.Trigger>
                    </div>

                    <Combobox.Portal>
                      <Combobox.Content
                        class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 max-h-64 w-[var(--bits-combobox-anchor-width)] overflow-hidden rounded-md border border-neutral-200 bg-white shadow-md dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-50"
                      >
                        <Combobox.ScrollUpButton
                          class="flex w-full items-center justify-center py-1 text-neutral-400"
                        >
                          <CaretDoubleUp class="h-3 w-3" />
                        </Combobox.ScrollUpButton>
                        <Combobox.Viewport class="p-1">
                          {#each filteredProviderOptions as option (option.value)}
                            <Combobox.Item
                              class="flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[highlighted]:bg-neutral-100 dark:data-[highlighted]:bg-neutral-800"
                              value={option.value}
                              label={option.label}
                            >
                              {#snippet children({ selected })}
                                <span class="flex-1">{option.label}</span>
                                {#if selected}
                                  <Check class="h-4 w-4" />
                                {/if}
                              {/snippet}
                            </Combobox.Item>
                          {:else}
                            {#if isRegistryLoading}
                              <div
                                class="px-2 py-3 text-center text-sm text-neutral-400"
                              >
                                正在加载模型列表...
                              </div>
                            {:else if providersRegistry.length === 0}
                              <div
                                class="px-2 py-3 text-center text-sm text-neutral-400"
                              >
                                暂无模型数据，请点击上方的「同步模型列表」按钮获取
                              </div>
                            {:else}
                              <div
                                class="px-2 py-3 text-center text-sm text-neutral-400"
                              >
                                未找到匹配项
                              </div>
                            {/if}
                          {/each}
                        </Combobox.Viewport>
                        <Combobox.ScrollDownButton
                          class="flex w-full items-center justify-center py-1 text-neutral-400"
                        >
                          <CaretDoubleDown class="h-3 w-3" />
                        </Combobox.ScrollDownButton>
                      </Combobox.Content>
                    </Combobox.Portal>
                  </Combobox.Root>
                </div>
              {:else}
                <!-- 2. 已选定提供商，展示具体配置字段 -->

                <!-- 双栏排列: 配置名称 和 自定义显示别名 -->
                <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                  <div>
                    <label
                      for="provider-name-input"
                      class="mb-1.5 block text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
                    >
                      配置名称 (必填)
                    </label>
                    <input
                      id="provider-name-input"
                      type="text"
                      bind:value={editForm.name}
                      placeholder="如「DeepSeek」、「OpenAI」"
                      class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100"
                    />
                  </div>

                  <div>
                    <label
                      for="display-name-input"
                      class="mb-1.5 block text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
                    >
                      配置别名 (可选)
                    </label>
                    <input
                      id="display-name-input"
                      type="text"
                      bind:value={editForm.display_name}
                      placeholder="区分多个账号, 如「工作账号」"
                      class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100"
                    />
                  </div>
                </div>

                <!-- API Key -->
                <div>
                  <label
                    for="api-key-input"
                    class="mb-1.5 block text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
                  >
                    API 密钥 (API Key)
                  </label>
                  <PasswordInput
                    id="api-key-input"
                    bind:value={editForm.api_key}
                    placeholder={editForm.provider_type === "ollama"
                      ? "Ollama 本地服务通常免密钥"
                      : "输入您的 API Key / 密钥"}
                    class="h-10 w-full rounded-lg bg-white dark:bg-neutral-800"
                  />
                  {#if selectedRemoteProvider?.apiKeyUrl}
                    <p
                      class="mt-1.5 text-xs text-neutral-500 dark:text-neutral-400"
                    >
                      需要申请 API 密钥?
                      <button
                        type="button"
                        class="text-blue-600 hover:underline dark:text-blue-400"
                        onclick={() => {
                          if (selectedRemoteProvider?.apiKeyUrl) {
                            openUrl(selectedRemoteProvider.apiKeyUrl);
                          }
                        }}
                      >
                        点击前往官网申请
                      </button>
                    </p>
                  {/if}
                </div>

                <!-- Base URL -->
                <div>
                  <label
                    for="api-url-input"
                    class="mb-1.5 block text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
                  >
                    API 地址 (Base URL)
                  </label>
                  {#if editForm.provider_type === "openrouter"}
                    <input
                      id="api-url-input"
                      type="text"
                      value={editForm.base_url}
                      disabled
                      class="h-10 w-full cursor-not-allowed rounded-lg border border-neutral-200 bg-neutral-50 px-3 text-sm text-neutral-500 dark:border-neutral-700 dark:bg-neutral-800/50 dark:text-neutral-400"
                    />
                  {:else}
                    <input
                      id="api-url-input"
                      type="text"
                      bind:value={editForm.base_url}
                      placeholder="https://..."
                      class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100"
                    />
                  {/if}
                </div>

                <!-- Model Selector -->
                <div>
                  <div class="mb-1.5 flex items-center justify-between">
                    <label
                      for="default-model-input"
                      class="text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
                    >
                      默认模型
                    </label>
                    <button
                      type="button"
                      class="text-xs font-semibold text-blue-600 hover:underline disabled:opacity-50 dark:text-blue-400"
                      disabled={isSyncingDirect}
                      onclick={syncDirectModels}
                    >
                      {isSyncingDirect ? "正在拉取..." : "从 API 拉取最新模型"}
                    </button>
                  </div>
                  {#if modelOptions.length > 0}
                    <Combobox.Root
                      type="single"
                      name="model"
                      inputValue={modelOptions.find(
                        (o) => o.value === editForm.default_model,
                      )?.label ||
                        editForm.default_model ||
                        ""}
                      onOpenChange={(o) => {
                        if (!o) modelSearch = "";
                      }}
                      onValueChange={(v) => {
                        if (v) editForm.default_model = v;
                        modelSearch = "";
                      }}
                    >
                      <div class="relative w-full">
                        <Combobox.Input
                          id="default-model-input"
                          oninput={(e) => {
                            modelSearch = e.currentTarget.value;
                            editForm.default_model = e.currentTarget.value;
                          }}
                          onblur={(e) => {
                            if (e.currentTarget.value) {
                              editForm.default_model = e.currentTarget.value;
                            }
                          }}
                          class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm font-medium text-neutral-900 placeholder:text-neutral-500 focus:ring-2 focus:ring-neutral-950 focus:ring-offset-2 focus:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:ring-offset-neutral-950 dark:placeholder:text-neutral-400 dark:focus:ring-neutral-300"
                          placeholder="选择或输入模型"
                        />
                        <Combobox.Trigger
                          class="absolute top-1/2 right-3 -translate-y-1/2 text-neutral-400"
                        >
                          <CaretUpDown class="h-4 w-4" />
                        </Combobox.Trigger>
                      </div>

                      <Combobox.Portal>
                        <Combobox.Content
                          class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 max-h-64 w-[var(--bits-combobox-anchor-width)] overflow-hidden rounded-md border border-neutral-200 bg-white shadow-md dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-50"
                        >
                          <Combobox.ScrollUpButton
                            class="flex w-full items-center justify-center py-1 text-neutral-400"
                          >
                            <CaretDoubleUp class="h-3 w-3" />
                          </Combobox.ScrollUpButton>
                          <Combobox.Viewport class="p-1">
                            {#each filteredModelOptions as option (option.value)}
                              <Combobox.Item
                                class="flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[highlighted]:bg-neutral-100 dark:data-[highlighted]:bg-neutral-800"
                                value={option.value}
                                label={option.label}
                              >
                                {#snippet children({ selected })}
                                  <div
                                    class="flex min-w-0 flex-1 items-center gap-1.5"
                                  >
                                    <span class="truncate">{option.label}</span>
                                    <div
                                      class="flex shrink-0 items-center gap-1"
                                    >
                                      {#if option.model.context_window}
                                        <span
                                          class="rounded bg-neutral-100 px-1 py-0.5 text-[10px] font-medium text-neutral-500 dark:bg-neutral-800 dark:text-neutral-400"
                                        >
                                          {Math.round(
                                            option.model.context_window / 1024,
                                          )}k
                                        </span>
                                      {/if}
                                      {#if option.model.reasoning}
                                        <span
                                          class="rounded bg-purple-100 px-1 py-0.5 text-[10px] font-medium text-purple-600 dark:bg-purple-900/30 dark:text-purple-400"
                                          title="思考链推理">思考</span
                                        >
                                      {/if}
                                      {#if option.model.tool_call}
                                        <span
                                          class="rounded bg-blue-100 px-1 py-0.5 text-[10px] font-medium text-blue-600 dark:bg-blue-900/30 dark:text-blue-400"
                                          title="工具/函数调用">工具</span
                                        >
                                      {/if}
                                      {#if option.model.attachment}
                                        <span
                                          class="rounded bg-green-100 px-1 py-0.5 text-[10px] font-medium text-green-600 dark:bg-green-900/30 dark:text-green-400"
                                          title="图片/文件上传">附件</span
                                        >
                                      {/if}
                                    </div>
                                  </div>
                                  {#if selected}
                                    <Check class="h-4 w-4 shrink-0" />
                                  {/if}
                                {/snippet}
                              </Combobox.Item>
                            {:else}
                              <div
                                class="px-2 py-3 text-center text-sm text-neutral-400"
                              >
                                未找到匹配项
                              </div>
                            {/each}
                          </Combobox.Viewport>
                          <Combobox.ScrollDownButton
                            class="flex w-full items-center justify-center py-1 text-neutral-400"
                          >
                            <CaretDoubleDown class="h-3 w-3" />
                          </Combobox.ScrollDownButton>
                        </Combobox.Content>
                      </Combobox.Portal>
                    </Combobox.Root>
                  {:else}
                    <input
                      id="default-model-input"
                      class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100"
                      bind:value={editForm.default_model}
                      placeholder="例如 gpt-4o"
                    />
                  {/if}
                </div>
              {/if}

              <!-- Actions -->
              <div
                class="mt-6 flex items-center justify-between gap-2 border-t border-neutral-100 pt-3 dark:border-neutral-800/80"
              >
                <!-- Left: Test connection -->
                <div>
                  {#if editForm.provider_type}
                    <Button.Root
                      class="inline-flex h-9 items-center justify-center rounded-lg border border-neutral-200 bg-white px-4 text-xs font-semibold text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                      onclick={testConnection}
                    >
                      测试连接
                    </Button.Root>
                  {/if}
                </div>

                <!-- Right: Cancel and Save -->
                <div class="flex gap-2">
                  <Button.Root
                    class="inline-flex h-9 items-center justify-center rounded-lg border border-neutral-200 bg-white px-4 text-xs font-semibold text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                    onclick={cancelEdit}
                  >
                    取消
                  </Button.Root>
                  <Button.Root
                    class="inline-flex h-9 items-center justify-center rounded-lg bg-neutral-900 px-4 text-xs font-bold text-neutral-50 shadow-sm transition-colors hover:bg-neutral-900/90 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:bg-neutral-50 dark:text-neutral-900 dark:hover:bg-neutral-50/90"
                    onclick={save}
                    disabled={!editForm.provider_type}
                  >
                    保存
                  </Button.Root>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </main>
</AppScrollArea>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
  bind:open={deleteDialogOpen}
  title="删除活跃 Provider"
  description="这是当前正在使用的 Provider，删除后需要重新选择一个 Provider 才能使用 AI 功能。确定要删除吗？"
  onConfirm={handleDeleteConfirm}
  onCancel={handleDeleteCancel}
/>
