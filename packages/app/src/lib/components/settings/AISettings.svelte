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
    Trash,
    PencilSimple,
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
    { id: "providers", label: "模型", icon: Sparkle },
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

  let providers = $derived<RemoteProvider[]>(providersRegistry);

  let selectedRemoteProvider = $derived(
    providers.find((p) => p.id === editForm.provider_type),
  );

  let providerOptions = $derived(
    providers.map((p) => ({ value: p.id, label: p.name })),
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
      const remote = providers.find((p) => p.id === type);
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

  function startAdd() {
    editingIndex = -1;
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
      const remote = providers.find((p) => p.id === editForm.provider_type);
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
      toast.success("服务商配置已保存");
      editingIndex = null;
      providerSearch = "";
      modelSearch = "";
    } catch (e) {
      console.error(e);
      toast.error("保存服务商配置失败");
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
      toast.success("服务商配置已删除");
    } catch (e) {
      console.error(e);
      toast.error("删除服务商配置失败");
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
      toast.success("当前启用的服务商已更新");
    } catch (e) {
      console.error(e);
      toast.error("更新启用服务商失败");
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
      <!-- Header -->
      <div class="mb-6 flex items-center justify-between px-1">
        <div>
          <h2
            class="mb-1 text-sm font-semibold text-neutral-900 dark:text-neutral-100"
          >
            AI Providers
          </h2>
          <p class="text-xs text-neutral-500 dark:text-neutral-400">
            管理你的 AI 服务提供商
          </p>
        </div>
        {#if editingIndex === null}
          <div class="flex gap-2">
            <Button.Root
              class="inline-flex h-8 items-center justify-center gap-1.5 rounded-lg border border-neutral-200 bg-white px-3 text-xs font-medium text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
              disabled={isSyncingRegistry}
              onclick={() => syncProvidersRegistry(true)}
            >
              {isSyncingRegistry ? "正在同步..." : "同步提供商配置"}
            </Button.Root>
          </div>
        {/if}
      </div>

      <!-- Provider List or Edit Form -->
      <div class="space-y-3">
        {#if editingIndex === null}
          <!-- List View -->
          {#if config.providers.length === 0}
            <div
              class="rounded-xl border border-dashed border-neutral-300 bg-neutral-50 px-6 py-12 text-center dark:border-neutral-700 dark:bg-neutral-900/50"
            >
              <p class="mb-4 text-sm text-neutral-500 dark:text-neutral-400">
                No providers configured yet
              </p>
              <Button.Root
                class="inline-flex h-9 items-center justify-center gap-2 rounded-lg bg-neutral-900 px-4 text-sm font-medium text-neutral-50 shadow-sm transition-colors hover:bg-neutral-900/90 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:bg-neutral-50 dark:text-neutral-900 dark:hover:bg-neutral-50/90"
                onclick={startAdd}
              >
                <Plus class="h-4 w-4" />
                添加你的第一个服务商
              </Button.Root>
            </div>
          {:else}
            {#each config.providers as provider, index (provider.id)}
              <div
                class="group relative overflow-hidden rounded-xl border transition-all {config.active_provider_id ===
                provider.id
                  ? 'border-green-500 bg-green-50/50 dark:border-green-600 dark:bg-green-950/20'
                  : 'border-neutral-200 bg-white hover:border-neutral-300 dark:border-neutral-800 dark:bg-neutral-900 dark:hover:border-neutral-700'}"
              >
                <div class="flex items-start gap-4 p-4">
                  <!-- Active Indicator -->
                  <button
                    class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full border-2 transition-colors {config.active_provider_id ===
                    provider.id
                      ? 'border-green-500 bg-green-500'
                      : 'border-neutral-300 hover:border-neutral-400 dark:border-neutral-600 dark:hover:border-neutral-500'}"
                    onclick={() => setActive(provider.id)}
                    aria-label="Set as active provider"
                  >
                    {#if config.active_provider_id === provider.id}
                      <div class="h-2 w-2 rounded-full bg-white"></div>
                    {/if}
                  </button>

                  <!-- Provider Info -->
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <h3
                        class="font-semibold text-neutral-900 dark:text-neutral-100"
                      >
                        {provider.display_name || provider.name}
                      </h3>
                      {#if config.active_provider_id === provider.id}
                        <span
                          class="rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-700 dark:bg-green-900/30 dark:text-green-400"
                        >
                          当前启用
                        </span>
                      {/if}
                    </div>
                    <div
                      class="mt-1 flex items-center gap-2 text-xs text-neutral-500 dark:text-neutral-400"
                    >
                      {#if provider.display_name}
                        <span>{provider.name}</span>
                        <span>•</span>
                      {/if}
                      {#if provider.default_model}
                        <span>{provider.default_model}</span>
                        <span>•</span>
                      {/if}
                      <span class="truncate">{provider.base_url}</span>
                    </div>
                  </div>

                  <!-- Actions -->
                  <div class="flex shrink-0 gap-2">
                    <Button.Root
                      class="inline-flex h-8 items-center justify-center gap-1.5 rounded-md border border-neutral-200 bg-white px-3 text-xs font-medium text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                      onclick={() => startEdit(index)}
                    >
                      <PencilSimple class="h-3.5 w-3.5" />
                      编辑
                    </Button.Root>
                    <Button.Root
                      class="inline-flex h-8 items-center justify-center gap-1.5 rounded-md border border-red-200 bg-white px-3 text-xs font-medium text-red-600 shadow-sm transition-colors hover:bg-red-50 focus-visible:ring-2 focus-visible:ring-red-500 focus-visible:outline-hidden dark:border-red-900 dark:bg-neutral-800 dark:text-red-400 dark:hover:bg-red-950/30"
                      onclick={() => deleteProvider(index)}
                    >
                      <Trash class="h-3.5 w-3.5" />
                      删除
                    </Button.Root>
                  </div>
                </div>
              </div>
            {/each}

            <!-- Add Button -->
            <Button.Root
              class="flex h-12 w-full items-center justify-center gap-2 rounded-xl border-2 border-dashed border-neutral-300 bg-transparent text-sm font-medium text-neutral-600 transition-colors hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:text-neutral-400 dark:hover:border-neutral-600 dark:hover:bg-neutral-800/50"
              onclick={startAdd}
            >
              <Plus class="h-4 w-4" />
              添加新服务商
            </Button.Root>
          {/if}
        {:else}
          <!-- Edit Form -->
          <div
            class="overflow-hidden rounded-xl border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900"
          >
            <div
              class="border-b border-neutral-200 bg-neutral-50 px-4 py-3 dark:border-neutral-800 dark:bg-neutral-800/50"
            >
              <h3 class="font-semibold text-neutral-900 dark:text-neutral-100">
                {editingIndex === -1 ? "添加新服务商" : "编辑服务商"}
              </h3>
            </div>

            <div class="space-y-4 p-4">
              <!-- Provider Selector -->
              <div>
                <span
                  class="mb-1.5 block text-sm font-medium text-neutral-700 dark:text-neutral-300"
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
                      oninput={(e) => (providerSearch = e.currentTarget.value)}
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
                              正在加载服务商列表...
                            </div>
                          {:else if providersRegistry.length === 0}
                            <div
                              class="px-2 py-3 text-center text-sm text-neutral-400"
                            >
                              暂无服务商数据，请点击上方的「同步提供商配置」按钮获取
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

              <!-- 提供商名称 -->
              <div>
                <label
                  for="provider-name-input"
                  class="mb-1.5 block text-sm font-medium text-neutral-700 dark:text-neutral-300"
                >
                  配置名称
                </label>
                <input
                  id="provider-name-input"
                  type="text"
                  bind:value={editForm.name}
                  placeholder="如「DeepSeek」、「OpenAI」"
                  class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100"
                />
              </div>

              <!-- Base URL -->
              <div>
                <label
                  for="api-url-input"
                  class="mb-1.5 block text-sm font-medium text-neutral-700 dark:text-neutral-300"
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

              <!-- 自定义显示别名 (可选) -->
              <div>
                <label
                  for="display-name-input"
                  class="mb-1.5 block text-sm font-medium text-neutral-700 dark:text-neutral-300"
                >
                  自定义显示别名 (可选)
                </label>
                <input
                  id="display-name-input"
                  type="text"
                  bind:value={editForm.display_name}
                  placeholder="区分多个账号用, 如「我的工作账号」"
                  class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100"
                />
              </div>

              <!-- API Key -->
              <div>
                <label
                  for="api-key-input"
                  class="mb-1.5 block text-sm font-medium text-neutral-700 dark:text-neutral-300"
                >
                  API 密钥 (API Key)
                </label>
                <PasswordInput
                  id="api-key-input"
                  bind:value={editForm.api_key}
                  placeholder="sk-..."
                  class="h-10 w-full rounded-lg bg-white dark:bg-neutral-800"
                />
                {#if selectedRemoteProvider?.apiKeyUrl}
                  <p
                    class="mt-1 text-xs text-neutral-500 dark:text-neutral-400"
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

              <!-- Model Selector -->
              <div>
                <div class="mb-1.5 flex items-center justify-between">
                  <label
                    for="default-model-input"
                    class="block text-sm font-medium text-neutral-700 dark:text-neutral-300"
                  >
                    默认模型
                  </label>
                  <button
                    type="button"
                    class="text-xs font-medium text-blue-600 hover:underline disabled:opacity-50 dark:text-blue-400"
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
                                  <div class="flex shrink-0 items-center gap-1">
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

              <!-- Actions -->
              <div class="flex justify-end gap-2 pt-2">
                <Button.Root
                  class="inline-flex h-9 items-center justify-center rounded-lg border border-neutral-200 bg-white px-4 text-sm font-medium text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                  onclick={testConnection}
                >
                  测试连接
                </Button.Root>
                <Button.Root
                  class="inline-flex h-9 items-center justify-center rounded-lg border border-neutral-200 bg-white px-4 text-sm font-medium text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                  onclick={cancelEdit}
                >
                  取消
                </Button.Root>
                <Button.Root
                  class="inline-flex h-9 items-center justify-center rounded-lg bg-neutral-900 px-4 text-sm font-semibold text-neutral-50 shadow-sm transition-colors hover:bg-neutral-900/90 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:bg-neutral-50 dark:text-neutral-900 dark:hover:bg-neutral-50/90"
                  onclick={save}
                >
                  保存
                </Button.Root>
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
