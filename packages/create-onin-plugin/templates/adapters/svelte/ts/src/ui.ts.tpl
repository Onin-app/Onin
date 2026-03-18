import { mount } from "svelte";
import App from "./App.svelte";

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
  return mount(App, {
    target,
    props: {
      pluginName,
      pluginId,
    },
  });
}
