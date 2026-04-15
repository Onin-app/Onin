<script lang="ts">
  /**
   * Onin Plugin SDK 全面 API 测试工具
   * 使用 Svelte 5 重构，每个 API 都有独立测试按钮
   */
  import {
    lifecycle,
    pluginWindow,
    command,
    storage,
    notification,
    toast,
    clipboard,
    fs,
    http,
    dialog,
    settings,
    scheduler,
    ai,
  } from 'onin-sdk';

  // 当前选中的测试模块
  let currentModule = $state('lifecycle');

  // 日志记录
  let logs = $state<
    { time: string; message: string; type: 'info' | 'success' | 'error' }[]
  >([]);

  // 日志函数（带毫秒精度）
  function log(message: string, type: 'info' | 'success' | 'error' = 'info') {
    const now = new Date();
    const h = now.getHours().toString().padStart(2, '0');
    const m = now.getMinutes().toString().padStart(2, '0');
    const s = now.getSeconds().toString().padStart(2, '0');
    const ms = now.getMilliseconds().toString().padStart(3, '0');
    const time = `${h}:${m}:${s}.${ms}`;
    logs = [...logs, { time, message, type }];
    setTimeout(() => {
      const logContainer = document.getElementById('log-container');
      if (logContainer) logContainer.scrollTop = logContainer.scrollHeight;
    }, 0);
  }

  function clearLogs() {
    logs = [];
  }

  // 执行异步测试
  async function runTest(name: string, fn: () => Promise<any>) {
    log(`⏳ ${name} 执行中...`);
    try {
      const result = await fn();
      const resultStr =
        result !== undefined ? JSON.stringify(result, null, 2) : '(void)';
      log(
        `✓ ${name} 成功: ${resultStr.substring(0, 200)}${resultStr.length > 200 ? '...' : ''}`,
        'success',
      );
    } catch (err: any) {
      log(`✗ ${name} 失败: ${err.message || err}`, 'error');
    }
  }

  // API 模块列表
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
  ];

  // 生命周期 demo
  lifecycle.onLoad(async () => {
    const startedAt = new Date().toLocaleString();
    const startupMessage = `plugin-demo 已触发 onLoad（${startedAt}）`;

    console.log(`[plugin-demo/startup] ${startupMessage}`);
    log(`✓ ${startupMessage}`, 'success');

    // 用系统通知辅助验证“跟随主程序启动”是否生效
    try {
      await notification.show({
        title: 'Plugin Demo 已启动',
        body: startupMessage,
      });
      log('✓ 已发送启动通知，可用于验证 run_at_startup', 'success');
    } catch (err: any) {
      log(`✗ 启动通知发送失败: ${err?.message || err}`, 'error');
    }
  });
</script>

