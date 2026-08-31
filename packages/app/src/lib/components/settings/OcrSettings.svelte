<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import {
    Select,
    SelectTrigger,
    SelectContent,
    SelectItem,
  } from "$lib/components/ui/select";
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

  // 派生状态：可用的 Provider 列表
  let providerOptions = $derived.by(() => {
    const options = [
      {
        value: "default",
        label: "跟随全局默认 AI 提供商",
      },
    ];

    // 过滤出具备多模态能力或未拉取模型的提供商
    const supportedProviders = aiConfig.providers.filter(providerSupportsImage);

    supportedProviders.forEach((provider) => {
      options.push({
        value: provider.id,
        label: provider.display_name || provider.name,
      });
    });

    return options;
  });

  // 派生状态：获取当前生效的 Provider 的默认模型名称
  let activeProviderDefaultModelName = $derived.by(() => {
    const targetProviderId =
      config?.ocr_provider_id || aiConfig.active_provider_id;
    const provider = aiConfig.providers.find((p) => p.id === targetProviderId);
    if (!provider) return "无默认模型";

    if (provider.default_model) {
      const found = provider.models?.find(
        (m) => m.id === provider.default_model,
      );
      return found?.name || provider.default_model;
    }
    return "无默认模型";
  });

  // 派生状态：可供选择的模型列表
  let modelOptions = $derived.by(() => {
    const options = [];

    // 1. 如果没有指定提供商，即“跟随全局默认”
    if (!config?.ocr_provider_id) {
      options.push({
        value: "default",
        label: `跟随所选提供商默认 (${activeProviderDefaultModelName})`,
      });
    } else {
      // 2. 如果指定了特定提供商
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
  <div class="text-muted-foreground py-6 text-center text-xs">
    正在加载文字识别设置...
  </div>
{:else if config}
  <div class="flex flex-col gap-6">
    <!-- 默认引擎设置 -->
    <section class="border-border/50 border-t pt-4">
      <div class="flex flex-col gap-1.5">
        <h4 class="text-foreground text-sm font-semibold tracking-tight">
          默认识别引擎
        </h4>
        <p class="text-muted-foreground/75 text-xs leading-normal">
          设置每次打开文字识别页面时，首选激活的 OCR 识别模式。
        </p>

        <Select
          type="single"
          value={config.ocr_default_engine || "local"}
          onValueChange={handleEngineChange}
        >
          <SelectTrigger
            class="border-input bg-background text-foreground focus:ring-ring mt-2 h-10 w-full cursor-pointer rounded-xl border text-xs transition-[border-color,box-shadow,transform] duration-120 ease-out focus:ring-1 focus:outline-none active:scale-[0.99]"
          >
            <span class="truncate">
              {engineOptions.find(
                (o) => o.value === (config?.ocr_default_engine || "local"),
              )?.label}
            </span>
          </SelectTrigger>
          <SelectContent
            class="bg-popover text-popover-foreground border-border/60 z-50 w-[var(--bits-select-anchor-width)] rounded-xl border shadow-xl"
          >
            {#each engineOptions as option (option.value)}
              <SelectItem
                value={option.value}
                label={option.label}
                class="text-popover-foreground data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground cursor-pointer rounded-lg px-2.5 py-1.5 text-xs font-medium outline-none"
              >
                <span class="truncate">{option.label}</span>
              </SelectItem>
            {/each}
          </SelectContent>
        </Select>
      </div>
    </section>

    <!-- AI 识别专属配置 -->
    {#if (config.ocr_default_engine || "local") === "ai"}
      <section class="border-border/50 border-t pt-4">
        <div class="flex flex-col gap-4">
          <div>
            <h4 class="text-foreground text-sm font-semibold tracking-tight">
              AI 视觉模型设置
            </h4>
            <p class="text-muted-foreground/75 mt-0.5 text-xs leading-normal">
              指定 OCR 识别所使用的视觉大模型服务。支持按需定制专属 Provider
              与模型。
            </p>
          </div>

          <!-- Provider 选择 -->
          <div class="flex flex-col gap-1.5">
            <div class="text-foreground text-xs font-medium">
              AI 提供商 (Provider)
            </div>
            <Select
              type="single"
              value={config.ocr_provider_id || "default"}
              onValueChange={handleProviderChange}
            >
              <SelectTrigger
                class="border-input bg-background text-foreground focus:ring-ring h-10 w-full cursor-pointer rounded-xl border text-xs transition-[border-color,box-shadow,transform] duration-120 ease-out focus:ring-1 focus:outline-none active:scale-[0.99]"
              >
                <span class="truncate">
                  {providerOptions.find(
                    (p) => p.value === (config?.ocr_provider_id || "default"),
                  )?.label}
                </span>
              </SelectTrigger>
              <SelectContent
                class="bg-popover text-popover-foreground border-border/60 z-50 w-[var(--bits-select-anchor-width)] rounded-xl border shadow-xl"
              >
                {#each providerOptions as provider (provider.value)}
                  <SelectItem
                    value={provider.value}
                    label={provider.label}
                    class="text-popover-foreground data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground cursor-pointer rounded-lg px-2.5 py-1.5 text-xs font-medium outline-none"
                  >
                    <span class="truncate">{provider.label}</span>
                  </SelectItem>
                {/each}
              </SelectContent>
            </Select>
            <p class="text-muted-foreground/75 text-[11px]">
              如需添加新的 AI 提供商，请前往「AI 设置」页面配置。
            </p>
          </div>

          <!-- 模型选择 -->
          <div class="flex flex-col gap-1.5">
            <div class="text-foreground text-xs font-medium">
              指定识别模型 (Model)
            </div>
            <Select
              type="single"
              value={config.ocr_model_id || "default"}
              onValueChange={handleModelChange}
            >
              <SelectTrigger
                class="border-input bg-background text-foreground focus:ring-ring h-10 w-full cursor-pointer rounded-xl border text-xs transition-[border-color,box-shadow,transform] duration-120 ease-out focus:ring-1 focus:outline-none active:scale-[0.99]"
              >
                <span class="truncate">
                  {modelOptions.find(
                    (m) => m.value === (config?.ocr_model_id || "default"),
                  )?.label || "跟随默认"}
                </span>
              </SelectTrigger>
              <SelectContent
                class="bg-popover text-popover-foreground border-border/60 z-50 w-[var(--bits-select-anchor-width)] rounded-xl border shadow-xl"
              >
                {#each modelOptions as model (model.value)}
                  <SelectItem
                    value={model.value}
                    label={model.label}
                    class="text-popover-foreground data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground cursor-pointer rounded-lg px-2.5 py-1.5 text-xs font-medium outline-none"
                  >
                    <span class="truncate">{model.label}</span>
                  </SelectItem>
                {/each}
              </SelectContent>
            </Select>
            <p class="text-muted-foreground/75 text-[11px]">
              仅多模态（Vision / VL）模型支持图片 OCR 文本提取与排版分析。
            </p>
          </div>
        </div>
      </section>
    {/if}
  </div>
{/if}
