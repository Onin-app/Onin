import { createApp } from "vue";
import App from "./App.vue";

export function mountPluginUi({ target, pluginName, pluginId }) {
  const app = createApp(App, { pluginName, pluginId });
  app.mount(target);
  return app;
}
