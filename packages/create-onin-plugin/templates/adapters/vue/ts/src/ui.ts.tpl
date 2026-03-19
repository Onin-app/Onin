import { createApp } from "vue";
import App from "./App.vue";

interface MountPluginUiOptions {
  target: HTMLElement;
  pluginName: string;
  pluginId: string;
}

export function mountPluginUi({
  target,
  pluginName,
  pluginId,
}: MountPluginUiOptions) {
  const app = createApp(App, { pluginName, pluginId });
  app.mount(target);
  return () => {
    app.unmount();
  };
}
