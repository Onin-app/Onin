<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Button } from 'bits-ui';
  import type { PluginInfo } from '$lib/types/plugin';

  interface Props {
    plugins: PluginInfo[];
    searchQuery: string;
    showAllPlugins: boolean;
  }

  let { plugins, searchQuery, showAllPlugins }: Props = $props();

  const dispatch = createEventDispatcher<{
    togglePlugin: { plugin: PluginInfo };
    viewDetails: { plugin: PluginInfo };
  }>();

  // Filter plugins based on search query and view mode
  let filteredPlugins: PluginInfo[] = $state([]);
  
  $effect(() => {
    let filtered = plugins;

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(plugin => 
        plugin.name.toLowerCase().includes(query) ||
        plugin.description.toLowerCase().includes(query) ||
        plugin.author.toLowerCase().includes(query)
      );
    }

    // Filter by view mode (all vs installed)
    if (!showAllPlugins) {
      filtered = filtered.filter(plugin => plugin.enabled);
    }

    filteredPlugins = filtered;
  });

  const getStatusColor = (plugin: PluginInfo) => {
    if (plugin.status === 'error') return 'text-red-500';
    if (plugin.enabled && plugin.loaded) return 'text-green-500';
    if (plugin.enabled && !plugin.loaded) return 'text-yellow-500';
    return 'text-gray-500';
  };

  const getStatusText = (plugin: PluginInfo) => {
    if (plugin.status === 'error') return '错误';
    if (plugin.enabled && plugin.loaded) return '已启用';
    if (plugin.enabled && !plugin.loaded) return '加载中';
    return '已禁用';
  };
</script>

<div class="space-y-4">
  {#if filteredPlugins.length === 0}
    <div class="flex flex-col items-center justify-center py-12 text-neutral-500">
      <svg class="mb-4 h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/>
      </svg>
      <p class="text-lg font-medium">
        {searchQuery.trim() ? '未找到匹配的插件' : showAllPlugins ? '暂无可用插件' : '暂无已安装插件'}
      </p>
      <p class="text-sm">
        {searchQuery.trim() ? '尝试使用不同的搜索词' : '您可以手动导入插件或从插件商店安装'}
      </p>
    </div>
  {:else}
    {#each filteredPlugins as plugin (plugin.name)}
      <div class="rounded-lg border border-neutral-200 bg-white p-4 shadow-sm transition-shadow hover:shadow-md dark:border-neutral-700 dark:bg-neutral-900">
        <div class="flex items-start justify-between">
          <div class="flex-1">
            <div class="flex items-center gap-3">
              <h3 class="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
                {plugin.name}
              </h3>
              <span class="rounded-full px-2 py-1 text-xs font-medium bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400">
                v{plugin.version}
              </span>
              <span class="text-sm font-medium {getStatusColor(plugin)}">
                {getStatusText(plugin)}
              </span>
            </div>
            
            <p class="mt-2 text-sm text-neutral-600 dark:text-neutral-400">
              {plugin.description}
            </p>
            
            <div class="mt-2 flex items-center gap-4 text-xs text-neutral-500">
              <span>作者: {plugin.author}</span>
              {#if plugin.permissions && plugin.permissions.length > 0}
                <span>权限: {plugin.permissions.join(', ')}</span>
              {/if}
            </div>
          </div>
          
          <div class="flex items-center gap-2">
            <Button.Root
              class="rounded px-3 py-1 text-sm font-medium transition-colors hover:bg-neutral-100 dark:hover:bg-neutral-800"
              onclick={() => dispatch('viewDetails', { plugin })}
            >
              详情
            </Button.Root>
            
            <Button.Root
              class="rounded px-3 py-1 text-sm font-medium transition-colors {plugin.enabled 
                ? 'bg-red-100 text-red-700 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-900/50' 
                : 'bg-green-100 text-green-700 hover:bg-green-200 dark:bg-green-900/30 dark:text-green-400 dark:hover:bg-green-900/50'}"
              onclick={() => dispatch('togglePlugin', { plugin })}
              disabled={plugin.status === 'loading'}
            >
              {plugin.enabled ? '禁用' : '启用'}
            </Button.Root>
          </div>
        </div>
      </div>
    {/each}
  {/if}
</div>