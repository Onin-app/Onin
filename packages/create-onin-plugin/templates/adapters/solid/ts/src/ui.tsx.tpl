import { render } from "solid-js/web";
import App from "./App";

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
  return render(() => <App pluginName={pluginName} pluginId={pluginId} />, target);
}
