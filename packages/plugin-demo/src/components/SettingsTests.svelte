<script lang="ts">
  import { getContext } from 'svelte';
  import { settings } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>插件设置</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('settings.useSettingsSchema', () =>
          settings.useSettingsSchema([
            {
              key: 'apiKey',
              type: 'text',
              label: 'API Key',
              placeholder: '输入 API Key',
            },
            {
              key: 'enabled',
              type: 'switch',
              label: '启用功能',
              defaultValue: true,
            },
            {
              key: 'theme',
              type: 'select',
              label: '主题',
              options: [
                { label: '浅色', value: 'light' },
                { label: '深色', value: 'dark' },
              ],
              defaultValue: 'dark',
            },
          ]),
        )}
    >
      <span class="api-name">useSettingsSchema</span>
      <span class="api-desc">注册定义</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('settings.getSchema', async () => settings.getSchema())}
    >
      <span class="api-name">getSchema</span>
      <span class="api-desc">获取定义</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('settings.getAll', () => settings.getAll())}
    >
      <span class="api-name">getAll</span>
      <span class="api-desc">获取所有值</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('settings.onChange', async () => {
          await settings.onChange((newSettings) => {
            logger.log(`⚙️ 设置已更改: ${JSON.stringify(newSettings)}`, 'info');
          });
          logger.log('✓ 已添加设置更改侦听器', 'success');
          return '监听中...';
        })}
    >
      <span class="api-name">onChange</span>
      <span class="api-desc">监听设置更改</span>
    </button>
  </div>
</div>
