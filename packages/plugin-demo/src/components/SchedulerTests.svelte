<script lang="ts">
  import { getContext } from 'svelte';
  import { scheduler } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>定时任务</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('scheduler.schedule', () =>
          scheduler.schedule('test-cron', '* * * * *', () =>
            logger.log('cron 任务执行'),
          ),
        )}
    >
      <span class="api-name">schedule</span>
      <span class="api-desc">注册任务 (cron)</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('scheduler.daily', () =>
          scheduler.daily('test-daily', '08:00', () =>
            logger.log('每日任务执行'),
          ),
        )}
    >
      <span class="api-name">daily</span>
      <span class="api-desc">每日任务</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('scheduler.hourly', () =>
          scheduler.hourly('test-hourly', 30, () =>
            logger.log('每小时任务执行'),
          ),
        )}
    >
      <span class="api-name">hourly</span>
      <span class="api-desc">每小时</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('scheduler.weekly', () =>
          scheduler.weekly('test-weekly', 1, '09:00', () =>
            logger.log('每周任务执行'),
          ),
        )}
    >
      <span class="api-name">weekly</span>
      <span class="api-desc">每周任务</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('scheduler.list', () => scheduler.list())}
    >
      <span class="api-name">list</span>
      <span class="api-desc">列出任务</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('scheduler.timeout', () =>
          scheduler.timeout('test-timeout', 5000, () =>
            logger.log('✅ 5秒 Timeout 任务执行完成', 'success'),
          ),
        )}
    >
      <span class="api-name">timeout</span>
      <span class="api-desc">5秒后执行</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('scheduler.at', () => {
          const time = Date.now() + 10000;
          return scheduler.at('test-at', time, () =>
            logger.log('✅ 10秒 At 任务执行完成', 'success'),
          );
        })}
    >
      <span class="api-name">at</span>
      <span class="api-desc">10秒后执行</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('scheduler.cancel', () => scheduler.cancel('test-cron'))}
    >
      <span class="api-name">cancel</span>
      <span class="api-desc">取消任务</span>
    </button>
  </div>
</div>
