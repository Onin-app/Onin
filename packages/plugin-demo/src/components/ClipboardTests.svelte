<script lang="ts">
  import { getContext } from 'svelte';
  import { clipboard } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>剪贴板操作</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('clipboard.hasText', () => clipboard.hasText())}
    >
      <span class="api-name">hasText</span>
      <span class="api-desc">检查文本</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('clipboard.hasImage', () => clipboard.hasImage())}
    >
      <span class="api-name">hasImage</span>
      <span class="api-desc">检查图片</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('clipboard.readText', () => clipboard.readText())}
    >
      <span class="api-name">readText</span>
      <span class="api-desc">读取文本</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('clipboard.writeText', () =>
          clipboard.writeText('Hello from SDK Test! 你好！'),
        )}
    >
      <span class="api-name">writeText</span>
      <span class="api-desc">写入文本</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('clipboard.readImage', () => clipboard.readImage())}
    >
      <span class="api-name">readImage</span>
      <span class="api-desc">读取图片</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('clipboard.writeImage', () =>
          clipboard.writeImage(
            'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==',
          ),
        )}
    >
      <span class="api-name">writeImage</span>
      <span class="api-desc">写入图片</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('clipboard.copy', () =>
          clipboard.copy('使用 copy() 方法'),
        )}
    >
      <span class="api-name">copy</span>
      <span class="api-desc">复制 (别名)</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('clipboard.paste', () => clipboard.paste())}
    >
      <span class="api-name">paste</span>
      <span class="api-desc">粘贴 (别名)</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('clipboard.clear', () => clipboard.clear())}
    >
      <span class="api-name">clear</span>
      <span class="api-desc">清空剪贴板</span>
    </button>
  </div>
</div>
