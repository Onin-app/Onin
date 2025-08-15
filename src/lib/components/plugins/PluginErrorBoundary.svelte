<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Button } from 'bits-ui';
  import type { PluginError } from '$lib/types/plugin';

  interface Props {
    errors: PluginError[];
  }

  let { errors }: Props = $props();

  const dispatch = createEventDispatcher<{
    clearError: { plugin: string };
    clearAllErrors: void;
    retryPlugin: { plugin: string };
  }>();

  const getErrorIcon = (type: string) => {
    switch (type) {
      case 'load_failed':
        return 'M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z';
      case 'invalid_manifest':
        return 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z';
      case 'permission_denied':
        return 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z';
      case 'version_incompatible':
        return 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z';
      default:
        return 'M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z';
    }
  };

  const getErrorTypeText = (type: string) => {
    switch (type) {
      case 'load_failed':
        return '加载失败';
      case 'invalid_manifest':
        return '清单文件无效';
      case 'permission_denied':
        return '权限被拒绝';
      case 'version_incompatible':
        return '版本不兼容';
      default:
        return '未知错误';
    }
  };
</script>

{#if errors.length > 0}
  <div class="mb-4 space-y-2">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-red-800 dark:text-red-200">
        插件错误 ({errors.length})
      </h3>
      {#if errors.length > 1}
        <Button.Root
          class="text-xs text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-200"
          onclick={() => dispatch('clearAllErrors')}
        >
          清除所有
        </Button.Root>
      {/if}
    </div>
    
    {#each errors as error (error.plugin + error.message)}
      <div class="rounded-lg border border-red-200 bg-red-50 p-3 dark:border-red-800 dark:bg-red-900/20">
        <div class="flex items-start justify-between">
          <div class="flex items-start gap-3">
            <svg class="h-4 w-4 text-red-500 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getErrorIcon(error.type)}/>
            </svg>
            <div class="flex-1">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium text-red-800 dark:text-red-200">
                  {error.plugin === 'system' ? '系统错误' : `插件 "${error.plugin}"`}
                </span>
                <span class="rounded px-1.5 py-0.5 text-xs font-medium bg-red-200 text-red-800 dark:bg-red-800 dark:text-red-200">
                  {getErrorTypeText(error.type)}
                </span>
              </div>
              <p class="mt-1 text-sm text-red-700 dark:text-red-300">{error.message}</p>
            </div>
          </div>
          
          <div class="flex items-center gap-1 ml-2">
            {#if error.plugin !== 'system' && error.type === 'load_failed'}
              <Button.Root
                class="text-xs text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-200 px-2 py-1"
                onclick={() => dispatch('retryPlugin', { plugin: error.plugin })}
              >
                重试
              </Button.Root>
            {/if}
            <Button.Root
              class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-200 p-1"
              onclick={() => dispatch('clearError', { plugin: error.plugin })}
              aria-label="清除错误"
            >
              <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </Button.Root>
          </div>
        </div>
      </div>
    {/each}
  </div>
{/if}