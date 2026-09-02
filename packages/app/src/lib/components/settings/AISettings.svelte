<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Card } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Combobox } from "bits-ui";
  import { toast } from "svelte-sonner";
  import {
    Check,
    CaretUpDown,
    CaretDoubleUp,
    CaretDoubleDown,
    CaretDown,
    Plus,
    Sparkle,
    Cpu,
    Lightning,
    MagnifyingGlass,
    Sliders,
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

  let isOtherProvidersOpen = $state(false);
  let otherSearch = $state("");

  let availableProviders = $derived<RemoteProvider[]>(providersRegistry);

  interface FeaturedProviderDef {
    id: string;
    matchIds: string[];
    name: string;
    fallbackBaseUrl: string;
    fallbackApiKeyUrl: string;
  }

  const FEATURED_PROVIDERS: FeaturedProviderDef[] = [
    // 国际主流
    {
      id: "openai",
      matchIds: ["openai"],
      name: "OpenAI",
      fallbackBaseUrl: "https://api.openai.com/v1",
      fallbackApiKeyUrl: "https://platform.openai.com/api_keys",
    },
    {
      id: "anthropic",
      matchIds: ["anthropic"],
      name: "Anthropic",
      fallbackBaseUrl: "https://api.anthropic.com/v1",
      fallbackApiKeyUrl: "https://console.anthropic.com/settings/keys",
    },
    {
      id: "google",
      matchIds: ["google", "google-vertex"],
      name: "Google (Gemini)",
      fallbackBaseUrl:
        "https://generativelanguage.googleapis.com/v1beta/openai",
      fallbackApiKeyUrl: "https://aistudio.google.com/app/apikey",
    },
    {
      id: "xai",
      matchIds: ["xai", "x-ai"],
      name: "xAI (Grok)",
      fallbackBaseUrl: "https://api.x.ai/v1",
      fallbackApiKeyUrl: "https://console.x.ai",
    },
    // 国内主流
    {
      id: "deepseek",
      matchIds: ["deepseek"],
      name: "DeepSeek",
      fallbackBaseUrl: "https://api.deepseek.com",
      fallbackApiKeyUrl: "https://platform.deepseek.com/api_keys",
    },
    {
      id: "zhipuai",
      matchIds: ["zhipuai", "zhipu", "zhipuai-coding-plan"],
      name: "智谱 AI",
      fallbackBaseUrl: "https://open.bigmodel.cn/api/paas/v4",
      fallbackApiKeyUrl: "https://open.bigmodel.cn/usercenter/apikeys",
    },
    {
      id: "moonshotai",
      matchIds: ["moonshotai", "moonshot", "moonshotai-cn", "kimi-for-coding"],
      name: "月之暗面 (Kimi)",
      fallbackBaseUrl: "https://api.moonshot.ai/v1",
      fallbackApiKeyUrl: "https://platform.moonshot.cn/console/api-keys",
    },
    {
      id: "minimax",
      matchIds: [
        "minimax",
        "minimax-cn",
        "minimax-coding-plan",
        "minimax-cn-coding-plan",
      ],
      name: "MiniMax",
      fallbackBaseUrl: "https://api.minimax.io/v1",
      fallbackApiKeyUrl: "https://platform.minimax.io/",
    },
    {
      id: "alibaba",
      matchIds: [
        "alibaba-cn",
        "alibaba",
        "qwen",
        "alibaba-coding-plan-cn",
        "alibaba-token-plan",
      ],
      name: "通义千问 (Qwen)",
      fallbackBaseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
      fallbackApiKeyUrl: "https://dashscope.console.aliyun.com/",
    },
  ];

  let allFeaturedMatchedIds = $derived(
    new Set(
      FEATURED_PROVIDERS.flatMap((f) => [
        f.id.toLowerCase(),
        ...f.matchIds.map((m) => m.toLowerCase()),
      ]),
    ),
  );

  let otherProviders = $derived(
    availableProviders.filter(
      (p) => !allFeaturedMatchedIds.has(p.id.toLowerCase()),
    ),
  );

  let filteredOtherProviders = $derived(
    otherSearch.trim() === ""
      ? otherProviders
      : otherProviders.filter(
          (p) =>
            p.name.toLowerCase().includes(otherSearch.toLowerCase()) ||
            p.id.toLowerCase().includes(otherSearch.toLowerCase()),
        ),
  );

  let selectedRemoteProvider = $derived(
    availableProviders.find((p) => p.id === editForm.provider_type),
  );

  let providerOptions = $derived([
    { value: "custom", label: "自定义直连 (OpenAI 兼容)" },
    ...availableProviders.map((p) => ({ value: p.id, label: p.name })),
  ]);

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
      if (type === "custom") {
        if (!editForm.name) editForm.name = "自定义提供商";
        return;
      }
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

  function getMatchedRemoteProvider(
    featured: FeaturedProviderDef,
  ): RemoteProvider | undefined {
    return availableProviders.find(
      (p) =>
        featured.matchIds.some(
          (mid) => p.id.toLowerCase() === mid.toLowerCase(),
        ) || p.id.toLowerCase() === featured.id.toLowerCase(),
    );
  }

  function isFeaturedConnected(featured: FeaturedProviderDef): boolean {
    const allIds = [
      featured.id.toLowerCase(),
      ...featured.matchIds.map((m) => m.toLowerCase()),
    ];
    return config.providers.some((p) =>
      allIds.includes(p.provider_type.toLowerCase()),
    );
  }

  function connectFeaturedProvider(featured: FeaturedProviderDef) {
    const remote = getMatchedRemoteProvider(featured);
    editingIndex = -1;
    providerSearch = "";
    modelSearch = "";
    editForm = {
      id: "",
      provider_type: remote?.id || featured.id,
      name: featured.name.split(" ")[0],
      display_name: null,
      base_url: remote?.baseUrl || featured.fallbackBaseUrl,
      api_key: null,
      default_model:
        remote?.models && remote.models.length > 0 ? remote.models[0].id : null,
      models: null,
    };
  }

  function startCustomProvider() {
    editingIndex = -1;
    providerSearch = "";
    modelSearch = "";
    editForm = {
      id: "",
      provider_type: "custom",
      name: "自定义提供商",
      display_name: null,
      base_url: "",
      api_key: null,
      default_model: null,
      models: null,
    };
  }

  function startAdd() {
    startCustomProvider();
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

<ScrollArea class="h-full w-full" viewportClass="h-full w-full">
  <main class="h-full w-full pr-2 pb-8">
    <!-- Tab 导航 -->
    <div class="border-border/50 mb-6 flex gap-1.5 border-b pb-2">
      {#each tabs as tab (tab.id)}
        {@const TabIcon = tab.icon}
        {@const isActive = activeTab === tab.id}
        <button
          class="flex cursor-pointer items-center gap-2 rounded-xl px-3.5 py-1.5 text-xs font-medium transition-[background-color,color,transform,box-shadow] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-95 {isActive
            ? 'bg-card text-foreground border-border/60 border font-semibold shadow-2xs'
            : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
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
              class="text-foreground mb-1 text-sm font-semibold tracking-tight"
            >
              提供商
            </h2>
            <p class="text-muted-foreground/75 text-xs">
              管理你的 AI 提供商与服务
            </p>
          </div>
          <div class="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              class="h-8 cursor-pointer gap-1.5 rounded-xl text-xs font-medium transition-[transform,background-color] duration-140 active:scale-95"
              disabled={isSyncingRegistry}
              onclick={() => syncProvidersRegistry(true)}
            >
              <Sparkle
                size={13}
                class={isSyncingRegistry ? "animate-spin" : ""}
              />
              {isSyncingRegistry ? "正在同步..." : "同步模型列表"}
            </Button>
          </div>
        </div>
      {/if}

      <!-- Provider List or Edit Form -->
      <div class="space-y-6">
        {#if editingIndex === null}
          <!-- 已连接的提供商分区 -->
          <div>
            <h3
              class="text-muted-foreground mb-3 text-xs font-semibold tracking-wider uppercase"
            >
              已连接的提供商
            </h3>

            {#if config.providers.length === 0}
              <div
                class="border-border/60 bg-muted/20 rounded-2xl border border-dashed px-6 py-8 text-center"
              >
                <p class="text-muted-foreground text-xs">暂无已连接的提供商</p>
              </div>
            {:else}
              <div
                class="divide-border/40 border-border/60 bg-card divide-y overflow-hidden rounded-2xl border shadow-2xs"
              >
                {#each config.providers as provider, index (provider.id)}
                  {@const defaultModelInfo = getModelInfo(provider)}
                  <div
                    class="group hover:bg-muted/40 relative flex items-center transition-colors"
                  >
                    <button
                      type="button"
                      class="flex min-w-0 flex-1 cursor-pointer items-center justify-between px-4 py-3.5 text-left outline-none"
                      aria-pressed={config.active_provider_id === provider.id}
                      onclick={() => setActive(provider.id)}
                    >
                      <!-- Left indicator & Info -->
                      <div class="flex min-w-0 items-center gap-3">
                        <!-- Active dot indicator -->
                        <div class="flex w-4 items-center justify-center">
                          {#if config.active_provider_id === provider.id}
                            <span class="relative flex h-2 w-2">
                              <span
                                class="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75"
                              ></span>
                              <span
                                class="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"
                              ></span>
                            </span>
                          {:else}
                            <span
                              class="bg-muted-foreground/30 group-hover:bg-muted-foreground/50 h-2 w-2 rounded-full transition-colors"
                            ></span>
                          {/if}
                        </div>

                        <!-- Text details -->
                        <div class="min-w-0">
                          <div class="flex items-center gap-2">
                            <span
                              class="text-foreground truncate text-sm font-semibold tracking-tight"
                            >
                              {provider.display_name || provider.name}
                            </span>
                            <span
                              class="border-border/50 bg-muted text-muted-foreground rounded-md border px-1.5 py-0.5 text-[10px] font-medium"
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
                            class="border-border/50 bg-muted/60 text-muted-foreground rounded-md border px-2 py-0.5 font-mono text-[11px]"
                          >
                            {provider.default_model}
                          </code>
                        {/if}
                      </div>
                    </button>

                    <!-- Right actions -->
                    <div class="flex items-center gap-3 pr-4">
                      <button
                        type="button"
                        class="text-muted-foreground hover:text-foreground cursor-pointer text-xs font-medium transition-[color,transform] duration-120 active:scale-95"
                        onclick={() => startEdit(index)}
                      >
                        编辑
                      </button>
                      <span class="text-border">|</span>
                      <button
                        type="button"
                        class="text-destructive/80 hover:text-destructive cursor-pointer text-xs font-medium transition-[color,transform] duration-120 active:scale-95"
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
              class="text-muted-foreground text-xs font-semibold tracking-wider uppercase"
            >
              热门提供商
            </h3>
            <div class="grid grid-cols-1 gap-2.5 sm:grid-cols-2 md:grid-cols-3">
              <!-- 9 大精选热门提供商 -->
              {#each FEATURED_PROVIDERS as fp (fp.id)}
                {@const isConnected = isFeaturedConnected(fp)}
                <div
                  class="border-border/60 bg-card hover:border-border flex items-center justify-between rounded-xl border p-2.5 shadow-2xs transition-[border-color,box-shadow,transform] duration-140 hover:-translate-y-0.5"
                >
                  <div class="flex min-w-0 items-center gap-2 pr-1">
                    <span class="text-foreground truncate text-xs font-medium">
                      {fp.name}
                    </span>
                    {#if isConnected}
                      <span
                        class="inline-flex shrink-0 items-center gap-1 text-[10px] font-medium text-emerald-500"
                        title="已连接此提供商"
                      >
                        <span class="h-1.5 w-1.5 rounded-full bg-emerald-500"
                        ></span>
                        已连接
                      </span>
                    {/if}
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    class="h-7 shrink-0 cursor-pointer gap-0.5 rounded-lg px-2.5 text-[11px] font-medium transition-[transform,background-color] duration-120 active:scale-90"
                    onclick={() => connectFeaturedProvider(fp)}
                  >
                    <Plus class="h-3 w-3" />
                    连接
                  </Button>
                </div>
              {/each}

              <!-- 自定义直连卡片 (横跨 3 列) -->
              <div
                class="border-border/60 bg-card hover:border-border col-span-1 flex items-center justify-between rounded-xl border border-dashed p-2.5 shadow-2xs transition-[border-color,box-shadow,transform] duration-140 hover:-translate-y-0.5 sm:col-span-2 md:col-span-3"
              >
                <div class="flex min-w-0 items-center gap-2 pr-1">
                  <Sliders class="text-primary h-3.5 w-3.5 shrink-0" />
                  <span class="text-foreground text-xs font-medium">
                    自定义提供商
                  </span>
                  <span
                    class="text-muted-foreground hidden text-[11px] sm:inline"
                  >
                    (OpenAI 兼容协议 / 私有部署与第三方中转)
                  </span>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  class="h-7 shrink-0 cursor-pointer gap-0.5 rounded-lg px-2.5 text-[11px] font-medium transition-[transform,background-color] duration-120 active:scale-90"
                  onclick={startCustomProvider}
                >
                  <Plus class="h-3 w-3" />
                  连接
                </Button>
              </div>
            </div>
          </div>

          <!-- 其他可用提供商分区 (默认折叠) -->
          <div
            class="border-border/50 bg-muted/10 space-y-3 rounded-2xl border p-3.5"
          >
            <button
              type="button"
              class="group flex w-full cursor-pointer items-center justify-between text-left outline-none"
              onclick={() => (isOtherProvidersOpen = !isOtherProvidersOpen)}
            >
              <div class="flex items-center gap-2">
                <span
                  class="text-muted-foreground text-xs font-semibold tracking-wider uppercase"
                >
                  其他可用提供商
                </span>
                <span
                  class="bg-muted text-muted-foreground rounded-full px-2 py-0.5 text-[10px] font-medium"
                >
                  {otherProviders.length}
                </span>
              </div>
              <div
                class="text-muted-foreground group-hover:text-foreground flex items-center gap-1 text-xs transition-colors"
              >
                <span>{isOtherProvidersOpen ? "收起列表" : "展开全部"}</span>
                <CaretDown
                  class="h-3.5 w-3.5 transition-transform duration-200 {isOtherProvidersOpen
                    ? 'rotate-180'
                    : ''}"
                />
              </div>
            </button>

            {#if isOtherProvidersOpen}
              <div class="space-y-3 pt-1">
                <!-- 搜索栏 -->
                <div class="relative w-full">
                  <MagnifyingGlass
                    class="text-muted-foreground absolute top-1/2 left-3 h-3.5 w-3.5 -translate-y-1/2"
                  />
                  <input
                    type="text"
                    bind:value={otherSearch}
                    placeholder="搜索其他可用提供商..."
                    class="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring h-8 w-full rounded-lg border pr-3 pl-8 text-xs transition-[border-color,box-shadow] duration-120 ease-out focus:ring-1 focus:outline-none"
                  />
                </div>

                <!-- 列表网格 (使用 ScrollArea 自定义滚动条) -->
                {#if filteredOtherProviders.length === 0}
                  <div class="text-muted-foreground py-6 text-center text-xs">
                    未找到匹配的提供商
                  </div>
                {:else}
                  <ScrollArea
                    class="h-72 w-full"
                    viewportClass="h-full w-full pr-3"
                  >
                    <div
                      class="grid grid-cols-2 gap-2 pb-1 sm:grid-cols-3 md:grid-cols-4"
                    >
                      {#each filteredOtherProviders as rp (rp.id)}
                        <div
                          class="border-border/60 bg-card hover:border-border flex items-center justify-between rounded-xl border p-2 shadow-2xs transition-[border-color,box-shadow,transform] duration-140 hover:-translate-y-0.5"
                        >
                          <span
                            class="text-foreground truncate pr-1 text-xs font-medium"
                            title={rp.name}
                          >
                            {rp.name}
                          </span>
                          <Button
                            variant="outline"
                            size="sm"
                            class="h-6 shrink-0 cursor-pointer gap-0.5 rounded-md px-2 text-[10px] font-medium transition-[transform,background-color] duration-120 active:scale-90"
                            onclick={() => connectRegistryProvider(rp)}
                          >
                            <Plus class="h-2.5 w-2.5" />
                            连接
                          </Button>
                        </div>
                      {/each}
                    </div>
                  </ScrollArea>
                {/if}
              </div>
            {/if}
          </div>
        {:else}
          <!-- Edit Form -->
          <div
            class="border-border/60 bg-card overflow-hidden rounded-2xl border shadow-2xs transition-[border-color,box-shadow] duration-140 ease-out"
          >
            <!-- Header with name -->
            <div
              class="border-border/50 bg-muted/20 relative flex items-center justify-between border-b px-5 py-4"
            >
              <div class="flex items-center gap-2.5">
                <div>
                  <h3
                    class="text-foreground text-sm font-semibold tracking-tight"
                  >
                    {editingIndex === -1 ? "连接新提供商" : "编辑提供商配置"}
                  </h3>
                  <p
                    class="text-muted-foreground/75 mt-0.5 text-xs leading-normal"
                  >
                    {editForm.provider_type
                      ? editForm.provider_type === "custom"
                        ? "正在配置自定义直连服务 (OpenAI 兼容协议)"
                        : `正在配置 ${selectedRemoteProvider?.name || editForm.provider_type}`
                      : "选择一个大模型提供商开始连接"}
                  </p>
                </div>
              </div>

              <!-- 如果是新建且已经选了类型，允许用户清除重选 -->
              {#if editingIndex === -1 && editForm.provider_type}
                <button
                  type="button"
                  class="text-muted-foreground hover:text-foreground cursor-pointer text-xs font-medium transition-colors"
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

            <div class="space-y-4 p-5">
              <!-- 1. 如果尚未选择提供商类型 (仅在新建且 provider_type 为空时显示) -->
              {#if !editForm.provider_type}
                <div>
                  <span
                    class="text-muted-foreground mb-1.5 block text-xs font-semibold tracking-wider uppercase"
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
                        class="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring h-10 w-full rounded-xl border px-3 text-sm font-medium transition-[border-color,box-shadow] duration-120 ease-out focus:ring-1 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
                        placeholder="选择提供商"
                      />
                      <Combobox.Trigger
                        class="text-muted-foreground absolute top-1/2 right-3 -translate-y-1/2"
                      >
                        <CaretUpDown class="h-4 w-4" />
                      </Combobox.Trigger>
                    </div>

                    <Combobox.Portal>
                      <Combobox.Content
                        class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:data-[state=open]:slide-in-from-top-2 data-[side=bottom]:data-[state=closed]:slide-out-to-top-2 data-[side=top]:data-[state=open]:slide-in-from-bottom-2 data-[side=top]:data-[state=closed]:slide-out-to-bottom-2 bg-popover text-popover-foreground border-border/60 z-50 max-h-64 w-[var(--bits-combobox-anchor-width)] origin-[var(--bits-floating-transform-origin,center)] overflow-hidden rounded-xl border shadow-xl duration-140 ease-[cubic-bezier(0.23,1,0.32,1)]"
                      >
                        <Combobox.ScrollUpButton
                          class="text-muted-foreground flex w-full items-center justify-center py-1"
                        >
                          <CaretDoubleUp class="h-3 w-3" />
                        </Combobox.ScrollUpButton>
                        <Combobox.Viewport class="p-1">
                          {#each filteredProviderOptions as option (option.value)}
                            <Combobox.Item
                              class="text-popover-foreground data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground flex cursor-pointer items-center rounded-lg px-2.5 py-1.5 text-sm outline-none select-none"
                              value={option.value}
                              label={option.label}
                            >
                              {#snippet children({ selected })}
                                <span class="flex-1">{option.label}</span>
                                {#if selected}
                                  <Check class="text-primary h-4 w-4" />
                                {/if}
                              {/snippet}
                            </Combobox.Item>
                          {:else}
                            {#if isRegistryLoading}
                              <div
                                class="text-muted-foreground px-2 py-3 text-center text-sm"
                              >
                                正在加载模型列表...
                              </div>
                            {:else if providersRegistry.length === 0}
                              <div
                                class="text-muted-foreground px-2 py-3 text-center text-sm"
                              >
                                暂无模型数据，请点击上方的「同步模型列表」按钮获取
                              </div>
                            {:else}
                              <div
                                class="text-muted-foreground px-2 py-3 text-center text-sm"
                              >
                                未找到匹配项
                              </div>
                            {/if}
                          {/each}
                        </Combobox.Viewport>
                        <Combobox.ScrollDownButton
                          class="text-muted-foreground flex w-full items-center justify-center py-1"
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
                      class="text-muted-foreground mb-1.5 block text-xs font-semibold tracking-wider uppercase"
                    >
                      配置名称 (必填)
                    </label>
                    <input
                      id="provider-name-input"
                      type="text"
                      bind:value={editForm.name}
                      placeholder="如「DeepSeek」、「OpenAI」"
                      class="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring h-10 w-full rounded-xl border px-3 text-sm transition-[border-color,box-shadow] duration-120 ease-out focus:ring-1 focus:outline-none"
                    />
                  </div>

                  <div>
                    <label
                      for="display-name-input"
                      class="text-muted-foreground mb-1.5 block text-xs font-semibold tracking-wider uppercase"
                    >
                      配置别名 (可选)
                    </label>
                    <input
                      id="display-name-input"
                      type="text"
                      bind:value={editForm.display_name}
                      placeholder="区分多个账号, 如「工作账号」"
                      class="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring h-10 w-full rounded-xl border px-3 text-sm transition-[border-color,box-shadow] duration-120 ease-out focus:ring-1 focus:outline-none"
                    />
                  </div>
                </div>

                <!-- API Key -->
                <div>
                  <label
                    for="api-key-input"
                    class="text-muted-foreground mb-1.5 block text-xs font-semibold tracking-wider uppercase"
                  >
                    API 密钥 (API Key)
                  </label>
                  <PasswordInput
                    id="api-key-input"
                    bind:value={editForm.api_key}
                    placeholder={editForm.provider_type === "ollama"
                      ? "Ollama 本地服务通常免密钥"
                      : "输入您的 API Key / 密钥"}
                    class="bg-background h-10 w-full rounded-xl"
                  />
                  {#if selectedRemoteProvider?.apiKeyUrl}
                    <p class="text-muted-foreground mt-1.5 text-xs">
                      需要申请 API 密钥?
                      <button
                        type="button"
                        class="text-primary cursor-pointer font-medium hover:underline"
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
                    class="text-muted-foreground mb-1.5 block text-xs font-semibold tracking-wider uppercase"
                  >
                    API 地址 (Base URL)
                  </label>
                  {#if editForm.provider_type === "openrouter"}
                    <input
                      id="api-url-input"
                      type="text"
                      value={editForm.base_url}
                      disabled
                      class="border-input bg-muted/50 text-muted-foreground h-10 w-full cursor-not-allowed rounded-xl border px-3 text-sm"
                    />
                  {:else}
                    <input
                      id="api-url-input"
                      type="text"
                      bind:value={editForm.base_url}
                      placeholder="https://..."
                      class="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring h-10 w-full rounded-xl border px-3 text-sm transition-[border-color,box-shadow] duration-120 ease-out focus:ring-1 focus:outline-none"
                    />
                  {/if}
                </div>

                <!-- Model Selector -->
                <div>
                  <div class="mb-1.5 flex items-center justify-between">
                    <label
                      for="default-model-input"
                      class="text-muted-foreground text-xs font-semibold tracking-wider uppercase"
                    >
                      默认模型
                    </label>
                    <button
                      type="button"
                      class="text-primary cursor-pointer text-xs font-medium hover:underline disabled:opacity-50"
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
                          class="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring h-10 w-full rounded-xl border px-3 text-sm font-medium transition-[border-color,box-shadow] duration-120 ease-out focus:ring-1 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
                          placeholder="选择或输入模型"
                        />
                        <Combobox.Trigger
                          class="text-muted-foreground absolute top-1/2 right-3 -translate-y-1/2"
                        >
                          <CaretUpDown class="h-4 w-4" />
                        </Combobox.Trigger>
                      </div>

                      <Combobox.Portal>
                        <Combobox.Content
                          class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:data-[state=open]:slide-in-from-top-2 data-[side=bottom]:data-[state=closed]:slide-out-to-top-2 data-[side=top]:data-[state=open]:slide-in-from-bottom-2 data-[side=top]:data-[state=closed]:slide-out-to-bottom-2 bg-popover text-popover-foreground border-border/60 z-50 max-h-64 w-[var(--bits-combobox-anchor-width)] origin-[var(--bits-floating-transform-origin,center)] overflow-hidden rounded-xl border shadow-xl duration-140 ease-[cubic-bezier(0.23,1,0.32,1)]"
                        >
                          <Combobox.ScrollUpButton
                            class="text-muted-foreground flex w-full items-center justify-center py-1"
                          >
                            <CaretDoubleUp class="h-3 w-3" />
                          </Combobox.ScrollUpButton>
                          <Combobox.Viewport class="p-1">
                            {#each filteredModelOptions as option (option.value)}
                              <Combobox.Item
                                class="text-popover-foreground data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground flex cursor-pointer items-center rounded-lg px-2.5 py-1.5 text-sm outline-none select-none"
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
                                          class="bg-muted text-muted-foreground rounded px-1 py-0.5 text-[10px] font-medium"
                                        >
                                          {Math.round(
                                            option.model.context_window / 1024,
                                          )}k
                                        </span>
                                      {/if}
                                      {#if option.model.reasoning}
                                        <span
                                          class="rounded bg-purple-500/10 px-1 py-0.5 text-[10px] font-medium text-purple-600 dark:text-purple-400"
                                          title="思考链推理">思考</span
                                        >
                                      {/if}
                                      {#if option.model.tool_call}
                                        <span
                                          class="rounded bg-blue-500/10 px-1 py-0.5 text-[10px] font-medium text-blue-600 dark:text-blue-400"
                                          title="工具/函数调用">工具</span
                                        >
                                      {/if}
                                      {#if option.model.attachment}
                                        <span
                                          class="rounded bg-emerald-500/10 px-1 py-0.5 text-[10px] font-medium text-emerald-600 dark:text-emerald-400"
                                          title="图片/文件上传">附件</span
                                        >
                                      {/if}
                                    </div>
                                  </div>
                                  {#if selected}
                                    <Check
                                      class="text-primary h-4 w-4 shrink-0"
                                    />
                                  {/if}
                                {/snippet}
                              </Combobox.Item>
                            {:else}
                              <div
                                class="text-muted-foreground px-2 py-3 text-center text-sm"
                              >
                                未找到匹配项
                              </div>
                            {/each}
                          </Combobox.Viewport>
                          <Combobox.ScrollDownButton
                            class="text-muted-foreground flex w-full items-center justify-center py-1"
                          >
                            <CaretDoubleDown class="h-3 w-3" />
                          </Combobox.ScrollDownButton>
                        </Combobox.Content>
                      </Combobox.Portal>
                    </Combobox.Root>
                  {:else}
                    <input
                      id="default-model-input"
                      class="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring h-10 w-full rounded-xl border px-3 text-sm transition-[border-color,box-shadow] duration-120 ease-out focus:ring-1 focus:outline-none"
                      bind:value={editForm.default_model}
                      placeholder="例如 gpt-4o"
                    />
                  {/if}
                </div>
              {/if}

              <!-- Actions -->
              <div
                class="border-border/50 mt-6 flex items-center justify-between gap-2 border-t pt-3"
              >
                <!-- Left: Test connection -->
                <div>
                  {#if editForm.provider_type}
                    <Button
                      variant="outline"
                      size="sm"
                      class="h-9 cursor-pointer rounded-xl px-4 text-xs font-semibold transition-[transform,background-color] duration-120 active:scale-95"
                      onclick={testConnection}
                    >
                      测试连接
                    </Button>
                  {/if}
                </div>

                <!-- Right: Cancel and Save -->
                <div class="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    class="h-9 cursor-pointer rounded-xl px-4 text-xs font-semibold transition-[transform,background-color] duration-120 active:scale-95"
                    onclick={cancelEdit}
                  >
                    取消
                  </Button>
                  <Button
                    variant="default"
                    size="sm"
                    class="h-9 cursor-pointer rounded-xl px-4 text-xs font-semibold transition-[transform,background-color] duration-120 active:scale-95"
                    onclick={save}
                    disabled={!editForm.provider_type}
                  >
                    保存
                  </Button>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </main>
</ScrollArea>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
  bind:open={deleteDialogOpen}
  title="删除活跃 Provider"
  description="这是当前正在使用的 Provider，删除后需要重新选择一个 Provider 才能使用 AI 功能。确定要删除吗？"
  onConfirm={handleDeleteConfirm}
  onCancel={handleDeleteCancel}
/>
