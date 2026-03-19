import { definePlugin, notification } from 'onin-sdk';

export const setup = async () => {
  const startedAt = new Date().toLocaleString();
  const startupMessage = `plugin-demo setup 已执行（${startedAt}）`;

  console.log(`[plugin-demo/setup] ${startupMessage}`);

  try {
    await notification.show({
      title: 'Plugin Demo setup 已执行',
      body: startupMessage,
    });
  } catch (err) {
    console.error('[plugin-demo/setup] 启动通知发送失败:', err);
  }
};

export const mount = async ({ target }: { target: HTMLElement }) => {
  const { mountPluginUi } = await import('./ui');
  return mountPluginUi({ target });
};

export default definePlugin({
  setup,
  mount,
});
