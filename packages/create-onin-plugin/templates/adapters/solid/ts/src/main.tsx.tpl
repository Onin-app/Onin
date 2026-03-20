import { render } from "solid-js/web";
import App from "./App";

render(
  () => <App pluginName="__PLUGIN_NAME__" pluginId="__PLUGIN_ID__" />,
  document.getElementById("app")!,
);
