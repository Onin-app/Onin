/**
 * Onin Plugin SDK 全面测试工具入口
 */
import './style.css';
import { mountPlugin } from 'onin-sdk';
import plugin from './plugin';

const target = document.getElementById('app');

if (!(target instanceof HTMLElement)) {
  throw new Error('Missing "#app" mount target.');
}

const cleanup = await mountPlugin(plugin, target);

export default cleanup;
