import { mountPlugin } from "onin-sdk";
import plugin from "./plugin";

const target = document.getElementById("app") ?? document.getElementById("root");

if (!(target instanceof HTMLElement)) {
  throw new Error('Missing "#app" or "#root" mount target.');
}

const cleanup = await mountPlugin(plugin, target);

export default cleanup;
