/**
 * plugin-window-fab.js
 *
 * 插件独立窗口的悬浮操作菜单（FAB）
 *
 * 在编译时通过 include_str! 宏嵌入到 Rust 初始化脚本中，
 * 运行时变量（pluginId、version）从 window.__ONIN_RUNTIME__ 读取，
 * 无需 Rust format! 占位符，文件可独立编辑与格式化。
 */
(function () {
  window.addEventListener("DOMContentLoaded", function () {
    // ── 注入样式 ──────────────────────────────────────────────────
    var style = document.createElement("style");
    style.textContent = [
      "#onin-fab-container { position: fixed; bottom: 24px; right: 20px; z-index: 2147483647;",
      "  font-family: ui-sans-serif, system-ui, -apple-system, sans-serif;",
      "  user-select: none; display: flex; flex-direction: column; align-items: flex-end; }",

      "#onin-fab-menu { position: absolute; bottom: 44px; right: 0; width: 224px; box-sizing: border-box;",
      "  background: rgba(28,28,30,0.96); backdrop-filter: blur(20px) saturate(180%);",
      "  -webkit-backdrop-filter: blur(20px) saturate(180%);",
      "  border: 1px solid rgba(255,255,255,0.10); border-radius: 12px;",
      "  box-shadow: 0 8px 32px rgba(0,0,0,0.5), 0 1px 0 rgba(255,255,255,0.05) inset;",
      "  display: flex; flex-direction: column; padding: 6px;",
      "  opacity: 0; visibility: hidden; transform: translateY(8px) scale(0.94);",
      "  transition: all 0.2s cubic-bezier(0.16,1,0.3,1); transform-origin: bottom right; pointer-events: none; }",

      "#onin-fab-menu *, #onin-fab-menu *::before, #onin-fab-menu *::after { box-sizing: border-box; }",
      "#onin-fab-menu.open { opacity: 1; visibility: visible; transform: translateY(0) scale(1); pointer-events: auto; }",

      ".onin-fab-item { background: transparent; color: rgba(235,235,245,0.9); border: none;",
      "  padding: 10px 12px; text-align: left; border-radius: 8px; cursor: pointer;",
      "  font-size: 13px; letter-spacing: -0.1px; transition: background 0.12s;",
      "  width: 100%; display: flex; align-items: center; justify-content: space-between;",
      "  gap: 12px; font-weight: 500; outline: none; }",
      ".onin-fab-item:hover { background: rgba(255,255,255,0.07); }",
      ".onin-fab-item:active { background: rgba(255,255,255,0.11); }",

      ".onin-fab-divider { height: 1px; background: rgba(255,255,255,0.07); margin: 6px 4px; flex-shrink: 0; }",

      "#onin-fab-btn { display: flex; align-items: center; justify-content: center;",
      "  width: 34px; height: 34px; border-radius: 50%;",
      "  background-color: rgba(40,40,42,0.92); color: rgba(255,255,255,0.85);",
      "  box-shadow: 0 2px 10px rgba(0,0,0,0.25); cursor: grab;",
      "  border: 1px solid rgba(255,255,255,0.14);",
      "  transition: transform 0.2s cubic-bezier(0.4,0,0.2,1), opacity 0.2s, box-shadow 0.2s, background-color 0.2s;",
      "  opacity: 0.45; outline: none; flex-shrink: 0; }",
      "#onin-fab-btn:active { cursor: grabbing; }",
      "#onin-fab-container:hover #onin-fab-btn,",
      "#onin-fab-menu.open ~ #onin-fab-btn {",
      "  opacity: 1; transform: scale(1.06); box-shadow: 0 4px 16px rgba(0,0,0,0.35);",
      "  background-color: rgba(58,58,60,0.98); }",

      ".onin-zoom-controls { display: flex; align-items: center; justify-content: space-between;",
      "  padding: 8px 12px; color: rgba(235,235,245,0.9); font-size: 13px;",
      "  font-weight: 500; letter-spacing: -0.1px; gap: 16px; }",

      ".onin-zoom-btn { background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1);",
      "  color: rgba(255,255,255,0.85); border-radius: 6px; width: 26px; height: 26px;",
      "  cursor: pointer; display: flex; align-items: center; justify-content: center;",
      "  transition: all 0.12s; padding: 0; margin: 0; font-size: 14px; outline: none; flex-shrink: 0; }",
      ".onin-zoom-btn:hover { background: rgba(255,255,255,0.13); }",
      ".onin-zoom-btn:active { background: rgba(255,255,255,0.18); transform: scale(0.93); }",

      ".onin-zoom-label { width: 28px; text-align: center; font-variant-numeric: tabular-nums;",
      "  font-size: 12px; font-weight: 600; opacity: 0.85; color: rgba(235,235,245,0.9); }",

      ".onin-icon-text { display: flex; align-items: center; gap: 8px; white-space: nowrap;",
      "  flex: 1 1 auto; min-width: 0; overflow: hidden; text-overflow: ellipsis; line-height: 1; }",
      ".onin-icon-text svg { width: 15px; height: 15px; opacity: 0.7; display: block; flex-shrink: 0; }",
    ].join("\n");
    document.head.appendChild(style);

    // ── 注入 HTML ─────────────────────────────────────────────────
    var btnContainer = document.createElement("div");
    btnContainer.id = "onin-fab-container";
    btnContainer.innerHTML =
      '<div id="onin-fab-menu">' +
      '<button class="onin-fab-item" id="onin-btn-pin">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256">' +
      '<path d="M216,168h-9.29L185.54,48H192a8,8,0,0,0,0-16H64a8,8,0,0,0,0,16h6.46L49.29,168H40a8,8,0,0,0,0,16h80v56a8,8,0,0,0,16,0V184h80a8,8,0,0,0,0-16ZM86.71,48h82.58l21.17,120H65.54Z"></path>' +
      "</svg>" +
      "窗口置顶" +
      "</span>" +
      '<div id="onin-pin-switch" style="width:34px;height:20px;border-radius:10px;background:rgba(255,255,255,0.15);position:relative;transition:0.25s cubic-bezier(0.4,0,0.2,1);flex-shrink:0;">' +
      '<div id="onin-pin-thumb" style="width:16px;height:16px;border-radius:50%;background:white;position:absolute;top:2px;left:2px;transition:0.25s cubic-bezier(0.4,0,0.2,1);box-shadow:0 1px 3px rgba(0,0,0,0.3);"></div>' +
      "</div>" +
      "</button>" +
      '<div class="onin-zoom-controls">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256">' +
      '<path d="M216,40H40A16,16,0,0,0,24,56V200a16,16,0,0,0,16,16H216a16,16,0,0,0,16-16V56A16,16,0,0,0,216,40Zm0,160H40V56H216V200ZM176,128a8,8,0,0,1-8,8H88a8,8,0,0,1,0-16h88a8,8,0,0,1,0,16Z"></path>' +
      "</svg>" +
      "缩放视图" +
      "</span>" +
      '<div style="display:flex;gap:6px;align-items:center;flex-shrink:0;">' +
      '<button class="onin-zoom-btn" id="onin-btn-zoom-out">-</button>' +
      '<span class="onin-zoom-label" id="onin-zoom-label">100</span>' +
      '<button class="onin-zoom-btn" id="onin-btn-zoom-in">+</button>' +
      "</div>" +
      "</div>" +
      '<div class="onin-fab-divider"></div>' +
      // 自动分离
      '<button class="onin-fab-item" id="onin-btn-auto-detach">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256"><path d="M224,128a96,96,0,1,1-96-96A96,96,0,0,1,224,128Z" opacity="0.2"></path><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216Zm45.66-93.66a8,8,0,0,1,0,11.32l-32,32a8,8,0,0,1-11.32-11.32L148.69,136H88a8,8,0,0,1,0-16h60.69l-18.35-18.34a8,8,0,0,1,11.32-11.32Z"></path></svg>' +
      "自动分离为独立窗口" +
      "</span>" +
      '<div id="onin-auto-detach-switch" style="width:34px;height:20px;border-radius:10px;background:rgba(255,255,255,0.15);position:relative;transition:0.25s;flex-shrink:0;">' +
      '<div id="onin-auto-detach-thumb" style="width:16px;height:16px;border-radius:50%;background:white;position:absolute;top:2px;left:2px;transition:0.25s;"></div>' +
      "</div>" +
      "</button>" +
      // 后台停止
      '<button class="onin-fab-item" id="onin-btn-terminate-bg">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256"><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216Zm40-88a8,8,0,0,1-8,8H96a8,8,0,0,1,0-16h64A8,8,0,0,1,168,128Z"></path></svg>' +
      "退出到后台立即结束" +
      "</span>" +
      '<div id="onin-terminate-bg-switch" style="width:34px;height:20px;border-radius:10px;background:rgba(255,255,255,0.15);position:relative;transition:0.25s;flex-shrink:0;">' +
      '<div id="onin-terminate-bg-thumb" style="width:16px;height:16px;border-radius:50%;background:white;position:absolute;top:2px;left:2px;transition:0.25s;"></div>' +
      "</div>" +
      "</button>" +
      // 随主程序启动
      '<button class="onin-fab-item" id="onin-btn-run-startup">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256"><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216Zm37.66-93.66-48,48a8,8,0,0,1-11.32,0l-24-24a8,8,0,0,1,11.32-11.32L112,156.69l42.34-42.35a8,8,0,0,1,11.32,11.32Z"></path></svg>' +
      "跟随主程序同时启动" +
      "</span>" +
      '<div id="onin-run-startup-switch" style="width:34px;height:20px;border-radius:10px;background:rgba(255,255,255,0.15);position:relative;transition:0.25s;flex-shrink:0;">' +
      '<div id="onin-run-startup-thumb" style="width:16px;height:16px;border-radius:50%;background:white;position:absolute;top:2px;left:2px;transition:0.25s;"></div>' +
      "</div>" +
      "</button>" +
      '<div class="onin-fab-divider"></div>' +
      // 刷新界面
      '<button class="onin-fab-item" id="onin-btn-reload">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256"><path d="M224,128a96,96,0,0,1-94.58,95.88l-2.45,0c-11.41,0-23.11-2.4-33.84-7.14a8,8,0,0,1,6.4-14.72c8.89,3.87,18.79,5.86,28.44,5.86,50.7,0,92-41.3,92-92s-41.3-92-92-92c-29.3,0-57,14-74.15,37.52L46.8,80H72a8,8,0,0,1,0,16H24a8,8,0,0,1-8-8V40a8,8,0,0,1,16,0V62.48l9.4-12.83C62.84,20,95,8,128,8A104,104,0,0,1,232,112Z"></path></svg>' +
      "刷新界面" +
      "</span>" +
      "</button>" +
      // 重启插件
      '<button class="onin-fab-item" id="onin-btn-restart">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256"><path d="M224,128a96,96,0,0,1-94.58,95.88l-2.45,0c-11.41,0-23.11-2.4-33.84-7.14a8,8,0,0,1,6.4-14.72c8.89,3.87,18.79,5.86,28.44,5.86,50.7,0,92-41.3,92-92s-41.3-92-92-92c-29.3,0-57,14-74.15,37.52L46.8,80H72a8,8,0,0,1,0,16H24a8,8,0,0,1-8-8V40a8,8,0,0,1,16,0V62.48l9.4-12.83C62.84,20,95,8,128,8A104,104,0,0,1,232,112Z"></path><path d="M128,80a48,48,0,1,0,48,48A48.05,48.05,0,0,0,128,80Zm0,80a32,32,0,1,1,32-32A32,32,0,0,1,128,160Z" opacity="0.2"></path></svg>' +
      "重启插件" +
      "</span>" +
      "</button>" +
      // 开发者工具
      '<button class="onin-fab-item" id="onin-btn-devtools">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256"><path d="M160,128a32,32,0,1,1-32-32A32,32,0,0,1,160,128Z" opacity="0.2"></path><path d="M216,40H40A16,16,0,0,0,24,56V200a16,16,0,0,0,16,16H216a16,16,0,0,0,16-16V56A16,16,0,0,0,216,40Zm0,160H40V56H216V200ZM176,128a32,32,0,1,0-32,32A32,32,0,0,0,176,128Z"></path></svg>' +
      "开发者工具" +
      "</span>" +
      "</button>" +
      // 卸载
      '<button class="onin-fab-item" id="onin-btn-uninstall" style="color: #ff453a;">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256"><path d="M216,48H40a8,8,0,0,0,0,16h8V208a16,16,0,0,0,16,16H192a16,16,0,0,0,16-16V64h8a8,8,0,0,0,0-16ZM192,208H64V64H192ZM80,24a8,8,0,0,1,8-8h80a8,8,0,0,1,8,8v16H80Z"></path></svg>' +
      "卸载插件" +
      "</span>" +
      "</button>" +
      '<div class="onin-fab-divider"></div>' +
      '<button class="onin-fab-item" id="onin-btn-inline">' +
      '<span class="onin-icon-text">' +
      '<svg fill="currentColor" viewBox="0 0 256 256">' +
      '<path d="M144,104V64a8,8,0,0,1,16,0V84.69l42.34-42.35a8,8,0,0,1,11.32,11.32L171.31,96H192a8,8,0,0,1,0,16H152A8,8,0,0,1,144,104Zm-40,40H64a8,8,0,0,0,0,16H84.69L42.34,202.34a8,8,0,0,0,11.32,11.32L96,171.31V192a8,8,0,0,0,16,0V152A8,8,0,0,0,104,144Zm67.31,16H192a8,8,0,0,0,0-16H152a8,8,0,0,0-8,8v40a8,8,0,0,0,16,0V171.31l42.34,42.35a8,8,0,0,0,11.32-11.32ZM104,56a8,8,0,0,0-8,8V84.69L53.66,42.34A8,8,0,0,0,42.34,53.66L84.69,96H64a8,8,0,0,0,0,16h40a8,8,0,0,0,8-8V64A8,8,0,0,0,104,56Z"></path>' +
      "</svg>" +
      "切回内联模式" +
      "</span>" +
      "</button>" +
      "</div>" +
      '<button id="onin-fab-btn" title="插件菜单">' +
      '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" fill="currentColor" viewBox="0 0 256 256">' +
      '<path d="M224,128a8,8,0,0,1-8,8H40a8,8,0,0,1,0-16H216A8,8,0,0,1,224,128ZM40,72H216a8,8,0,0,0,0-16H40a8,8,0,0,0,0,16ZM216,184H40a8,8,0,0,0,0,16H216a8,8,0,0,0,0-16Z"></path>' +
      "</svg>" +
      "</button>";
    document.body.appendChild(btnContainer);

    // ── 运行时变量（由 window.rs 中的短 format! 注入） ───────────
    var pluginId =
      (window.__ONIN_RUNTIME__ && window.__ONIN_RUNTIME__.pluginId) ||
      window.__PLUGIN_ID__ ||
      "unknown";

    // ── 按 pluginId 隔离的 localStorage 键名 ─────────────────────
    // 所有插件共享同一 origin（http://127.0.0.1:<port>），
    // 必须加 pluginId 前缀避免不同插件之间相互污染设置。
    var KEY_ZOOM = "onin:" + pluginId + ":zoom";
    var KEY_FAB_Y = "onin:" + pluginId + ":fab-y";
    var KEY_PINNED = "onin:" + pluginId + ":pinned";

    // ── 状态 ──────────────────────────────────────────────────────
    var isMenuOpen = false;
    // WebKit/Blink 均支持 style.zoom，Firefox 不支持但 Tauri webview 基于 WKWebView/WebView2
    var currentZoom = Number(localStorage.getItem(KEY_ZOOM)) || 1.0;
    // 从持久化存储恢复 pin 状态，避免窗口重建后开关与真实状态不一致
    var isPinned = localStorage.getItem(KEY_PINNED) === "true";
    var isDragging = false;
    var startY = 0;
    var initialBottom = 24;

    // 插件设置状态
    var settings = {
      auto_detach: false,
      terminate_on_bg: false,
      run_at_startup: false
    };

    // 恢复上次拖拽位置（加 clamp 防止旧值超出当前窗口范围）
    var storedBottom = localStorage.getItem(KEY_FAB_Y);
    if (storedBottom) {
      var parsedBottom = parseInt(storedBottom, 10);
      if (!isNaN(parsedBottom)) {
        // 同拖拽时相同的边界约束，避免在小窗口 / 分辨率切换后跑到屏幕外
        var maxBottom = window.innerHeight - 80;
        initialBottom = Math.min(Math.max(parsedBottom, 24), maxBottom);
      }
      btnContainer.style.bottom = initialBottom + "px";
    }

    var menuEl = document.getElementById("onin-fab-menu");
    var fabBtn = document.getElementById("onin-fab-btn");
    var pinSwitch = document.getElementById("onin-pin-switch");
    var pinThumb = document.getElementById("onin-pin-thumb");
    var zoomLabel = document.getElementById("onin-zoom-label");

    // 初始化缩放
    if (currentZoom !== 1.0) {
      document.body.style.zoom = currentZoom;
      zoomLabel.innerText = Math.round(currentZoom * 100);
    }

    // 恢复置顶开关视觉状态，并同步到真实窗口
    // 窗口每次创建时 always_on_top 默认为 false，必须重新调用命令才能真正生效
    if (isPinned) {
      pinSwitch.style.background = "#0F8BFF";
      pinThumb.style.left = "16px";
      // 主动同步：UI 已显示置顶，让真实窗口也置顶
      invoke("plugin_toggle_window_pin", {
        pluginId: pluginId,
        pin: true,
      }).catch(function (err) {
        // 同步失败则回退 UI 和持久化状态，避免假象
        console.warn("[FAB] 恢复置顶状态失败，重置:", err);
        isPinned = false;
        localStorage.removeItem(KEY_PINNED);
        pinSwitch.style.background = "rgba(255,255,255,0.15)";
        pinThumb.style.left = "2px";
      });
    }

    // 初始化获取插件详细设置
    invoke("get_plugin_with_schema", { pluginId: pluginId }).then(function (plugin) {
      if (plugin) {
        settings.auto_detach = !!plugin.auto_detach;
        settings.terminate_on_bg = !!plugin.terminate_on_bg;
        settings.run_at_startup = !!plugin.run_at_startup;

        updateSwitchUI("auto-detach", settings.auto_detach);
        updateSwitchUI("terminate-bg", settings.terminate_on_bg);
        updateSwitchUI("run-startup", settings.run_at_startup);
      }
    }).catch(console.error);

    function updateSwitchUI(id, enabled) {
      var sw = document.getElementById("onin-" + id + "-switch");
      var thumb = document.getElementById("onin-" + id + "-thumb");
      if (sw && thumb) {
        sw.style.background = enabled ? "#0F8BFF" : "rgba(255,255,255,0.15)";
        thumb.style.left = enabled ? "16px" : "2px";
      }
    }

    function toggleSetting(key, elementId) {
      var next = !settings[key];
      var commandMap = {
        auto_detach: "toggle_plugin_auto_detach",
        terminate_on_bg: "toggle_plugin_terminate_on_bg",
        run_at_startup: "toggle_plugin_run_at_startup"
      };

      // 乐观更新
      updateSwitchUI(elementId, next);

      var args = { pluginId: pluginId };
      args[key] = next;

      invoke(commandMap[key], args).then(function () {
        settings[key] = next;
      }).catch(function (err) {
        console.error("[FAB] Toggle failed:", err);
        updateSwitchUI(elementId, settings[key]); // 回滚
      });
    }

    // ── 辅助：invoke Tauri 命令 ───────────────────────────────────
    function invoke(cmd, args) {
      if (window.__TAURI__ && window.__TAURI__.core) {
        return window.__TAURI__.core.invoke(cmd, args);
      } else if (window.__TAURI_INTERNALS__) {
        return window.__TAURI_INTERNALS__.invoke(cmd, args);
      }
      return Promise.reject("Tauri API not available");
    }

    // ── 拖拽逻辑 ──────────────────────────────────────────────────
    fabBtn.addEventListener("mousedown", function (e) {
      isDragging = false;
      startY = e.clientY;
      var cs = window.getComputedStyle(btnContainer);
      initialBottom = parseInt(cs.bottom, 10) || 24;
    });

    document.addEventListener("mousemove", function (e) {
      if (e.buttons === 1 && Math.abs(e.clientY - startY) > 3) {
        isDragging = true;
        if (isMenuOpen) {
          isMenuOpen = false;
          menuEl.classList.remove("open");
        }
        fabBtn.style.transition = "none";
        var deltaY = startY - e.clientY;
        var newBottom = initialBottom + deltaY;
        if (newBottom < 24) newBottom = 24;
        var maxBottom = window.innerHeight - 80;
        if (newBottom > maxBottom) newBottom = maxBottom;
        btnContainer.style.bottom = newBottom + "px";
      }
    });

    document.addEventListener("mouseup", function () {
      if (isDragging) {
        fabBtn.style.transition = "";
        var finalBottom = parseInt(btnContainer.style.bottom, 10);
        localStorage.setItem(KEY_FAB_Y, finalBottom.toString());
        setTimeout(function () {
          isDragging = false;
        }, 50);
      }
    });

    // ── FAB 按钮开关菜单 ──────────────────────────────────────────
    fabBtn.addEventListener("click", function (e) {
      if (isDragging) return;
      e.preventDefault();
      e.stopPropagation();
      isMenuOpen = !isMenuOpen;
      menuEl.classList.toggle("open", isMenuOpen);
    });

    document.addEventListener("click", function (e) {
      if (isMenuOpen && !btnContainer.contains(e.target)) {
        isMenuOpen = false;
        menuEl.classList.remove("open");
      }
    });

    // ── 1. 切回内联模式 ───────────────────────────────────────────
    document
      .getElementById("onin-btn-inline")
      .addEventListener("click", async function (e) {
        e.preventDefault();
        e.stopPropagation();
        var shouldSwitch = false;
        try {
          shouldSwitch = await invoke("plugin_dialog_confirm", {
            options: {
              title: "切换显示方式",
              message:
                "切换显示方式会重新打开插件，当前页面状态可能丢失。确定继续吗？",
              kind: "warning",
            },
          });
        } catch (err) {
          console.error("[FAB] 切换确认弹窗失败:", err);
          return;
        }
        if (!shouldSwitch) return;
        invoke("return_to_inline_from_window", { pluginId: pluginId }).catch(
          console.error,
        );
      });

    // ── 2. 窗口置顶开关（悲观更新：命令成功后才持久化，失败则回滚） ────
    document
      .getElementById("onin-btn-pin")
      .addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        var next = !isPinned;
        // 先更新 UI（乐观显示），但不立即持久化
        if (next) {
          pinSwitch.style.background = "#0F8BFF";
          pinThumb.style.left = "16px";
        } else {
          pinSwitch.style.background = "rgba(255,255,255,0.15)";
          pinThumb.style.left = "2px";
        }
        invoke("plugin_toggle_window_pin", {
          pluginId: pluginId,
          pin: next,
        })
          .then(function () {
            // 命令成功才提交状态
            isPinned = next;
            localStorage.setItem(KEY_PINNED, isPinned.toString());
          })
          .catch(function (err) {
            // 命令失败：回滚 UI，不更新 isPinned / localStorage
            console.error("[FAB] 置顶切换失败，已回滚:", err);
            if (isPinned) {
              pinSwitch.style.background = "#0F8BFF";
              pinThumb.style.left = "16px";
            } else {
              pinSwitch.style.background = "rgba(255,255,255,0.15)";
              pinThumb.style.left = "2px";
            }
          });
      });

    // ── 3. 缩放控制 ───────────────────────────────────────────────
    function applyZoom(zoom) {
      document.body.style.zoom = zoom;
      zoomLabel.innerText = Math.round(zoom * 100);
      localStorage.setItem(KEY_ZOOM, zoom.toString());
    }

    document
      .getElementById("onin-btn-zoom-in")
      .addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        if (currentZoom < 3.0) currentZoom += 0.1;
        applyZoom(Math.round(currentZoom * 10) / 10);
      });

    document
      .getElementById("onin-btn-zoom-out")
      .addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        if (currentZoom > 0.3) currentZoom -= 0.1;
        applyZoom(Math.round(currentZoom * 10) / 10);
      });

    // ── 4. 插件项事件 ──────────────────────────────────────────────
    document.getElementById("onin-btn-auto-detach").addEventListener("click", function (e) {
      e.preventDefault(); e.stopPropagation();
      toggleSetting("auto_detach", "auto-detach");
    });

    document.getElementById("onin-btn-terminate-bg").addEventListener("click", function (e) {
      e.preventDefault(); e.stopPropagation();
      toggleSetting("terminate_on_bg", "terminate-bg");
    });

    document.getElementById("onin-btn-run-startup").addEventListener("click", function (e) {
      e.preventDefault(); e.stopPropagation();
      toggleSetting("run_at_startup", "run-startup");
    });

    document.getElementById("onin-btn-reload").addEventListener("click", function (e) {
      e.preventDefault(); e.stopPropagation();
      window.location.reload();
    });

    document.getElementById("onin-btn-restart").addEventListener("click", function (e) {
      e.preventDefault(); e.stopPropagation();
      invoke("plugin_restart_window", { pluginId: pluginId }).catch(console.error);
    });

    document.getElementById("onin-btn-devtools").addEventListener("click", function (e) {
      e.preventDefault(); e.stopPropagation();
      invoke("plugin_open_devtools").catch(console.error);
    });

    document.getElementById("onin-btn-uninstall").addEventListener("click", function (e) {
      e.preventDefault(); e.stopPropagation();
      if (confirm("确定要卸载此插件吗？此操作无法撤销。")) {
        invoke("uninstall_plugin", { pluginId: pluginId }).then(function () {
          // 卸载后关闭窗口
          invoke("plugin_close_window", { label: "plugin_" + pluginId.replace(/\./g, "_") });
        }).catch(console.error);
      }
    });

  });
})();
