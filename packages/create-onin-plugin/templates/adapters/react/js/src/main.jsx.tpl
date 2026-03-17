import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <App pluginName="__PLUGIN_NAME__" pluginId="__PLUGIN_ID__" />
  </React.StrictMode>,
);
