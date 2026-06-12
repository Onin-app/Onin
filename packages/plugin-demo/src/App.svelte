<script lang="ts">
  import { setContext } from 'svelte';
  import { LoggerState } from './state/logger.svelte';
  import { lifecycle, notification } from 'onin-sdk';

  // 导入拆分后的子组件
  import LogPanel from './components/LogPanel.svelte';
  import LifecycleTests from './components/LifecycleTests.svelte';
  import WindowTests from './components/WindowTests.svelte';
  import CommandTests from './components/CommandTests.svelte';
  import StorageTests from './components/StorageTests.svelte';
  import NotificationTests from './components/NotificationTests.svelte';
  import ToastTests from './components/ToastTests.svelte';
  import ClipboardTests from './components/ClipboardTests.svelte';
  import FsTests from './components/FsTests.svelte';
  import HttpTests from './components/HttpTests.svelte';
  import DialogTests from './components/DialogTests.svelte';
  import SettingsTests from './components/SettingsTests.svelte';
  import SchedulerTests from './components/SchedulerTests.svelte';
  import AiTests from './components/AiTests.svelte';
  import OcrTests from './components/OcrTests.svelte';

  // 初始化全局 Logger 状态并通过 Context 广播
  const logger = new LoggerState();
  setContext('logger', logger);

  // 当前激活模块，默认 lifecycle
  let currentModule = $state('lifecycle');

  // 各模块注册参数
  const modules = [
    { id: 'lifecycle', name: 'Lifecycle', icon: '🔄', count: 2 },
    { id: 'window', name: 'Window', icon: '🪟', count: 4 },
    { id: 'command', name: 'Command', icon: '⌨️', count: 3 },
    { id: 'storage', name: 'Storage', icon: '💾', count: 9 },
    { id: 'notification', name: 'Notification', icon: '🔔', count: 1 },
    { id: 'toast', name: 'Toast', icon: '🍞', count: 5 },
    { id: 'clipboard', name: 'Clipboard', icon: '📋', count: 9 },
    { id: 'fs', name: 'File System', icon: '📁', count: 10 },
    { id: 'http', name: 'HTTP', icon: '🌐', count: 6 },
    { id: 'dialog', name: 'Dialog', icon: '💬', count: 8 },
    { id: 'settings', name: 'Settings', icon: '⚙️', count: 3 },
    { id: 'scheduler', name: 'Scheduler', icon: '⏰', count: 8 },
    { id: 'ai', name: 'AI', icon: '🤖', count: 10 },
    { id: 'ocr', name: 'OCR', icon: '🔍', count: 3 },
  ];

  // 注册全局 onLoad 监听器以演示跟随主应用启动
  lifecycle.onLoad(async () => {
    const startedAt = new Date().toLocaleString();
    const startupMessage = `plugin-demo 已触发 onLoad（${startedAt}）`;

    console.log(`[plugin-demo/startup] ${startupMessage}`);
    logger.log(`✓ ${startupMessage}`, 'success');

    try {
      await notification.show({
        title: 'Plugin Demo 已启动',
        body: startupMessage,
      });
      logger.log('✓ 已发送启动通知，可用于验证 run_at_startup', 'success');
    } catch (err: any) {
      logger.log(`✗ 启动通知发送失败: ${err?.message || err}`, 'error');
    }
  });
</script>

<div class="app">
  <!-- 侧边栏导航 -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h1>🚀 SDK Test</h1>
      <span class="version">v0.2.0</span>
    </div>
    <nav class="nav">
      {#each modules as m}
        <button
          class="nav-item"
          class:active={currentModule === m.id}
          onclick={() => (currentModule = m.id)}
        >
          <span class="icon">{m.icon}</span>
          <span class="name">{m.name}</span>
          <span class="badge">{m.count}</span>
        </button>
      {/each}
    </nav>
  </aside>

  <!-- 主界面区域 -->
  <main class="main">
    <header class="header">
      <h2>
        {modules.find((m) => m.id === currentModule)?.icon}
        {modules.find((m) => m.id === currentModule)?.name} API
      </h2>
      <div class="actions">
        <button class="btn btn-secondary" onclick={() => logger.clear()}>
          🗑️ 清空日志
        </button>
      </div>
    </header>

    <div class="content">
      <!-- 动态 API 测试面板区域 -->
      <section class="test-panel">
        {#if currentModule === 'lifecycle'}
          <LifecycleTests />
        {:else if currentModule === 'window'}
          <WindowTests />
        {:else if currentModule === 'command'}
          <CommandTests />
        {:else if currentModule === 'storage'}
          <StorageTests />
        {:else if currentModule === 'notification'}
          <NotificationTests />
        {:else if currentModule === 'toast'}
          <ToastTests />
        {:else if currentModule === 'clipboard'}
          <ClipboardTests />
        {:else if currentModule === 'fs'}
          <FsTests />
        {:else if currentModule === 'http'}
          <HttpTests />
        {:else if currentModule === 'dialog'}
          <DialogTests />
        {:else if currentModule === 'settings'}
          <SettingsTests />
        {:else if currentModule === 'scheduler'}
          <SchedulerTests />
        {:else if currentModule === 'ai'}
          <AiTests />
        {:else if currentModule === 'ocr'}
          <OcrTests />
        {/if}
      </section>

      <!-- 日志面板组件 -->
      <LogPanel />
    </div>
  </main>
</div>

<style>
  .app {
    display: flex;
    height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family:
      'Inter',
      -apple-system,
      BlinkMacSystemFont,
      sans-serif;
  }

  .sidebar {
    width: 220px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
  }

  .sidebar-header {
    padding: 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sidebar-header h1 {
    font-size: 18px;
    font-weight: 600;
    margin: 0;
  }

  .version {
    font-size: 11px;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .nav {
    padding: 8px;
    flex: 1;
    overflow-y: auto;
  }

  .nav-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--radius);
    cursor: pointer;
    transition: all 0.15s;
    font-size: 14px;
  }

  .nav-item:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--accent);
    color: white;
  }

  .nav-item .icon {
    font-size: 16px;
  }
  .nav-item .name {
    flex: 1;
    text-align: left;
  }
  .nav-item .badge {
    font-size: 11px;
    background: rgba(255, 255, 255, 0.15);
    padding: 2px 6px;
    border-radius: 10px;
  }

  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .header h2 {
    font-size: 18px;
    font-weight: 600;
    margin: 0;
  }

  .content {
    flex: 1;
    display: flex;
    gap: 16px;
    padding: 16px;
    overflow: hidden;
  }

  .test-panel {
    flex: 2;
    overflow-y: auto;
    padding-right: 8px;
  }
</style>
