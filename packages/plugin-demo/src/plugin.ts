import { definePlugin, notification } from 'onin-sdk';

export const background = async () => {
  const startedAt = new Date().toLocaleString();
  const startupMessage = `plugin-demo 后台入口已执行（${startedAt}）`;

  console.log(`[plugin-demo/background] ${startupMessage}`);

  try {
    await notification.show({
      title: 'Plugin Demo 已启动',
      body: startupMessage,
    });
  } catch (err) {
    console.error('[plugin-demo/background] 启动通知发送失败:', err);
  }
};

export const ui = {
  mount: async ({ target }: { target: HTMLElement }) => {
    const { mountPluginUi } = await import('./ui');
    return mountPluginUi({ target });
  },
};

export default definePlugin({
  background,
  ui,
});
