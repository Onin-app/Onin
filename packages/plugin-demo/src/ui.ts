import {
  ai,
  clipboard,
  command,
  dialog,
  fs,
  http,
  lifecycle,
  notification,
  pluginWindow,
  scheduler,
  settings,
  storage,
} from 'onin-sdk';

type LogType = 'info' | 'success' | 'error';
type ModuleId =
  | 'lifecycle'
  | 'window'
  | 'command'
  | 'storage'
  | 'settings'
  | 'io'
  | 'scheduler'
  | 'ai';

type DemoAction = {
  name: string;
  description: string;
  run: () => Promise<unknown> | unknown;
};

const modules: { id: ModuleId; name: string; icon: string }[] = [
  { id: 'lifecycle', name: 'Lifecycle', icon: '🔄' },
  { id: 'window', name: 'Window', icon: '🪟' },
  { id: 'command', name: 'Command', icon: '⌨️' },
  { id: 'storage', name: 'Storage', icon: '💾' },
  { id: 'settings', name: 'Settings', icon: '⚙️' },
  { id: 'io', name: 'I/O', icon: '📦' },
  { id: 'scheduler', name: 'Scheduler', icon: '⏰' },
  { id: 'ai', name: 'AI', icon: '🤖' },
];

const actionsByModule: Record<ModuleId, DemoAction[]> = {
  lifecycle: [
    {
      name: 'onLoad',
      description: '注册后台加载回调',
      run: () => lifecycle.onLoad(() => appendLog('onLoad 回调触发', 'success')),
    },
    {
      name: 'onUnload',
      description: '注册后台卸载回调',
      run: () => lifecycle.onUnload(() => appendLog('onUnload 回调触发', 'success')),
    },
    {
      name: 'show notice',
      description: '发送系统通知验证宿主通路',
      run: () =>
        notification.show({
          title: 'Plugin Demo',
          body: '后台入口和 SDK 通知链路正常。',
        }),
    },
  ],
  window: [
    {
      name: 'getMode',
      description: '查看当前窗口模式',
      run: () => pluginWindow.getMode(),
    },
    {
      name: 'onShow',
      description: '注册显示回调',
      run: () => pluginWindow.onShow(() => appendLog('窗口显示事件已触发', 'success')),
    },
    {
      name: 'onHide',
      description: '注册隐藏回调',
      run: () => pluginWindow.onHide(() => appendLog('窗口隐藏事件已触发', 'success')),
    },
  ],
  command: [
    {
      name: 'register',
      description: '注册动态命令',
      run: () =>
        command.register({
          code: 'demo-runtime-command',
          name: 'Runtime Demo Command',
          description: 'Registered from plugin-demo UI',
          keywords: [{ name: 'runtime-demo', type: 'prefix' }],
        }),
    },
    {
      name: 'handle',
      description: '注册命令处理器',
      run: () =>
        command.handle(async (code, args) => {
          appendLog(`收到命令 ${code}: ${JSON.stringify(args)}`);
          return { handled: true, code };
        }),
    },
    {
      name: 'remove',
      description: '移除动态命令',
      run: () => command.remove('demo-runtime-command'),
    },
  ],
  storage: [
    {
      name: 'setItem',
      description: '保存示例数据',
      run: () =>
        storage.setItem('plugin-demo', {
          updatedAt: new Date().toISOString(),
          enabled: true,
        }),
    },
    {
      name: 'getItem',
      description: '读取示例数据',
      run: () => storage.getItem('plugin-demo'),
    },
    {
      name: 'getAll',
      description: '读取全部存储',
      run: () => storage.getAll(),
    },
    {
      name: 'clear',
      description: '清空插件存储',
      run: () => storage.clear(),
    },
  ],
  settings: [
    {
      name: 'useSettingsSchema',
      description: '注册设置面板 schema',
      run: () =>
        settings.useSettingsSchema([
          {
            key: 'accent',
            type: 'color',
            label: 'Accent',
            defaultValue: '#6366f1',
          },
          {
            key: 'enabled',
            type: 'switch',
            label: 'Enabled',
            defaultValue: true,
          },
        ]),
    },
    {
      name: 'getSchema',
      description: '读取当前 schema',
      run: () => settings.getSchema(),
    },
    {
      name: 'getAll',
      description: '读取当前设置值',
      run: () => settings.getAll(),
    },
  ],
  io: [
    {
      name: 'clipboard.writeText',
      description: '写入剪贴板文本',
      run: () => clipboard.writeText('Hello from plugin-demo'),
    },
    {
      name: 'fs.writeFile',
      description: '写入示例文件',
      run: () => fs.writeFile('plugin-demo.txt', `Generated at ${new Date().toISOString()}`),
    },
    {
      name: 'http.get',
      description: '发起 GET 请求',
      run: () => http.get('https://httpbin.org/get'),
    },
    {
      name: 'dialog.confirm',
      description: '弹出确认框',
      run: () => dialog.confirm('Plugin demo dialog test', 'Confirm'),
    },
  ],
  scheduler: [
    {
      name: 'list',
      description: '列出当前任务',
      run: () => scheduler.list(),
    },
    {
      name: 'timeout',
      description: '注册 3 秒后触发的任务',
      run: () =>
        scheduler.timeout('plugin-demo-timeout', 3000, () =>
          appendLog('timeout 任务执行完成', 'success'),
        ),
    },
    {
      name: 'cancel',
      description: '取消 timeout 任务',
      run: () => scheduler.cancel('plugin-demo-timeout'),
    },
  ],
  ai: [
    {
      name: 'isAvailable',
      description: '检查 AI 能力',
      run: () => ai.isAvailable(),
    },
    {
      name: 'listModels',
      description: '列出可用模型',
      run: () => ai.listModels(),
    },
    {
      name: 'ask',
      description: '发起一个简单问答',
      run: () => ai.ask('用一句话介绍 Onin plugin SDK'),
    },
  ],
};

