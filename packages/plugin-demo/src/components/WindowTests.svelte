<script lang="ts">
  import { getContext } from 'svelte';
  import { pluginWindow } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>窗口事件</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() => {
        pluginWindow.onShow(() => logger.log('窗口显示了!'));
        logger.log('✓ onShow 回调已注册', 'success');
      }}
    >
      <span class="api-name">onShow</span>
      <span class="api-desc">注册窗口显示回调</span>
    </button>
    <button
      class="test-btn"
      onclick={() => {
        pluginWindow.onHide(() => logger.log('窗口隐藏了!'));
        logger.log('✓ onHide 回调已注册', 'success');
      }}
    >
      <span class="api-name">onHide</span>
      <span class="api-desc">注册窗口隐藏回调</span>
    </button>
    <button
      class="test-btn"
      onclick={() => {
        pluginWindow.onFocus(() => logger.log('窗口获得焦点!'));
        logger.log('✓ onFocus 回调已注册', 'success');
      }}
    >
      <span class="api-name">onFocus</span>
      <span class="api-desc">注册焦点获得回调</span>
    </button>
    <button
      class="test-btn"
      onclick={() => {
        pluginWindow.onBlur(() => logger.log('窗口失去焦点!'));
        logger.log('✓ onBlur 回调已注册', 'success');
      }}
    >
      <span class="api-name">onBlur</span>
      <span class="api-desc">注册焦点失去回调</span>
    </button>
  </div>
</div>
<div class="api-group">
  <h3>运行模式</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() => {
        const mode = pluginWindow.getMode();
        logger.log(`当前运行模式: ${mode}`, 'success');
      }}
    >
      <span class="api-name">getMode</span>
      <span class="api-desc">获取当前运行模式</span>
    </button>
  </div>
</div>
