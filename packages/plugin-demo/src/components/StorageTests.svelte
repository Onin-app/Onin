<script lang="ts">
  import { getContext } from 'svelte';
  import { storage } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>持久化存储</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('storage.setItem', () =>
          storage.setItem('test-key', { value: 'Hello', time: Date.now() }),
        )}
    >
      <span class="api-name">setItem</span>
      <span class="api-desc">保存数据</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('storage.getItem', () => storage.getItem('test-key'))}
    >
      <span class="api-name">getItem</span>
      <span class="api-desc">读取数据</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('storage.removeItem', () =>
          storage.removeItem('test-key'),
        )}
    >
      <span class="api-name">removeItem</span>
      <span class="api-desc">删除数据</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('storage.keys', () => storage.keys())}
    >
      <span class="api-name">keys</span>
      <span class="api-desc">获取所有键</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('storage.setItems', () =>
          storage.setItems({ 'batch-1': 1, 'batch-2': 2, 'batch-3': 3 }),
        )}
    >
      <span class="api-name">setItems</span>
      <span class="api-desc">批量保存</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('storage.getItems', () =>
          storage.getItems(['batch-1', 'batch-2', 'batch-3']),
        )}
    >
      <span class="api-name">getItems</span>
      <span class="api-desc">批量读取</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('storage.getAll', () => storage.getAll())}
    >
      <span class="api-name">getAll</span>
      <span class="api-desc">获取全部</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('storage.setAll', () =>
          storage.setAll({ replaced: true, time: Date.now() }),
        )}
    >
      <span class="api-name">setAll</span>
      <span class="api-desc">替换全部</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('storage.clear', () => storage.clear())}
    >
      <span class="api-name">clear</span>
      <span class="api-desc">清空所有</span>
    </button>
  </div>
</div>