<div class="app">
  <!-- 侧边栏 -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h1>🚀 SDK Test</h1>
      <span class="version">v0.1.0</span>
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

  <!-- 主内容区 -->
  <main class="main">
    <header class="header">
      <h2>
        {modules.find((m) => m.id === currentModule)?.icon}
        {modules.find((m) => m.id === currentModule)?.name} API
      </h2>
      <div class="actions">
        <button class="btn btn-secondary" onclick={clearLogs}
          >🗑️ 清空日志</button
        >
      </div>
    </header>

    <div class="content">
      <!-- 测试面板 -->
      <section class="test-panel">
        {#if currentModule === 'lifecycle'}
          {@render lifecycleTests()}
        {:else if currentModule === 'window'}
          {@render windowTests()}
        {:else if currentModule === 'command'}
          {@render commandTests()}
        {:else if currentModule === 'storage'}
          {@render storageTests()}
        {:else if currentModule === 'notification'}
          {@render notificationTests()}
        {:else if currentModule === 'toast'}
          {@render toastTests()}
        {:else if currentModule === 'clipboard'}
          {@render clipboardTests()}
        {:else if currentModule === 'fs'}
          {@render fsTests()}
        {:else if currentModule === 'http'}
          {@render httpTests()}
        {:else if currentModule === 'dialog'}
          {@render dialogTests()}
        {:else if currentModule === 'settings'}
          {@render settingsTests()}
        {:else if currentModule === 'scheduler'}
          {@render schedulerTests()}
        {:else if currentModule === 'ai'}
          {@render aiTests()}
        {/if}
      </section>

      <!-- 日志面板 -->
      <section class="log-panel">
        <div class="log-header">
          <h3>📝 日志输出</h3>
          <span class="log-count">{logs.length} 条</span>
        </div>
        <div id="log-container" class="log-container">
          {#each logs as entry}
            <div class="log-entry log-{entry.type}">
              <span class="log-time">[{entry.time}]</span>
              <span class="log-message">{entry.message}</span>
            </div>
          {/each}
          {#if logs.length === 0}
            <div class="log-empty">点击测试按钮查看输出...</div>
          {/if}
        </div>
      </section>
    </div>
  </main>
</div>

<!-- Lifecycle Tests -->
{#snippet lifecycleTests()}
  <div class="api-group">
    <h3>生命周期钩子</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() => {
          lifecycle.onLoad(() => log('onLoad 回调触发'));
          log('✓ onLoad 回调已注册', 'success');
        }}
      >
        <span class="api-name">onLoad</span>
        <span class="api-desc">注册加载回调</span>
      </button>
      <button
        class="test-btn"
        onclick={() => {
          lifecycle.onUnload(() => log('onUnload 回调触发'));
          log('✓ onUnload 回调已注册', 'success');
        }}
      >
        <span class="api-name">onUnload</span>
        <span class="api-desc">注册卸载回调</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- Window Tests -->
{#snippet windowTests()}
  <div class="api-group">
    <h3>窗口事件</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() => {
          pluginWindow.onShow(() => log('窗口显示了!'));
          log('✓ onShow 回调已注册', 'success');
        }}
      >
        <span class="api-name">onShow</span>
        <span class="api-desc">注册窗口显示回调</span>
      </button>
      <button
        class="test-btn"
        onclick={() => {
          pluginWindow.onHide(() => log('窗口隐藏了!'));
          log('✓ onHide 回调已注册', 'success');
        }}
      >
        <span class="api-name">onHide</span>
        <span class="api-desc">注册窗口隐藏回调</span>
      </button>
      <button
        class="test-btn"
        onclick={() => {
          pluginWindow.onFocus(() => log('窗口获得焦点!'));
          log('✓ onFocus 回调已注册', 'success');
        }}
      >
        <span class="api-name">onFocus</span>
        <span class="api-desc">注册焦点获得回调</span>
      </button>
      <button
        class="test-btn"
        onclick={() => {
          pluginWindow.onBlur(() => log('窗口失去焦点!'));
          log('✓ onBlur 回调已注册', 'success');
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
          log(`当前运行模式: ${mode}`, 'success');
        }}
      >
        <span class="api-name">getMode</span>
        <span class="api-desc">获取当前运行模式</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- Command Tests -->
{#snippet commandTests()}
  <div class="api-group">
    <h3>命令管理</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('command.register', () =>
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
          runTest('command.handle', async () => {
            await command.handle(async (code, args) => {
              log(`收到命令: ${code}, args: ${JSON.stringify(args)}`);
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
          runTest('command.remove', () => command.remove('test-cmd'))}
      >
        <span class="api-name">remove</span>
        <span class="api-desc">移除命令</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- Storage Tests -->
{#snippet storageTests()}
  <div class="api-group">
    <h3>持久化存储</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('storage.setItem', () =>
            storage.setItem('test-key', { value: 'Hello', time: Date.now() }),
          )}
      >
        <span class="api-name">setItem</span>
        <span class="api-desc">保存数据</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('storage.getItem', () => storage.getItem('test-key'))}
      >
        <span class="api-name">getItem</span>
        <span class="api-desc">读取数据</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('storage.removeItem', () => storage.removeItem('test-key'))}
      >
        <span class="api-name">removeItem</span>
        <span class="api-desc">删除数据</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('storage.keys', () => storage.keys())}
      >
        <span class="api-name">keys</span>
        <span class="api-desc">获取所有键</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('storage.setItems', () =>
            storage.setItems({ 'batch-1': 1, 'batch-2': 2, 'batch-3': 3 }),
          )}
      >
        <span class="api-name">setItems</span>
        <span class="api-desc">批量保存</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('storage.getItems', () =>
            storage.getItems(['batch-1', 'batch-2', 'batch-3']),
          )}
      >
        <span class="api-name">getItems</span>
        <span class="api-desc">批量读取</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('storage.getAll', () => storage.getAll())}
      >
        <span class="api-name">getAll</span>
        <span class="api-desc">获取全部</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('storage.setAll', () =>
            storage.setAll({ replaced: true, time: Date.now() }),
          )}
      >
        <span class="api-name">setAll</span>
        <span class="api-desc">替换全部</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('storage.clear', () => storage.clear())}
      >
        <span class="api-name">clear</span>
        <span class="api-desc">清空所有</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- Notification Tests -->
{#snippet notificationTests()}
  <div class="api-group">
    <h3>系统通知</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('notification.isPermissionGranted', () =>
            notification.isPermissionGranted(),
          )}
      >
        <span class="api-name">isPermissionGranted</span>
        <span class="api-desc">检查通知权限</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('notification.requestPermission', () =>
            notification.requestPermission(),
          )}
      >
        <span class="api-name">requestPermission</span>
        <span class="api-desc">申请通知权限</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('notification.show (Sound)', () =>
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
          runTest('notification.show', () =>
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
{/snippet}

<!-- Clipboard Tests -->
{#snippet clipboardTests()}
  <div class="api-group">
    <h3>剪贴板操作</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() => runTest('clipboard.hasText', () => clipboard.hasText())}
      >
        <span class="api-name">hasText</span>
        <span class="api-desc">检查文本</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('clipboard.hasImage', () => clipboard.hasImage())}
      >
        <span class="api-name">hasImage</span>
        <span class="api-desc">检查图片</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('clipboard.readText', () => clipboard.readText())}
      >
        <span class="api-name">readText</span>
        <span class="api-desc">读取文本</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('clipboard.writeText', () =>
            clipboard.writeText('Hello from SDK Test! 你好！'),
          )}
      >
        <span class="api-name">writeText</span>
        <span class="api-desc">写入文本</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('clipboard.readImage', () => clipboard.readImage())}
      >
        <span class="api-name">readImage</span>
        <span class="api-desc">读取图片</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('clipboard.writeImage', () =>
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
          runTest('clipboard.copy', () => clipboard.copy('使用 copy() 方法'))}
      >
        <span class="api-name">copy</span>
        <span class="api-desc">复制 (别名)</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('clipboard.paste', () => clipboard.paste())}
      >
        <span class="api-name">paste</span>
        <span class="api-desc">粘贴 (别名)</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('clipboard.clear', () => clipboard.clear())}
      >
        <span class="api-name">clear</span>
        <span class="api-desc">清空剪贴板</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- File System Tests -->
{#snippet fsTests()}
  <div class="api-group">
    <h3>文件系统</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('fs.writeFile', () =>
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
        onclick={() => runTest('fs.readFile', () => fs.readFile('test.txt'))}
      >
        <span class="api-name">readFile</span>
        <span class="api-desc">读取文件</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('fs.exists', () => fs.exists('test.txt'))}
      >
        <span class="api-name">exists</span>
        <span class="api-desc">检查存在</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('fs.createDir', () => fs.createDir('test-folder', true))}
      >
        <span class="api-name">createDir</span>
        <span class="api-desc">创建目录</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('fs.listDir', () => fs.listDir('.'))}
      >
        <span class="api-name">listDir</span>
        <span class="api-desc">列出目录</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('fs.getFileInfo', () => fs.getFileInfo('test.txt'))}
      >
        <span class="api-name">getFileInfo</span>
        <span class="api-desc">获取信息</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('fs.copyFile', () =>
            fs.copyFile('test.txt', 'test-copy.txt'),
          )}
      >
        <span class="api-name">copyFile</span>
        <span class="api-desc">复制文件</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('fs.moveFile', () =>
            fs.moveFile('test-copy.txt', 'test-moved.txt'),
          )}
      >
        <span class="api-name">moveFile</span>
        <span class="api-desc">移动文件</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('fs.deleteFile', () => fs.deleteFile('test-moved.txt'))}
      >
        <span class="api-name">deleteFile</span>
        <span class="api-desc">删除文件</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('fs.deleteDir', () => fs.deleteDir('test-folder', true))}
      >
        <span class="api-name">deleteDir</span>
        <span class="api-desc">删除目录</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- HTTP Tests -->
{#snippet httpTests()}
  <div class="api-group">
    <h3>HTTP 请求</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('http.get', () => http.get('https://httpbin.org/get'))}
      >
        <span class="api-name">get</span>
        <span class="api-desc">GET 请求</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('http.post', () =>
            http.post('https://httpbin.org/post', {
              test: 'data',
              time: Date.now(),
            }),
          )}
      >
        <span class="api-name">post</span>
        <span class="api-desc">POST 请求</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('http.put', () =>
            http.put('https://httpbin.org/put', { updated: true }),
          )}
      >
        <span class="api-name">put</span>
        <span class="api-desc">PUT 请求</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('http.patch', () =>
            http.patch('https://httpbin.org/patch', { patched: true }),
          )}
      >
        <span class="api-name">patch</span>
        <span class="api-desc">PATCH 请求</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('http.delete', () =>
            http.delete('https://httpbin.org/delete'),
          )}
      >
        <span class="api-name">delete</span>
        <span class="api-desc">DELETE 请求</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('http.request', () =>
            http.request({
              url: 'https://httpbin.org/anything',
              method: 'POST',
              headers: { 'X-Custom': 'test' },
              body: { custom: true },
            }),
          )}
      >
        <span class="api-name">request</span>
        <span class="api-desc">通用请求</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- Dialog Tests -->
{#snippet dialogTests()}
  <div class="api-group">
    <h3>对话框</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('dialog.showMessage', () =>
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
          runTest('dialog.showConfirm', () =>
            dialog.showConfirm({ title: '确认', message: '是否继续操作？' }),
          )}
      >
        <span class="api-name">showConfirm</span>
        <span class="api-desc">确认框</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('dialog.showOpen', () =>
            dialog.showOpen({ title: '选择文件', multiple: false }),
          )}
      >
        <span class="api-name">showOpen</span>
        <span class="api-desc">打开文件</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('dialog.showSave', () =>
            dialog.showSave({ title: '保存文件', defaultPath: 'test.txt' }),
          )}
      >
        <span class="api-name">showSave</span>
        <span class="api-desc">保存文件</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('dialog.info', () => dialog.info('这是一条信息', '信息'))}
      >
        <span class="api-name">info</span>
        <span class="api-desc">信息提示</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('dialog.warning', () =>
            dialog.warning('这是一条警告', '警告'),
          )}
      >
        <span class="api-name">warning</span>
        <span class="api-desc">警告提示</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('dialog.error', () => dialog.error('这是一条错误', '错误'))}
      >
        <span class="api-name">error</span>
        <span class="api-desc">错误提示</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('dialog.confirm', () =>
            dialog.confirm('确定要执行此操作吗？', '确认'),
          )}
      >
        <span class="api-name">confirm</span>
        <span class="api-desc">确认 (简化)</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- Settings Tests -->
{#snippet settingsTests()}
  <div class="api-group">
    <h3>插件设置</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('settings.useSettingsSchema', () =>
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
          runTest('settings.getSchema', async () => settings.getSchema())}
      >
        <span class="api-name">getSchema</span>
        <span class="api-desc">获取定义</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('settings.getAll', () => settings.getAll())}
      >
        <span class="api-name">getAll</span>
        <span class="api-desc">获取所有值</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('settings.onChange', async () => {
             const unlisten = await settings.onChange((newSettings) => {
               log(`⚙️ 设置已更改: ${JSON.stringify(newSettings)}`, 'info');
             });
             log('✓ 已添加设置更改侦听器', 'success');
             return '监听中...';
          })
        }
      >
        <span class="api-name">onChange</span>
        <span class="api-desc">监听设置更改</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- Scheduler Tests -->
{#snippet schedulerTests()}
  <div class="api-group">
    <h3>定时任务</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('scheduler.schedule', () =>
            scheduler.schedule('test-cron', '* * * * *', () =>
              log('cron 任务执行'),
            ),
          )}
      >
        <span class="api-name">schedule</span>
        <span class="api-desc">注册任务 (cron)</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('scheduler.daily', () =>
            scheduler.daily('test-daily', '08:00', () => log('每日任务执行')),
          )}
      >
        <span class="api-name">daily</span>
        <span class="api-desc">每日任务</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('scheduler.hourly', () =>
            scheduler.hourly('test-hourly', 30, () => log('每小时任务执行')),
          )}
      >
        <span class="api-name">hourly</span>
        <span class="api-desc">每小时</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('scheduler.weekly', () =>
            scheduler.weekly('test-weekly', 1, '09:00', () =>
              log('每周任务执行'),
            ),
          )}
      >
        <span class="api-name">weekly</span>
        <span class="api-desc">每周任务</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('scheduler.list', () => scheduler.list())}
      >
        <span class="api-name">list</span>
        <span class="api-desc">列出任务</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('scheduler.timeout', () =>
            scheduler.timeout('test-timeout', 5000, () =>
              log('✅ 5秒 Timeout 任务执行完成', 'success'),
            ),
          )}
      >
        <span class="api-name">timeout</span>
        <span class="api-desc">5秒后执行</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('scheduler.at', () => {
            const time = Date.now() + 10000;
            return scheduler.at('test-at', time, () =>
              log('✅ 10秒 At 任务执行完成', 'success'),
            );
          })}
      >
        <span class="api-name">at</span>
        <span class="api-desc">10秒后执行</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('scheduler.cancel', () => scheduler.cancel('test-cron'))}
      >
        <span class="api-name">cancel</span>
        <span class="api-desc">取消任务</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- AI Tests -->
{#snippet aiTests()}
  <div class="api-group">
    <h3>基础检测</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() => runTest('ai.isAvailable', () => ai.isAvailable())}
      >
        <span class="api-name">isAvailable</span>
        <span class="api-desc">检查 AI 是否可用</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('ai.getCapabilities', () => ai.getCapabilities())}
      >
        <span class="api-name">getCapabilities</span>
        <span class="api-desc">获取 AI 能力</span>
      </button>
      <button
        class="test-btn"
        onclick={() => runTest('ai.listModels', () => ai.listModels())}
      >
        <span class="api-name">listModels</span>
        <span class="api-desc">列出可用模型</span>
      </button>
    </div>
  </div>

  <div class="api-group">
    <h3>简单问答</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('ai.ask (简单)', () => ai.ask('用一句话介绍 TypeScript'))}
      >
        <span class="api-name">ask</span>
        <span class="api-desc">简单问答</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('ai.ask (带选项)', () =>
            ai.ask('写一首关于代码的俳句', {
              temperature: 0.9,
              max_tokens: 100,
            }),
          )}
      >
        <span class="api-name">ask + options</span>
        <span class="api-desc">带参数问答</span>
      </button>
    </div>
  </div>

  <div class="api-group">
    <h3>流式响应</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={async () => {
          log('⏳ 开始流式响应测试...');
          let fullText = '';
          try {
            await ai.stream('用三句话介绍 Rust 编程语言', (chunk) => {
              fullText += chunk;
              log(`📨 收到 chunk: ${chunk}`, 'info');
            });
            log(`✓ 流式响应完成，总长度: ${fullText.length}`, 'success');
          } catch (err: any) {
            log(`✗ 流式响应失败: ${err.message}`, 'error');
          }
        }}
      >
        <span class="api-name">stream</span>
        <span class="api-desc">流式问答</span>
      </button>
    </div>
  </div>

  <div class="api-group">
    <h3>多轮对话</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={async () => {
          log('⏳ 创建对话管理器...');
          try {
            const conv = ai.createConversation(
              '你是一个友好的编程助手，用简洁的语言回答问题。',
            );
            log('✓ 对话管理器已创建', 'success');

            log('⏳ 第一轮对话...');
            const resp1 = await conv.ask('什么是闭包？');
            log(`✓ AI: ${resp1.substring(0, 100)}...`, 'success');

            log('⏳ 第二轮对话...');
            const resp2 = await conv.ask('能举个例子吗？');
            log(`✓ AI: ${resp2.substring(0, 100)}...`, 'success');

            const history = conv.getHistory();
            log(`✓ 对话历史共 ${history.length} 条消息`, 'success');
          } catch (err: any) {
            log(`✗ 多轮对话失败: ${err.message}`, 'error');
          }
        }}
      >
        <span class="api-name">createConversation</span>
        <span class="api-desc">多轮对话</span>
      </button>
      <button
        class="test-btn"
        onclick={async () => {
          log('⏳ 测试对话流式响应...');
          try {
            const conv = ai.createConversation('你是一个诗人。');
            let fullText = '';
            await conv.stream('写一首关于月亮的诗', (chunk) => {
              fullText += chunk;
            });
            log(`✓ 流式对话完成: ${fullText.substring(0, 100)}...`, 'success');
          } catch (err: any) {
            log(`✗ 流式对话失败: ${err.message}`, 'error');
          }
        }}
      >
        <span class="api-name">conversation.stream</span>
        <span class="api-desc">对话流式</span>
      </button>
    </div>
  </div>

  <div class="api-group">
    <h3>消息构建</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={async () => {
          log('⏳ 测试文本消息构建...');
          try {
            // 直接构建文本消息对象
            const msg = {
              role: 'user' as const,
              content: [{ type: 'text' as const, text: '这是一条测试消息' }],
            };
            log(`✓ 文本消息: ${JSON.stringify(msg)}`, 'success');
          } catch (err: any) {
            log(`✗ 构建失败: ${err.message}`, 'error');
          }
        }}
      >
        <span class="api-name">createTextMessage</span>
        <span class="api-desc">创建文本消息</span>
      </button>
      <button
        class="test-btn"
        onclick={async () => {
          log('⏳ 测试图片消息构建...');
          try {
            // 直接构建图片消息对象
            const msg = {
              role: 'user' as const,
              content: [
                { type: 'text' as const, text: '描述这张图片' },
                {
                  type: 'image_url' as const,
                  image_url: { url: 'https://picsum.photos/200' },
                },
                {
                  type: 'image_url' as const,
                  image_url: {
                    url: 'https://picsum.photos/300',
                    detail: 'high' as const,
                  },
                },
              ],
            };
            log(`✓ 图片消息包含 ${msg.content.length} 个内容项`, 'success');
          } catch (err: any) {
            log(`✗ 构建失败: ${err.message}`, 'error');
          }
        }}
      >
        <span class="api-name">createImageMessage</span>
        <span class="api-desc">创建图片消息</span>
      </button>
      <button
        class="test-btn"
        onclick={async () => {
          log('⏳ 测试图片识别 (需要支持 vision 的模型)...');
          try {
            // 直接构建包含图片的消息
            const msg = {
              role: 'user' as const,
              content: [
                { type: 'text' as const, text: '这张图片里有什么？' },
                {
                  type: 'image_url' as const,
                  image_url: { url: 'https://picsum.photos/400/300' },
                },
              ],
            };
            const response = await ai.ask([msg]);
            log(`✓ AI 识别结果: ${response.substring(0, 150)}...`, 'success');
          } catch (err: any) {
            log(`✗ 图片识别失败: ${err.message}`, 'error');
          }
        }}
      >
        <span class="api-name">Vision API</span>
        <span class="api-desc">图片识别</span>
      </button>
    </div>
  </div>
{/snippet}

