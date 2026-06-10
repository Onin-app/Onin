<script lang="ts">
  import { getContext } from 'svelte';
  import { toast } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>Toast 提示</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('toast.show', () => toast.show('这是一条通用提示'))}
    >
      <span class="api-name">show</span>
      <span class="api-desc">显示通用提示</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('toast.success', () =>
          toast.success('操作成功！', { duration: 3000 }),
        )}
    >
      <span class="api-name">success</span>
      <span class="api-desc">显示成功提示 (3s)</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('toast.error', () => toast.error('发生错误，请重试'))}
    >
      <span class="api-name">error</span>
      <span class="api-desc">显示错误提示</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('toast.warning', () =>
          toast.warning('这是一条警告信息', { duration: 5000 }),
        )}
    >
      <span class="api-name">warning</span>
      <span class="api-desc">显示警告提示 (5s)</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('toast.info', () => toast.info('这是一条普通信息'))}
    >
      <span class="api-name">info</span>
      <span class="api-desc">显示信息提示</span>
    </button>
  </div>
</div>
