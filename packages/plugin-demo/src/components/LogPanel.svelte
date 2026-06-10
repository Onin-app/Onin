<script lang="ts">
  import { getContext } from 'svelte';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');

  let logContainer = $state<HTMLDivElement | null>(null);

  // 每当 logs 变化时自动滚动到底部
  $effect(() => {
    // 显式引用 logs.length 以触发这个 effect 监听日志数组大小的变化
    logger.logs.length;
    if (logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });
</script>

<section class="log-panel">
  <div class="log-header">
    <h3>📝 日志输出</h3>
    <span class="log-count">{logger.logs.length} 条</span>
  </div>
  <div bind:this={logContainer} class="log-container">
    {#each logger.logs as entry}
      <div class="log-entry log-{entry.type}">
        <span class="log-time">[{entry.time}]</span>
        <span class="log-message">{entry.message}</span>
      </div>
    {/each}
    {#if logger.logs.length === 0}
      <div class="log-empty">点击测试按钮查看输出...</div>
    {/if}
  </div>
</section>

<style>
  .log-panel {
    flex: 1;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .log-header {
    padding: 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .log-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .log-count {
    font-size: 11px;
    color: var(--text-muted);
  }

  .log-container {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    line-height: 1.6;
  }

  .log-entry {
    margin-bottom: 8px;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .log-time {
    color: var(--text-muted);
    margin-right: 8px;
  }

  .log-info {
    color: var(--text-secondary);
  }

  .log-success {
    color: var(--success);
  }

  .log-error {
    color: var(--error);
  }

  .log-empty {
    color: var(--text-muted);
    text-align: center;
    padding-top: 40px;
    font-style: italic;
  }
</style>
