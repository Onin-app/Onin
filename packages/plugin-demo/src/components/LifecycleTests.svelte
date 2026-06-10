<script lang="ts">
  import { getContext } from 'svelte';
  import { lifecycle } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>生命周期钩子</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() => {
        lifecycle.onLoad(() => logger.log('onLoad 回调触发'));
        logger.log('✓ onLoad 回调已注册', 'success');
      }}
    >
      <span class="api-name">onLoad</span>
      <span class="api-desc">注册加载回调</span>
    </button>
    <button
      class="test-btn"
      onclick={() => {
        lifecycle.onUnload(() => logger.log('onUnload 回调触发'));
        logger.log('✓ onUnload 回调已注册', 'success');
      }}
    >
      <span class="api-name">onUnload</span>
      <span class="api-desc">注册卸载回调</span>
    </button>
  </div>
</div>
