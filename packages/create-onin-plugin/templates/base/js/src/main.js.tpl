import { mountPluginUi } from "onin-sdk";
import { ui } from "./plugin";

const target = document.getElementById("app") ?? document.getElementById("root");

if (!(target instanceof HTMLElement)) {
  throw new Error('Missing "#app" or "#root" mount target.');
}

void mountPluginUi(ui, target);
