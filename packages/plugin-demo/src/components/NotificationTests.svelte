<script lang="ts">
  import { getContext } from 'svelte';
  import { notification } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>系统通知</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('notification.isPermissionGranted', () =>
          notification.isPermissionGranted(),
        )}
    >
      <span class="api-name">isPermissionGranted</span>
      <span class="api-desc">检查通知权限</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('notification.requestPermission', () =>
          notification.requestPermission(),
        )}
    >
      <span class="api-name">requestPermission</span>
      <span class="api-desc">申请通知权限</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('notification.show (Sound)', () =>
          notification.show({
            title: 'Sound Test',
            body: '这是一条带声音的通知',
            sound: 'default',
          }),
        )}
    >
      <span class="api-name">show (Sound)</span>
      <span class="api-desc">带声音显示</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('notification.show', () =>
          notification.show({
            title: 'SDK Test',
            body: '这是来自 SDK 测试的通知！',
          }),
        )}
    >
      <span class="api-name">show</span>
      <span class="api-desc">显示通知</span>
    </button>
  </div>
</div>
