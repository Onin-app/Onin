import { mount } from "svelte";
import App from "./App.svelte";

export function mountPluginUi({ target, pluginName, pluginId }) {
  return mount(App, {
    target,
    props: {
      pluginName,
      pluginId,
    },
  });
}
