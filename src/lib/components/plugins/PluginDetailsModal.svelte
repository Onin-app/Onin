<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Button } from 'bits-ui';
  import type { PluginInfo } from '$lib/types/plugin';

  interface Props {
    plugin: PluginInfo | null;
    open: boolean;
  }

  let { plugin, open = $bindable() }: Props = $props();

  const dispatch = createEventDispatcher<{
    togglePlugin: { plugin: PluginInfo };
    saveConfig: { plugin: PluginInfo; config: Record<string, any> };
  }>();

  let configData = $state<Record<string, any>>({});
  let configMode = $state<'view' | 'edit'>('view');

  // Initialize config data when plugin changes
  $effect(() => {
    if (plugin) {
      // TODO: Load actual plugin configuration
      configData = {
        greeting: "Hello from plugin!",
        enabled_features: ["notifications", "storage"],
        theme: "auto"
      };
    }
  });

  const handleClose = () => {
    open = false;
    configMode = 'view';
  };

  const handleTogglePlugin = () => {
    if (plugin) {
      dispatch('togglePlugin', { plugin });
    }
  };

  const handleSaveConfig = () => {
    if (plugin) {
      dispatch('saveConfig', { plugin, config: configData });
      configMode = 'view';
    }
  };

  const handleCancelEdit = () => {
    configMode = 'view';
    // Reset config data
    if (plugin) {
      configData = {
        greeting: "Hello from plugin!",
        enabled_features: ["notifications", "storage"],
        theme: "auto"
      };
    }
  };

  const formatPermissions = (permissions: string[] | undefined) => {
    if (!permissions || permissions.length === 0) return '无';
    return permissions.join(', ');
  };

  const getStatusBadgeClass = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400';
      case 'inactive':
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400';
      case 'error':
        return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400';
      case 'loading':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400';
    }
  };
</script>

