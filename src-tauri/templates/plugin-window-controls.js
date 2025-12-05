(function () {
  const debugInfo = document.getElementById('plugin-debug-info');
  function updateDebug(msg) {
    if (debugInfo) {
      debugInfo.textContent = msg;
      console.log('[Plugin Window Debug]', msg);
    }
  }

  // 如果 Tauri API 不存在，创建一个简化版本
  if (!window.__TAURI__) {
    updateDebug('创建 Tauri API 桥接...');

    // 使用 Tauri 的内部 IPC 机制
    window.__TAURI__ = {
      core: {
        invoke: async function (cmd, args) {
          return window.__TAURI_INTERNALS__.invoke(cmd, args);
        }
      },
      webviewWindow: {
        getCurrent: function () {
          const label = window.__TAURI_INTERNALS__.metadata.currentWindow.label;
          return {
            label: label,
            close: async function () {
              return window.__TAURI_INTERNALS__.invoke('plugin_close_window', { label });
            },
            minimize: async function () {
              return window.__TAURI_INTERNALS__.invoke('plugin_minimize_window', { label });
            },
            maximize: async function () {
              return window.__TAURI_INTERNALS__.invoke('plugin_maximize_window', { label });
            },
            unmaximize: async function () {
              return window.__TAURI_INTERNALS__.invoke('plugin_unmaximize_window', { label });
            },
            isMaximized: async function () {
              return window.__TAURI_INTERNALS__.invoke('plugin_is_maximized', { label });
            },
            show: async function () {
              return window.__TAURI_INTERNALS__.invoke('plugin_show_window', { label });
            },
            setFocus: async function () {
              return window.__TAURI_INTERNALS__.invoke('plugin_set_focus', { label });
            }
          };
        },
        WebviewWindow: {
          getByLabel: function (label) {
            return {
              label: label,
              show: async function () {
                return window.__TAURI_INTERNALS__.invoke('plugin_show_window', { label });
              },
              setFocus: async function () {
                return window.__TAURI_INTERNALS__.invoke('plugin_set_focus', { label });
              }
            };
          }
        }
      }
    };

    updateDebug('Tauri API 桥接已创建');
  }

  function initWindowControls() {
    updateDebug('检查 Tauri API...');

    if (!window.__TAURI__ || !window.__TAURI_INTERNALS__) {
      updateDebug('等待 Tauri 初始化...');
      console.warn('[Plugin Window] Tauri API not available yet, retrying...');
      setTimeout(initWindowControls, 100);
      return;
    }

    updateDebug('Tauri API 已加载，初始化控制按钮...');
    console.log('[Plugin Window] Initializing window controls');

    try {
      const { getCurrent } = window.__TAURI__.webviewWindow;
      const currentWindow = getCurrent();
      updateDebug('窗口对象已获取: ' + currentWindow.label);

      // 初始化拖动功能
      const topbar = document.getElementById('plugin-window-topbar');
      const topbarTitle = document.getElementById('plugin-window-topbar-title');

      if (topbar) {
        // 为整个顶栏添加拖动功能
        const initDrag = (element) => {
          element.addEventListener('mousedown', async (e) => {
            // 只在左键点击且不是在按钮上时触发拖动
            if (e.button === 0 && !e.target.closest('button')) {
              e.preventDefault();
              try {
                // 使用 Tauri 命令来启动拖动
                await window.__TAURI__.core.invoke('plugin_start_dragging');
                console.log('[Plugin Window] Dragging started');
              } catch (error) {
                console.error('[Plugin Window] Failed to start dragging:', error);
                updateDebug('拖动失败: ' + error.message);
              }
            }
          });
        };

        initDrag(topbar);
        if (topbarTitle) {
          initDrag(topbarTitle);
        }

        console.log('[Plugin Window] Drag event listeners attached');
        updateDebug('拖动监听器已添加 ✓');
      } else {
        console.warn('[Plugin Window] Topbar element not found');
        updateDebug('未找到顶栏元素');
      }

      // 切换到主窗口模式
      const backBtn = document.getElementById('plugin-back-to-inline-btn');
      if (backBtn) {
        backBtn.addEventListener('click', async () => {
          try {
            updateDebug('切换到主窗口...');
            const label = currentWindow.label;
            const pluginId = label.replace('plugin_', '').replace(/_/g, '.');

            console.log('[Plugin Window] Switching to inline mode:', pluginId);

            await window.__TAURI__.core.invoke('toggle_plugin_auto_detach', {
              pluginId: pluginId,
              autoDetach: false
            });

            await currentWindow.close();

            const { WebviewWindow } = window.__TAURI__.webviewWindow;
            const mainWindow = WebviewWindow.getByLabel('main');
            if (mainWindow) {
              await mainWindow.show();
              await mainWindow.setFocus();
            }
          } catch (error) {
            console.error('[Plugin Window] Failed to switch to inline mode:', error);
            updateDebug('切换失败: ' + error.message);
            alert('切换失败: ' + error);
          }
        });
        console.log('[Plugin Window] Back button initialized');
      }

      // 最小化
      const minimizeBtn = document.getElementById('plugin-minimize-btn');
      if (minimizeBtn) {
        minimizeBtn.addEventListener('click', async () => {
          try {
            updateDebug('最小化中...');
            console.log('[Plugin Window] Minimizing...');
            await currentWindow.minimize();
            updateDebug('已最小化');
          } catch (error) {
            console.error('[Plugin Window] Failed to minimize:', error);
            updateDebug('最小化失败: ' + error.message);
            alert('最小化失败: ' + error);
          }
        });
        console.log('[Plugin Window] Minimize button initialized');
      }

      // 最大化/还原
      const maximizeBtn = document.getElementById('plugin-maximize-btn');
      if (maximizeBtn) {
        maximizeBtn.addEventListener('click', async () => {
          try {
            updateDebug('切换最大化...');
            console.log('[Plugin Window] Toggling maximize...');
            const isMaximized = await currentWindow.isMaximized();
            if (isMaximized) {
              await currentWindow.unmaximize();
              updateDebug('已还原');
            } else {
              await currentWindow.maximize();
              updateDebug('已最大化');
            }
          } catch (error) {
            console.error('[Plugin Window] Failed to toggle maximize:', error);
            updateDebug('最大化失败: ' + error.message);
            alert('最大化失败: ' + error);
          }
        });
        console.log('[Plugin Window] Maximize button initialized');
      }

      // 关闭
      const closeBtn = document.getElementById('plugin-close-btn');
      if (closeBtn) {
        closeBtn.addEventListener('click', async () => {
          try {
            updateDebug('关闭中...');
            console.log('[Plugin Window] Closing...');
            await currentWindow.close();
          } catch (error) {
            console.error('[Plugin Window] Failed to close:', error);
            updateDebug('关闭失败: ' + error.message);
            alert('关闭失败: ' + error);
          }
        });
        console.log('[Plugin Window] Close button initialized');
      }

      // ESC 键最小化功能已通过 Rust 全局快捷键实现
      console.log('[Plugin Window] ESC key handled by global shortcut system');

      updateDebug('所有按钮已初始化 ✓');
      console.log('[Plugin Window] All controls initialized successfully');

      // 3秒后隐藏调试信息
      setTimeout(() => {
        if (debugInfo) debugInfo.style.display = 'none';
      }, 3000);

    } catch (error) {
      updateDebug('初始化失败: ' + error.message);
      console.error('[Plugin Window] Initialization error:', error);
    }
  }

  // 等待 DOM 加载完成
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initWindowControls);
  } else {
    initWindowControls();
  }
})();
