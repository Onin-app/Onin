/**
 * Onin Plugin SDK 全面测试工具入口
 */
import './style.css';
import { mountPluginUi } from 'onin-sdk';
import { ui } from './plugin';

const target = document.getElementById('app');

if (!(target instanceof HTMLElement)) {
  throw new Error('Missing "#app" mount target.');
}

const app = await mountPluginUi(ui, target);

export default app;