{#if open && plugin}
  <!-- Modal backdrop -->
  <div 
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
    onclick={handleClose}
    onkeydown={(e) => e.key === 'Escape' && handleClose()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="plugin-details-title"
    tabindex="-1"
  >
    <!-- Modal content -->
    <div 
      class="w-full max-w-2xl max-h-[90vh] overflow-hidden rounded-lg bg-white shadow-xl dark:bg-neutral-900"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="document"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-neutral-200 p-6 dark:border-neutral-700">
        <div class="flex items-center gap-3">
          <h2 id="plugin-details-title" class="text-xl font-semibold text-neutral-900 dark:text-neutral-100">
            {plugin.name}
          </h2>
          <span class="rounded-full px-2 py-1 text-xs font-medium bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400">
            v{plugin.version}
          </span>
          <span class="rounded-full px-2 py-1 text-xs font-medium {getStatusBadgeClass(plugin.status)}">
            {plugin.status === 'active' ? '已启用' : 
             plugin.status === 'inactive' ? '已禁用' : 
             plugin.status === 'error' ? '错误' : '加载中'}
          </span>
        </div>
        
        <Button.Root
          class="rounded p-2 hover:bg-neutral-100 dark:hover:bg-neutral-800"
          onclick={handleClose}
          aria-label="关闭"
        >
          <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </Button.Root>
      </div>

      <!-- Content -->
      <div class="max-h-[calc(90vh-8rem)] overflow-y-auto p-6">
        <!-- Basic Information -->
        <div class="mb-6">
          <h3 class="mb-3 text-lg font-medium text-neutral-900 dark:text-neutral-100">基本信息</h3>
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <div>
              <span class="block text-sm font-medium text-neutral-700 dark:text-neutral-300">名称</span>
              <p class="mt-1 text-sm text-neutral-900 dark:text-neutral-100">{plugin.name}</p>
            </div>
            <div>
              <span class="block text-sm font-medium text-neutral-700 dark:text-neutral-300">版本</span>
              <p class="mt-1 text-sm text-neutral-900 dark:text-neutral-100">{plugin.version}</p>
            </div>
            <div>
              <span class="block text-sm font-medium text-neutral-700 dark:text-neutral-300">作者</span>
              <p class="mt-1 text-sm text-neutral-900 dark:text-neutral-100">{plugin.author}</p>
            </div>
            <div>
              <span class="block text-sm font-medium text-neutral-700 dark:text-neutral-300">状态</span>
              <p class="mt-1 text-sm text-neutral-900 dark:text-neutral-100">
                {plugin.enabled ? '已启用' : '已禁用'} 
                {plugin.loaded ? '(已加载)' : '(未加载)'}
              </p>
            </div>
          </div>
          
          <div class="mt-4">
            <span class="block text-sm font-medium text-neutral-700 dark:text-neutral-300">描述</span>
            <p class="mt-1 text-sm text-neutral-900 dark:text-neutral-100">{plugin.description}</p>
          </div>

          <div class="mt-4">
            <span class="block text-sm font-medium text-neutral-700 dark:text-neutral-300">权限</span>
            <p class="mt-1 text-sm text-neutral-900 dark:text-neutral-100">{formatPermissions(plugin.permissions)}</p>
          </div>

          {#if plugin.path}
            <div class="mt-4">
              <span class="block text-sm font-medium text-neutral-700 dark:text-neutral-300">路径</span>
              <p class="mt-1 text-sm font-mono text-neutral-600 dark:text-neutral-400">{plugin.path}</p>
            </div>
          {/if}
        </div>

        <!-- Configuration Section -->
        <div class="mb-6">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-lg font-medium text-neutral-900 dark:text-neutral-100">配置</h3>
            {#if configMode === 'view'}
              <Button.Root
                class="rounded px-3 py-1 text-sm font-medium transition-colors hover:bg-neutral-100 dark:hover:bg-neutral-800"
                onclick={() => configMode = 'edit'}
              >
                编辑
              </Button.Root>
            {:else}
              <div class="flex gap-2">
                <Button.Root
                  class="rounded px-3 py-1 text-sm font-medium transition-colors bg-green-100 text-green-700 hover:bg-green-200 dark:bg-green-900/30 dark:text-green-400 dark:hover:bg-green-900/50"
                  onclick={handleSaveConfig}
                >
                  保存
                </Button.Root>
                <Button.Root
                  class="rounded px-3 py-1 text-sm font-medium transition-colors hover:bg-neutral-100 dark:hover:bg-neutral-800"
                  onclick={handleCancelEdit}
                >
                  取消
                </Button.Root>
              </div>
            {/if}
          </div>

          <div class="rounded-lg border border-neutral-200 bg-neutral-50 p-4 dark:border-neutral-700 dark:bg-neutral-800">
            {#if configMode === 'view'}
              <pre class="text-sm text-neutral-900 dark:text-neutral-100 whitespace-pre-wrap">{JSON.stringify(configData, null, 2)}</pre>
            {:else}
              <div class="space-y-4">
                {#each Object.entries(configData) as [key, value]}
                  <div>
                    {#if typeof value === 'string'}
                      <label class="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1" for="config-{key}">
                        {key}
                      </label>
                      <input
                        id="config-{key}"
                        type="text"
                        bind:value={configData[key]}
                        class="w-full rounded border border-neutral-300 px-3 py-2 text-sm focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-900 dark:focus:border-neutral-400"
                      />
                    {:else if typeof value === 'boolean'}
                      <label class="flex items-center gap-2">
                        <input
                          type="checkbox"
                          bind:checked={configData[key]}
                          class="rounded border-neutral-300 focus:ring-neutral-500 dark:border-neutral-600"
                        />
                        <span class="text-sm text-neutral-700 dark:text-neutral-300">{key}</span>
                      </label>
                    {:else}
                      <label class="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1" for="config-{key}">
                        {key}
                      </label>
                      <textarea
                        id="config-{key}"
                        bind:value={configData[key]}
                        rows="3"
                        class="w-full rounded border border-neutral-300 px-3 py-2 text-sm focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-900 dark:focus:border-neutral-400"
                      ></textarea>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- Error Display -->
        {#if plugin.status === 'error'}
          <div class="mb-6">
            <h3 class="mb-3 text-lg font-medium text-red-700 dark:text-red-400">错误信息</h3>
            <div class="rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-800 dark:bg-red-900/20">
              <div class="flex items-center gap-2 mb-2">
                <svg class="h-4 w-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                <span class="text-sm font-medium text-red-800 dark:text-red-200">插件运行错误</span>
              </div>
              <p class="text-sm text-red-700 dark:text-red-300">
                插件在运行过程中遇到错误，请检查插件配置或联系插件作者。
              </p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between border-t border-neutral-200 p-6 dark:border-neutral-700">
        <div class="flex gap-2">
          <Button.Root
            class="rounded px-4 py-2 text-sm font-medium transition-colors {plugin.enabled 
              ? 'bg-red-100 text-red-700 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-900/50' 
              : 'bg-green-100 text-green-700 hover:bg-green-200 dark:bg-green-900/30 dark:text-green-400 dark:hover:bg-green-900/50'}"
            onclick={handleTogglePlugin}
            disabled={plugin.status === 'loading'}
          >
            {plugin.enabled ? '禁用插件' : '启用插件'}
          </Button.Root>
        </div>
        
        <Button.Root
          class="rounded px-4 py-2 text-sm font-medium transition-colors hover:bg-neutral-100 dark:hover:bg-neutral-800"
          onclick={handleClose}
        >
          关闭
        </Button.Root>
      </div>
    </div>
  </div>
{/if}