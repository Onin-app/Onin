import { render } from "solid-js/web";
import App from "./App";

export function mountPluginUi({ target, pluginName, pluginId }) {
  return render(() => <App pluginName={pluginName} pluginId={pluginId} />, target);
}
