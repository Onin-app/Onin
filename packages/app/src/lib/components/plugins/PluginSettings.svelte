<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button, ScrollArea } from "bits-ui";
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
    // 清除之前的定时器
    if (saveTimeout !== null) {
      clearTimeout(saveTimeout);
    }

    // 设置新的定时器（500ms 防抖）
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

  // 初始化加载
  $effect(() => {
    loadSettings();
  });
</script>

<div class="flex h-full flex-col bg-neutral-50 dark:bg-neutral-900">
  <!-- Header -->
  <div
    class="flex items-center gap-2 border-b border-neutral-200 bg-white px-4 py-3 dark:border-neutral-700 dark:bg-neutral-800"
  >
    <Button.Root
      class="rounded p-1.5 hover:bg-neutral-100 dark:hover:bg-neutral-700"
      onclick={onback}
      aria-label="返回"
    >
      <ArrowLeft class="h-5 w-5" />
    </Button.Root>
    <h2 class="text-lg font-semibold">{pluginName} - 设置</h2>
  </div>

  <!-- Content -->
  <ScrollArea.Root class="flex-1" type="hover">
    <ScrollArea.Viewport class="h-full w-full overflow-x-hidden">
      <div class="p-6 pr-8">
        {#if loading}
          <div class="flex items-center justify-center py-12">
            <div class="text-neutral-500">加载中...</div>
          </div>
        {:else if loadError}
          <div class="mx-auto max-w-2xl">
            <div
              class="rounded-lg border border-red-200 bg-red-50 p-4 text-red-800 dark:border-red-800 dark:bg-red-900/20 dark:text-red-200"
            >
              <p class="font-semibold">加载失败</p>
              <p class="mt-1 text-sm">{loadError}</p>
            </div>
          </div>
        {:else}
          <div class="mx-auto max-w-2xl">
            {#if saveError}
              <div
                class="mb-4 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-800 dark:border-red-800 dark:bg-red-900/20 dark:text-red-200"
              >
                {saveError}
              </div>
            {/if}
            <div
              class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-700 dark:bg-neutral-800"
            >
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
    </ScrollArea.Viewport>
    <ScrollArea.Scrollbar
      orientation="vertical"
      class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none rounded-full border-l border-l-transparent p-px transition-all duration-200 select-none hover:w-3"
    >
      <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
    </ScrollArea.Scrollbar>
    <ScrollArea.Corner />
  </ScrollArea.Root>
</div>
