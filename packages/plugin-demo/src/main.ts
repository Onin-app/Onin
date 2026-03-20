/**
 * Onin Plugin SDK 全面测试工具入口
 */
import './style.css';
import App from './App.svelte';
import { mount } from 'svelte';

const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;
