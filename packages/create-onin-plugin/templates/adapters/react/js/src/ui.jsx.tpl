import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

export function mountPluginUi({ target, pluginName, pluginId }) {
  const root = ReactDOM.createRoot(target);
  root.render(
    <React.StrictMode>
      <App pluginName={pluginName} pluginId={pluginId} />
    </React.StrictMode>,
  );

  return root;
}