let currentModule: ModuleId = 'lifecycle';
let logEntries: { time: string; message: string; type: LogType }[] = [
  {
    time: timeLabel(),
    message: '点击左侧任一动作开始验证 SDK 通路。',
    type: 'info',
  },
];
let actionCountEl: HTMLElement | null = null;
let logCountEl: HTMLElement | null = null;
let logContainerEl: HTMLElement | null = null;

function timeLabel() {
  return new Date().toLocaleTimeString('zh-CN', { hour12: false });
}

function escapeHtml(value: string) {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function appendLog(message: string, type: LogType = 'info') {
  logEntries = [...logEntries, { time: timeLabel(), message, type }];
  renderLogs();
}

function renderLogs() {
  if (!logContainerEl || !logCountEl) {
    return;
  }

  logContainerEl.innerHTML = logEntries
    .map(
      (entry) => `
        <div class="log-entry log-${entry.type}">
          <span class="log-time">[${entry.time}]</span>
          <span class="log-message">${escapeHtml(entry.message)}</span>
        </div>
      `,
    )
    .join('');

  logCountEl.textContent = `${logEntries.length} entries`;
  logContainerEl.scrollTop = logContainerEl.scrollHeight;
}

function renderActions(actionGridEl: HTMLElement) {
  const actions = actionsByModule[currentModule];
  actionGridEl.innerHTML = '';
  if (actionCountEl) {
    actionCountEl.textContent = `${actions.length} 个动作`;
  }

  for (const action of actions) {
    const button = document.createElement('button');
    button.className = 'action-card';
    button.innerHTML = `
      <span class="action-name">${escapeHtml(action.name)}</span>
      <span class="action-desc">${escapeHtml(action.description)}</span>
    `;
    button.onclick = async () => {
      appendLog(`⏳ ${action.name} 执行中...`);
      try {
        const result = await action.run();
        const output =
          result === undefined ? '(void)' : JSON.stringify(result, null, 2);
        appendLog(
          `✓ ${action.name} 成功: ${output.slice(0, 220)}${output.length > 220 ? '...' : ''}`,
          'success',
        );
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        appendLog(`✗ ${action.name} 失败: ${message}`, 'error');
      }
    };
    actionGridEl.append(button);
  }
}

export function mountPluginUi({ target }: { target: HTMLElement }) {
  target.innerHTML = `
    <div class="app">
      <aside class="sidebar">
        <div class="sidebar-header">
          <p class="eyebrow">Plugin Demo</p>
          <h1>Onin SDK</h1>
          <p class="sidebar-copy">单源码声明，双产物输出。这个 demo 既验证后台入口，也验证 UI 交互 API。</p>
        </div>
        <nav class="nav"></nav>
      </aside>
      <main class="main">
        <header class="header">
          <div>
            <p class="eyebrow">Current Module</p>
            <h2 id="module-title"></h2>
          </div>
          <button class="clear-btn" id="clear-logs">清空日志</button>
        </header>
        <div class="content">
          <section class="panel action-panel">
            <div class="panel-head">
              <h3>Interactive Checks</h3>
              <p id="action-count"></p>
            </div>
            <div class="grid" id="action-grid"></div>
          </section>
          <section class="panel log-panel">
            <div class="panel-head">
              <h3>Runtime Log</h3>
              <p id="log-count"></p>
            </div>
            <div class="log-container" id="log-container"></div>
          </section>
        </div>
      </main>
    </div>
  `;

  const navEl = target.querySelector('.nav');
  const actionGridEl = target.querySelector('#action-grid');
  const moduleTitleEl = target.querySelector('#module-title');
  const clearButtonEl = target.querySelector('#clear-logs');
  actionCountEl = target.querySelector('#action-count');
  logCountEl = target.querySelector('#log-count');
  logContainerEl = target.querySelector('#log-container');

  if (
    !(navEl instanceof HTMLElement) ||
    !(actionGridEl instanceof HTMLElement) ||
    !(moduleTitleEl instanceof HTMLElement) ||
    !(clearButtonEl instanceof HTMLButtonElement) ||
    !(actionCountEl instanceof HTMLElement) ||
    !(logCountEl instanceof HTMLElement) ||
    !(logContainerEl instanceof HTMLElement)
  ) {
    throw new Error('Failed to initialize plugin-demo UI.');
  }

  for (const module of modules) {
    const button = document.createElement('button');
    button.className = 'nav-item';
    if (module.id === currentModule) {
      button.classList.add('active');
    }
    button.innerHTML = `<span class="icon">${module.icon}</span><span class="name">${module.name}</span>`;
    button.onclick = () => {
      currentModule = module.id;
      moduleTitleEl.textContent = `${module.icon} ${module.name}`;
      for (const navButton of navEl.querySelectorAll('.nav-item')) {
        navButton.classList.remove('active');
      }
      button.classList.add('active');
      renderActions(actionGridEl);
    };
    navEl.append(button);
  }

  clearButtonEl.onclick = () => {
    logEntries = [{ time: timeLabel(), message: '日志已清空。', type: 'info' }];
    renderLogs();
  };

  moduleTitleEl.textContent = `${modules[0].icon} ${modules[0].name}`;
  renderActions(actionGridEl);
  renderLogs();
}
