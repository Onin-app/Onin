import { createApp } from "vue";
import App from "./App.vue";

createApp(App, {
  pluginName: "__PLUGIN_NAME__",
  pluginId: "__PLUGIN_ID__",
}).mount("#app");
