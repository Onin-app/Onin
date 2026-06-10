<script lang="ts">
  import { getContext } from 'svelte';
  import { command } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>命令管理</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('command.register', () =>
          command.register({
            code: 'test-cmd',
            name: '测试命令',
            description: '由 SDK Test 动态注册',
            keywords: [{ name: 'test' }],
          }),
        )}
    >
      <span class="api-name">register</span>
      <span class="api-desc">注册动态命令</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('command.handle', async () => {
          await command.handle(async (code, args) => {
            logger.log(`收到命令: ${code}, args: ${JSON.stringify(args)}`);
            return { handled: true };
          });
          return '处理器已注册';
        })}
    >
      <span class="api-name">handle</span>
      <span class="api-desc">注册命令处理器</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('command.remove', () => command.remove('test-cmd'))}
    >
      <span class="api-name">remove</span>
      <span class="api-desc">移除命令</span>
    </button>
  </div>
</div>
