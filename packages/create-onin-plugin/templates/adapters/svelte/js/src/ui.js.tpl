import { mount, unmount } from "svelte";
import App from "./App.svelte";

export function mountPluginUi({ target, pluginName, pluginId }) {
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
