<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import { Select } from "bits-ui";
  import { Check, CaretUpDown } from "phosphor-svelte";
  import type { AppConfig } from "$lib/type";

  // 定义 AI 相关接口
  interface ModelModalities {
    input: string[];
    output: string[];
  }

  interface ModelInfo {
    id: string;
    name: string;
    description?: string | null;
    modalities?: ModelModalities | null;
  }

  interface ProviderConfig {
    id: string;
    provider_type: string;
    name: string;
    display_name?: string | null;
    default_model?: string | null;
    models?: ModelInfo[] | null;
  }

  interface AIConfig {
    active_provider_id: string | null;
    providers: ProviderConfig[];
  }

  let config = $state<AppConfig | null>(null);
  let aiConfig = $state<AIConfig>({ active_provider_id: null, providers: [] });
  let loading = $state(true);
  let saving = $state(false);

  // 默认识别引擎可选项
  const engineOptions = [
    { value: "local", label: "本地原生 OCR (完全离线、低延迟、高隐私安全)" },
    {
      value: "ai",
      label: "AI 智能 OCR (支持复杂版面、多语言混合、需联网消耗Token)",
    },
  ];

  // 派生状态：当前选中的 Provider 的模型列表
  let activeProviderModels = $derived.by(() => {
    const providerId = config?.ocr_provider_id;
    if (!providerId) return [];
    const provider = aiConfig.providers.find((p) => p.id === providerId);
    return provider?.models || [];
  });

  // 判断提供商是否支持多模态识别（无本地模型缓存时保留以防误杀）
  function providerSupportsImage(provider: ProviderConfig): boolean {
    if (!provider.models || provider.models.length === 0) {
      return true;
    }
    return provider.models.some(supportsImage);
  }

  // 派生状态：全局激活的提供商和模型信息
  let globalProviderName = $derived.by(() => {
    if (!aiConfig.active_provider_id) return "未激活";
    const provider = aiConfig.providers.find(
      (p) => p.id === aiConfig.active_provider_id,
    );
    return provider ? provider.display_name || provider.name : "未激活";
  });

  let globalModelName = $derived.by(() => {
    if (!aiConfig.active_provider_id) return "未启用";
    const provider = aiConfig.providers.find(
      (p) => p.id === aiConfig.active_provider_id,
    );
    if (!provider) return "未启用";
    const defaultModelId = provider.default_model;
    if (!defaultModelId) return "未配置模型";
    const modelInfo = provider.models?.find((m) => m.id === defaultModelId);
    return modelInfo?.name || defaultModelId;
  });

  interface GlobalModelStatus {
    configured: boolean;
    modelName: string;
    supportsImage: boolean;
  }

  let globalModelStatus = $derived.by<GlobalModelStatus>(() => {
    if (!aiConfig.active_provider_id) {
      return { configured: false, modelName: "", supportsImage: false };
    }
    const provider = aiConfig.providers.find(
      (p) => p.id === aiConfig.active_provider_id,
    );
    if (!provider) {
      return { configured: false, modelName: "", supportsImage: false };
    }
    const defaultModelId = provider.default_model;
    if (!defaultModelId) {
      return { configured: false, modelName: "", supportsImage: false };
    }
    const modelInfo = provider.models?.find((m) => m.id === defaultModelId);
    const isVl = modelInfo
      ? supportsImage(modelInfo)
      : supportsImage({ id: defaultModelId, name: defaultModelId });
    return {
      configured: true,
      modelName: modelInfo?.name || defaultModelId,
      supportsImage: isVl,
    };
  });

  // 派生状态：AI 提供商可选项
  let providerOptions = $derived([
    { value: "default", label: `跟随全局配置 (${globalProviderName})` },
    ...aiConfig.providers.filter(providerSupportsImage).map((p) => ({
      value: p.id,
      label: p.display_name || p.name,
    })),
  ]);

  // 派生状态：当前选中的提供商的默认模型名称
  let activeProviderDefaultModelName = $derived.by(() => {
    const providerId = config?.ocr_provider_id;
    if (!providerId) return "未指定";
    const provider = aiConfig.providers.find((p) => p.id === providerId);
    if (!provider) return "未指定";
    const defaultModelId = provider.default_model;
    if (!defaultModelId) return "未配置模型";
    const modelInfo = provider.models?.find((m) => m.id === defaultModelId);
    return modelInfo?.name || defaultModelId;
  });

  // 派生状态：AI 模型可选项
  let modelOptions = $derived.by(() => {
    const options = [];
    if (!config?.ocr_provider_id) {
      options.push({
        value: "default",
        label: `跟随全局激活的默认模型 (${globalModelName})`,
      });
    } else {
      options.push({
        value: "default",
        label: `默认模型 (${activeProviderDefaultModelName})`,
      });

      // 1. 优先过滤出确定支持多模态（图片输入）的模型
      const vlModels = activeProviderModels.filter(supportsImage);
      if (vlModels.length > 0) {
        vlModels.forEach((model) => {
          options.push({
            value: model.id,
            label: model.name,
          });
        });
      } else {
        // 2. 后备机制：若未筛选出任何多模态模型，则罗列全部模型，并提示未检测到多模态支持
        activeProviderModels.forEach((model) => {
          options.push({
            value: model.id,
            label: `${model.name} (未检测到多模态支持)`,
          });
        });
      }
    }
    return options;
  });

  // 获取全局配置和 AI 配置
  async function loadData() {
    loading = true;
    try {
      config = await invoke<AppConfig>("get_app_config");
      aiConfig = await invoke<AIConfig>("get_ai_config");
    } catch (error) {
      console.error("Failed to load OCR settings data:", error);
      toast.error("加载设置失败");
    } finally {
      loading = false;
    }
  }

  // 保存设置到全局配置
  async function saveConfig() {
    if (!config || saving) return;
    saving = true;
    try {
      await invoke("update_app_config", { config });
    } catch (error) {
      console.error("Failed to save OCR config:", error);
      toast.error("保存 OCR 设置失败");
    } finally {
      saving = false;
    }
  }

  // 默认引擎更改处理
  function handleEngineChange(val: string | undefined) {
    if (!config || !val) return;
    config.ocr_default_engine = val;
    void saveConfig();
  }

  // 提供商更改处理
  function handleProviderChange(val: string | undefined) {
    if (!config || !val) return;
    config.ocr_provider_id = val === "default" ? null : val;
    // 切换提供商时，自动重置选中的模型为跟随全局默认
    config.ocr_model_id = null;
    void saveConfig();
  }

  // 模型更改处理
  function handleModelChange(val: string | undefined) {
    if (!config || !val) return;
    config.ocr_model_id = val === "default" ? null : val;
    void saveConfig();
  }

  // 判断模型是否支持图片识别（多模态）
  function supportsImage(model: ModelInfo): boolean {
    if (model.modalities?.input?.includes("image")) {
      return true;
    }
    const id = model.id.toLowerCase();
    return (
      id.includes("gpt-4o") ||
      id.includes("gemini") ||
      id.includes("vision") ||
      id.includes("vl") ||
      id.includes("claude-3") ||
      id.includes("qwen-vl") ||
      id.includes("pixtral") ||
      id.includes("llava")
    );
  }

  onMount(loadData);
