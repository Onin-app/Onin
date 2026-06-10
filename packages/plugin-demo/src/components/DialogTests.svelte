<script lang="ts">
  import { getContext } from 'svelte';
  import { dialog } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>对话框</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('dialog.showMessage', () =>
          dialog.showMessage({
            title: 'SDK Test',
            message: '这是一个消息对话框',
          }),
        )}
    >
      <span class="api-name">showMessage</span>
      <span class="api-desc">消息框</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('dialog.showConfirm', () =>
          dialog.showConfirm({ title: '确认', message: '是否继续操作？' }),
        )}
    >
      <span class="api-name">showConfirm</span>
      <span class="api-desc">确认框</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('dialog.showOpen', () =>
          dialog.showOpen({ title: '选择文件', multiple: false }),
        )}
    >
      <span class="api-name">showOpen</span>
      <span class="api-desc">打开文件</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('dialog.showSave', () =>
          dialog.showSave({ title: '保存文件', defaultPath: 'test.txt' }),
        )}
    >
      <span class="api-name">showSave</span>
      <span class="api-desc">保存文件</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('dialog.info', () =>
          dialog.info('这是一条信息', '信息'),
        )}
    >
      <span class="api-name">info</span>
      <span class="api-desc">信息提示</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('dialog.warning', () =>
          dialog.warning('这是一条警告', '警告'),
        )}
    >
      <span class="api-name">warning</span>
      <span class="api-desc">警告提示</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('dialog.error', () =>
          dialog.error('这是一条错误', '错误'),
        )}
    >
      <span class="api-name">error</span>
      <span class="api-desc">错误提示</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('dialog.confirm', () =>
          dialog.confirm('确定要执行此操作吗？', '确认'),
        )}
    >
      <span class="api-name">confirm</span>
      <span class="api-desc">确认 (简化)</span>
    </button>
  </div>
</div>
