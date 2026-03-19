import { mount, unmount } from "svelte";
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
  const app = mount(App, {
    target,
    props: {
      pluginName,
      pluginId,
    },
  });

  return () => {
    unmount(app);
  };
}