</script>

{#if loading}
  <div class="py-3 text-sm text-neutral-500 dark:text-neutral-400">
    正在加载文字识别设置...
  </div>
{:else if config}
  <div class="flex flex-col gap-5">
    <!-- 默认引擎设置 -->
    <section class="border-t border-neutral-100 pt-4 dark:border-neutral-800">
      <div class="flex flex-col gap-1.5">
        <h4 class="text-sm font-semibold text-neutral-950 dark:text-neutral-50">
          默认识别引擎
        </h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400">
          设置每次打开文字识别页面时，首选激活的 OCR 识别模式。
        </p>

        <Select.Root
          type="single"
          value={config.ocr_default_engine || "local"}
          onValueChange={handleEngineChange}
        >
          <Select.Trigger
            class="mt-2 inline-flex h-10 w-full items-center justify-between rounded-lg border border-neutral-200 bg-white px-3 py-2 text-sm text-neutral-900 transition-colors focus:border-neutral-400 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-100 dark:focus:border-neutral-600"
          >
            <span class="truncate">
              {engineOptions.find(
                (o) => o.value === (config?.ocr_default_engine || "local"),
              )?.label}
            </span>
            <CaretUpDown class="ml-auto size-4 shrink-0 text-neutral-400" />
          </Select.Trigger>
          <Select.Portal>
            <Select.Content
              class="z-50 max-h-60 w-[var(--bits-select-anchor-width)] min-w-[var(--bits-select-anchor-width)] rounded-lg border border-neutral-200 bg-white px-1 py-1.5 shadow-lg outline-hidden select-none data-[side=bottom]:translate-y-1 data-[side=top]:-translate-y-1 dark:border-neutral-800 dark:bg-neutral-950"
              sideOffset={4}
            >
              <Select.Viewport class="p-0.5">
                {#each engineOptions as option (option.value)}
                  <Select.Item
                    class="flex h-9 w-full cursor-pointer items-center rounded-md py-2 pr-2 pl-3 text-sm text-neutral-800 outline-hidden hover:bg-neutral-50 data-disabled:opacity-50 data-highlighted:bg-neutral-50 dark:text-neutral-200 dark:hover:bg-neutral-900 dark:data-highlighted:bg-neutral-900"
                    value={option.value}
                    label={option.label}
                  >
                    {#snippet children({ selected })}
                      <span class="truncate">{option.label}</span>
                      {#if selected}
                        <Check
                          class="ml-auto size-4 shrink-0 text-neutral-600 dark:text-neutral-400"
                        />
                      {/if}
                    {/snippet}
                  </Select.Item>
                {/each}
              </Select.Viewport>
            </Select.Content>
          </Select.Portal>
        </Select.Root>
      </div>
    </section>

    <!-- AI 提供商设置 -->
    <section class="border-t border-neutral-100 pt-4 dark:border-neutral-800">
      <div class="flex flex-col gap-1.5">
        <h4 class="text-sm font-semibold text-neutral-950 dark:text-neutral-50">
          AI 识别服务提供商
        </h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400">
          指定用于 AI 识别的服务提供商。选择跟随全局时，自动同步主 AI
          设置页面中的模型。
        </p>

        <Select.Root
          type="single"
          value={config.ocr_provider_id || "default"}
          onValueChange={handleProviderChange}
        >
          <Select.Trigger
            class="mt-2 inline-flex h-10 w-full items-center justify-between rounded-lg border border-neutral-200 bg-white px-3 py-2 text-sm text-neutral-900 transition-colors focus:border-neutral-400 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-100 dark:focus:border-neutral-600"
          >
            <span class="truncate">
              {providerOptions.find(
                (o) => o.value === (config?.ocr_provider_id || "default"),
              )?.label}
            </span>
            <CaretUpDown class="ml-auto size-4 shrink-0 text-neutral-400" />
          </Select.Trigger>
          <Select.Portal>
            <Select.Content
              class="z-50 max-h-60 w-[var(--bits-select-anchor-width)] min-w-[var(--bits-select-anchor-width)] rounded-lg border border-neutral-200 bg-white px-1 py-1.5 shadow-lg outline-hidden select-none data-[side=bottom]:translate-y-1 data-[side=top]:-translate-y-1 dark:border-neutral-800 dark:bg-neutral-950"
              sideOffset={4}
            >
              <Select.Viewport class="p-0.5">
                {#each providerOptions as option (option.value)}
                  <Select.Item
                    class="flex h-9 w-full cursor-pointer items-center rounded-md py-2 pr-2 pl-3 text-sm text-neutral-800 outline-hidden hover:bg-neutral-50 data-disabled:opacity-50 data-highlighted:bg-neutral-50 dark:text-neutral-200 dark:hover:bg-neutral-900 dark:data-highlighted:bg-neutral-900"
                    value={option.value}
                    label={option.label}
                  >
                    {#snippet children({ selected })}
                      <span class="truncate">{option.label}</span>
                      {#if selected}
                        <Check
                          class="ml-auto size-4 shrink-0 text-neutral-600 dark:text-neutral-400"
                        />
                      {/if}
                    {/snippet}
                  </Select.Item>
                {/each}
              </Select.Viewport>
            </Select.Content>
          </Select.Portal>
        </Select.Root>

        {#if !config.ocr_provider_id}
          <div
            class="mt-2.5 flex items-start gap-2 rounded-lg border px-3 py-2 text-xs
            {globalModelStatus.configured
              ? globalModelStatus.supportsImage
                ? 'border-emerald-200 bg-emerald-50/30 text-emerald-800 dark:border-emerald-800/30 dark:bg-emerald-950/10 dark:text-emerald-300'
                : 'border-amber-200 bg-amber-50/30 text-amber-800 dark:border-amber-800/30 dark:bg-amber-950/10 dark:text-amber-300'
              : 'border-amber-200 bg-amber-50/30 text-amber-800 dark:border-amber-800/30 dark:bg-amber-950/10 dark:text-amber-300'}"
          >
            {#if globalModelStatus.configured && globalModelStatus.supportsImage}
              <span
                class="mt-0.5 shrink-0 text-emerald-600 dark:text-emerald-400"
                >✓</span
              >
            {:else}
              <span class="mt-0.5 shrink-0 text-amber-600 dark:text-amber-400"
                >⚠️</span
              >
            {/if}
            <div>
              {#if !globalModelStatus.configured}
                <span class="font-semibold">全局未激活 AI 模型：</span
                >当前全局未配置或激活默认 AI 渠道，AI OCR
                识别可能无法工作。建议前往“设置 -
                模型”中激活，或者在此处手动指定支持多模态的独立模型。
              {:else if !globalModelStatus.supportsImage}
                <span class="font-semibold">全局模型可能不支持多模态：</span
                >当前全局默认模型
                <code
                  class="rounded-sm bg-amber-100/50 px-1 py-0.5 font-mono text-[10px] dark:bg-neutral-800"
                  >{globalModelStatus.modelName}</code
                > 未检测到图片识别能力。AI OCR 识别可能无法正常工作，建议在此手动绑定独立的多模态通道。
              {:else}
                <span class="font-semibold">全局模型兼容：</span
                >当前全局默认模型
                <code
                  class="rounded-sm bg-emerald-100/50 px-1 py-0.5 font-mono text-[10px] dark:bg-neutral-800"
                  >{globalModelStatus.modelName}</code
                > 支持图片多模态输入，已完美直接跟随使用。
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </section>

    <!-- AI 模型设置 -->
    <section class="border-t border-neutral-100 pt-4 dark:border-neutral-800">
      <div class="flex flex-col gap-1.5">
        <h4 class="text-sm font-semibold text-neutral-950 dark:text-neutral-50">
          AI 识别模型
        </h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400">
          选择具体执行图片识别的 AI 模型。推荐带有相机图标或标有 “多模态”
          的模型。
        </p>

        <Select.Root
          type="single"
          value={config.ocr_model_id || "default"}
          onValueChange={handleModelChange}
          disabled={!config.ocr_provider_id}
        >
          <Select.Trigger
            class="mt-2 inline-flex h-10 w-full items-center justify-between rounded-lg border border-neutral-200 bg-white px-3 py-2 text-sm text-neutral-900 transition-colors focus:border-neutral-400 disabled:cursor-not-allowed disabled:bg-neutral-50 disabled:text-neutral-400 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-100 dark:focus:border-neutral-600 dark:disabled:bg-neutral-900 dark:disabled:text-neutral-500"
          >
            <span class="truncate">
              {modelOptions.find(
                (o) => o.value === (config?.ocr_model_id || "default"),
              )?.label}
            </span>
            <CaretUpDown class="ml-auto size-4 shrink-0 text-neutral-400" />
          </Select.Trigger>
          <Select.Portal>
            <Select.Content
              class="z-50 max-h-60 w-[var(--bits-select-anchor-width)] min-w-[var(--bits-select-anchor-width)] rounded-lg border border-neutral-200 bg-white px-1 py-1.5 shadow-lg outline-hidden select-none data-[side=bottom]:translate-y-1 data-[side=top]:-translate-y-1 dark:border-neutral-800 dark:bg-neutral-950"
              sideOffset={4}
            >
              <Select.Viewport class="p-0.5">
                {#each modelOptions as option (option.value)}
                  <Select.Item
                    class="flex h-9 w-full cursor-pointer items-center rounded-md py-2 pr-2 pl-3 text-sm text-neutral-800 outline-hidden hover:bg-neutral-50 data-disabled:opacity-50 data-highlighted:bg-neutral-50 dark:text-neutral-200 dark:hover:bg-neutral-900 dark:data-highlighted:bg-neutral-900"
                    value={option.value}
                    label={option.label}
                  >
                    {#snippet children({ selected })}
                      <span class="truncate">{option.label}</span>
                      {#if selected}
                        <Check
                          class="ml-auto size-4 shrink-0 text-neutral-600 dark:text-neutral-400"
                        />
                      {/if}
                    {/snippet}
                  </Select.Item>
                {/each}
              </Select.Viewport>
            </Select.Content>
          </Select.Portal>
        </Select.Root>
      </div>
    </section>
  </div>
{/if}
