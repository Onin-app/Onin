import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, {
  target: document.getElementById("app")!,
  props: {
    pluginName: "__PLUGIN_NAME__",
    pluginId: "__PLUGIN_ID__",
  },
});

export default app;
