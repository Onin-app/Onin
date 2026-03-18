import React from "react";
import ReactDOM from "react-dom/client";
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
}: MountPluginUiOptions): ReactDOM.Root {
  const root = ReactDOM.createRoot(target);
  root.render(
    <React.StrictMode>
      <App pluginName={pluginName} pluginId={pluginId} />
    </React.StrictMode>,
  );

  return root;
}
