<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { ArrowLeft } from "phosphor-svelte";
  import SettingField from "./SettingField.svelte";
  import type {
    PluginSettingsSchema,
    PluginSettingsValues,
  } from "$lib/types/plugin-settings";

  interface Props {
    pluginId: string;
    pluginName: string;
    schema: PluginSettingsSchema;
    onback: () => void;
  }

  let { pluginId, pluginName, schema, onback }: Props = $props();

  let values = $state<PluginSettingsValues>({});
  let loading = $state(true);

  let loadError = $state<string | null>(null);
  let saveError = $state<string | null>(null);

  // 加载设置值
  async function loadSettings() {
    try {
      loading = true;
      loadError = null;
      const savedValues = await invoke<PluginSettingsValues>(
        "get_plugin_settings",
        { pluginId },
      );

      // 合并默认值和已保存的值
      const merged: PluginSettingsValues = {};
      for (const field of schema.fields) {
        if (savedValues && savedValues[field.key] !== undefined) {
          merged[field.key] = savedValues[field.key];
        } else if (
          "defaultValue" in field &&
          field.defaultValue !== undefined
        ) {
          merged[field.key] = field.defaultValue;
        }
      }
      values = merged;
    } catch (error) {
      console.error("Failed to load plugin settings:", error);
      loadError = error instanceof Error ? error.message : "加载设置失败";
      // 使用默认值
      const defaults: PluginSettingsValues = {};
      for (const field of schema.fields) {
        if ("defaultValue" in field && field.defaultValue !== undefined) {
          defaults[field.key] = field.defaultValue;
        }
      }
      values = defaults;
    } finally {
      loading = false;
    }
  }

  // 自动保存设置（带防抖）
  let saveTimeout: number | null = null;
  async function autoSaveSettings(key: string, value: any) {
    if (saveTimeout !== null) {
      clearTimeout(saveTimeout);
    }

    saveTimeout = setTimeout(async () => {
      try {
        saveError = null;
        await invoke("save_plugin_settings", {
          pluginId,
          settings: values,
        });
      } catch (error) {
        console.error("Failed to auto-save plugin settings:", error);
        saveError = error instanceof Error ? error.message : "保存设置失败";
      }
    }, 500) as unknown as number;
  }

  $effect(() => {
    loadSettings();
  });
</script>

<div class="bg-background flex h-full flex-col">
  <!-- Header -->
  <div class="bg-card flex items-center gap-2 border-b px-4 py-3">
    <Button
      variant="ghost"
      size="icon"
      class="h-8 w-8"
      onclick={onback}
      aria-label="返回"
    >
      <ArrowLeft class="h-4 w-4" />
    </Button>
    <h2 class="text-foreground text-base font-semibold">{pluginName} - 设置</h2>
  </div>

  <!-- Content -->
  <ScrollArea class="flex-1" viewportClass="h-full w-full overflow-x-hidden">
    <div class="p-6 pr-8">
      {#if loading}
        <div class="flex items-center justify-center py-12">
          <div class="text-muted-foreground text-sm">加载中...</div>
        </div>
      {:else if loadError}
        <div class="mx-auto max-w-2xl">
          <div
            class="border-destructive/20 bg-destructive/10 text-destructive rounded-xl border p-4"
          >
            <p class="text-sm font-semibold">加载失败</p>
            <p class="mt-1 text-xs">{loadError}</p>
          </div>
        </div>
      {:else}
        <div class="mx-auto max-w-2xl">
          {#if saveError}
            <div
              class="border-destructive/20 bg-destructive/10 text-destructive mb-4 rounded-xl border p-3 text-xs"
            >
              {saveError}
            </div>
          {/if}
          <div class="bg-card rounded-xl border p-4">
            {#each schema.fields as field (field.key)}
              <SettingField
                {field}
                bind:value={values[field.key]}
                onChange={(newValue) => {
                  values[field.key] = newValue;
                  autoSaveSettings(field.key, newValue);
                }}
              />
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </ScrollArea>
</div>
