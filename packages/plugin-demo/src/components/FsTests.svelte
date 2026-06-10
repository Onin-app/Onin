<script lang="ts">
  import { getContext } from 'svelte';
  import { fs } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>文件系统</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('fs.writeFile', () =>
          fs.writeFile(
            'test.txt',
            'Hello from SDK Test!\nTime: ' + new Date().toISOString(),
          ),
        )}
    >
      <span class="api-name">writeFile</span>
      <span class="api-desc">写入文件</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('fs.readFile', () => fs.readFile('test.txt'))}
    >
      <span class="api-name">readFile</span>
      <span class="api-desc">读取文件</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('fs.exists', () => fs.exists('test.txt'))}
    >
      <span class="api-name">exists</span>
      <span class="api-desc">检查存在</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('fs.createDir', () => fs.createDir('test-folder', true))}
    >
      <span class="api-name">createDir</span>
      <span class="api-desc">创建目录</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('fs.listDir', () => fs.listDir('.'))}
    >
      <span class="api-name">listDir</span>
      <span class="api-desc">列出目录</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('fs.getFileInfo', () => fs.getFileInfo('test.txt'))}
    >
      <span class="api-name">getFileInfo</span>
      <span class="api-desc">获取信息</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('fs.copyFile', () =>
          fs.copyFile('test.txt', 'test-copy.txt'),
        )}
    >
      <span class="api-name">copyFile</span>
      <span class="api-desc">复制文件</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('fs.moveFile', () =>
          fs.moveFile('test-copy.txt', 'test-moved.txt'),
        )}
    >
      <span class="api-name">moveFile</span>
      <span class="api-desc">移动文件</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('fs.deleteFile', () => fs.deleteFile('test-moved.txt'))}
    >
      <span class="api-name">deleteFile</span>
      <span class="api-desc">删除文件</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('fs.deleteDir', () => fs.deleteDir('test-folder', true))}
    >
      <span class="api-name">deleteDir</span>
      <span class="api-desc">删除目录</span>
    </button>
  </div>
</div>