<!-- Toast Tests -->
{#snippet toastTests()}
  <div class="api-group">
    <h3>Toast 提示</h3>
    <div class="test-grid">
      <button
        class="test-btn"
        onclick={() =>
          runTest('toast.show', () => toast.show('这是一条通用提示'))}
      >
        <span class="api-name">show</span>
        <span class="api-desc">显示通用提示</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('toast.success', () =>
            toast.success('操作成功！', { duration: 3000 }),
          )}
      >
        <span class="api-name">success</span>
        <span class="api-desc">显示成功提示 (3s)</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('toast.error', () => toast.error('发生错误，请重试'))}
      >
        <span class="api-name">error</span>
        <span class="api-desc">显示错误提示</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('toast.warning', () =>
            toast.warning('这是一条警告信息', { duration: 5000 }),
          )}
      >
        <span class="api-name">warning</span>
        <span class="api-desc">显示警告提示 (5s)</span>
      </button>
      <button
        class="test-btn"
        onclick={() =>
          runTest('toast.info', () => toast.info('这是一条普通信息'))}
      >
        <span class="api-name">info</span>
        <span class="api-desc">显示信息提示</span>
      </button>
    </div>
  </div>
{/snippet}

<style>
  :root {
    --bg-primary: #0f0f0f;
    --bg-secondary: #1a1a1a;
    --bg-tertiary: #252525;
    --text-primary: #ffffff;
    --text-secondary: #a0a0a0;
    --text-muted: #666666;
    --accent: #6366f1;
    --accent-hover: #818cf8;
    --success: #22c55e;
    --error: #ef4444;
    --border: #333333;
    --radius: 8px;
  }

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

  .btn {
    padding: 8px 16px;
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s;
  }

  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn-secondary:hover {
    background: var(--border);
  }

  .content {
    flex: 1;
    display: flex;
    gap: 16px;
    padding: 16px;
    overflow: hidden;
  }

  .test-panel {
    flex: 1;
    overflow-y: auto;
    padding-right: 8px;
  }

  .api-group {
    margin-bottom: 24px;
  }
  .api-group h3 {
    font-size: 14px;
    color: var(--text-secondary);
    margin: 0 0 12px 0;
    font-weight: 500;
  }

  .test-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 8px;
  }

  .test-btn {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    cursor: pointer;
    transition: all 0.15s;
  }

  .test-btn:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent);
  }

  .test-btn:active {
    transform: scale(0.98);
  }

  .test-btn .api-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    font-family: 'JetBrains Mono', monospace;
  }

  .test-btn .api-desc {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 4px;
  }

  .log-panel {
    width: 380px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
  }

  .log-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .log-header h3 {
    font-size: 14px;
    margin: 0;
    font-weight: 500;
  }
  .log-count {
    font-size: 11px;
    color: var(--text-muted);
  }

  .log-container {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
  }

  .log-entry {
    padding: 4px 0;
    display: flex;
    gap: 8px;
  }
  .log-time {
    color: var(--text-muted);
    flex-shrink: 0;
  }
  .log-message {
    word-break: break-all;
  }
  .log-info .log-message {
    color: var(--text-secondary);
  }
  .log-success .log-message {
    color: var(--success);
  }
  .log-error .log-message {
    color: var(--error);
  }
  .log-empty {
    color: var(--text-muted);
    text-align: center;
    padding: 32px;
  }

  ::-webkit-scrollbar {
    width: 6px;
  }
  ::-webkit-scrollbar-track {
    background: transparent;
  }
  ::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
  }
  ::-webkit-scrollbar-thumb:hover {
    background: var(--text-muted);
  }
</style>
