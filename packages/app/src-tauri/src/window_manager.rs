use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{App, AppHandle, Emitter, Listener, Manager, State};
use tokio::time::sleep;

// ============================================================================
// 状态定义
// ============================================================================

/// 窗口可见性状态机状态
///
/// - `hiding_initiated_by_command`：隐藏是否由命令/快捷键触发（防止失焦时重复隐藏）
/// - `show_generation`：显示代数。每次 `request_show` 递增；智能隐藏任务据此判断
///   自己是否已过期——若期间窗口被重新唤起（代数变化），必须放弃隐藏，否则会把
///   刚显示出来的窗口再次隐藏，导致"唤醒后无法获取焦点"。
pub struct WindowState {
    pub hiding_initiated_by_command: AtomicBool,
    pub show_generation: AtomicU64,
}

/// 防止窗口在某些操作（如对话框打开）期间关闭
/// 使用计数器以支持多个操作同时加锁
pub struct WindowCloseLockState(pub AtomicU32);

/// 持有隐藏任务的句柄，以便可以取消
///
/// 使用 `std::sync::Mutex` 而非 tokio Mutex：取消必须同步完成
/// （`request_show` 在显示窗口前必须确保陈旧隐藏任务已 abort），
/// 不能在异步竞态中"尽力而为"。
pub struct HideTaskState {
    pub handle: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

// ============================================================================
// 鼠标定位与显示器匹配
// ============================================================================

// TODO: 提升至 pub(crate) 预留供给后续录屏扩展或跨显示器定位逻辑等自定义扩展进行鼠标坐标抓取使用
pub(crate) fn get_cursor_position() -> Option<(i32, i32)> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        let mut point = windows::Win32::Foundation::POINT::default();
        unsafe {
            if GetCursorPos(&mut point).is_ok() {
                Some((point.x, point.y))
            } else {
                None
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        use core_graphics::event::CGEvent;
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState);
        let event = source.ok().and_then(|src| CGEvent::new(src).ok());
        if let Some(event) = event {
            let point = event.location();
            Some((point.x as i32, point.y as i32))
        } else {
            None
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

// TODO: 提升至 pub(crate) 预留供给后续跨窗口层次切换、显示器映射及关联时匹配鼠标所在屏幕使用
pub(crate) fn find_monitor_for_cursor(
    monitors: &[tauri::Monitor],
    _cursor_pos: (i32, i32),
) -> Option<&tauri::Monitor> {
    #[cfg(target_os = "windows")]
    {
        let (cx, cy) = _cursor_pos;
        for monitor in monitors {
            let m_pos = monitor.position();
            let m_size = monitor.size();
            let min_x = m_pos.x;
            let max_x = m_pos.x + m_size.width as i32;
            let min_y = m_pos.y;
            let max_y = m_pos.y + m_size.height as i32;

            if cx >= min_x && cx < max_x && cy >= min_y && cy < max_y {
                return Some(monitor);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let (cx, cy) = _cursor_pos;
        let cx_f = cx as f64;
        let cy_f = cy as f64;

        for monitor in monitors {
            let m_pos = monitor.position();
            let m_size = monitor.size();
            let sf = monitor.scale_factor();

            // 在 macOS 上，CGEvent 坐标为逻辑像素，而 Monitor 坐标为物理像素，需进行转换
            let min_x = m_pos.x as f64 / sf;
            let max_x = min_x + m_size.width as f64 / sf;
            let min_y = m_pos.y as f64 / sf;
            let max_y = min_y + m_size.height as f64 / sf;

            if cx_f >= min_x && cx_f < max_x && cy_f >= min_y && cy_f < max_y {
                return Some(monitor);
            }
        }
    }

    // 兜底返回第一个可用的显示器
    monitors.first()
}

/// 将窗口移动到鼠标所在的显示器并居中
pub fn move_window_to_cursor_monitor(window: &tauri::WebviewWindow) {
    let monitors = match window.available_monitors() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[window_manager] 无法获取显示器列表: {}", e);
            return;
        }
    };

    if monitors.is_empty() {
        return;
    }

    if let Some(cursor_pos) = get_cursor_position() {
        if let Some(monitor) = find_monitor_for_cursor(&monitors, cursor_pos) {
            let monitor_pos = monitor.position();
            let monitor_size = monitor.size();

            // 获取窗口当前大小，如果获取不到则使用默认大小 (960, 600)
            let window_size = window
                .outer_size()
                .unwrap_or(tauri::PhysicalSize::new(960, 600));

            // 计算居中坐标 (物理像素)
            let new_x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
            let new_y =
                monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;

            if let Err(e) = window.set_position(tauri::PhysicalPosition::new(new_x, new_y)) {
                eprintln!("[window_manager] 移动窗口失败: {}", e);
            }
        }
    }
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 获取窗口关闭锁
#[tauri::command]
pub fn acquire_window_close_lock(state: State<WindowCloseLockState>) {
    state.0.fetch_add(1, Ordering::Relaxed);
}

/// 释放窗口关闭锁
#[tauri::command]
pub fn release_window_close_lock(state: State<WindowCloseLockState>) {
    if state.0.load(Ordering::Relaxed) > 0 {
        state.0.fetch_sub(1, Ordering::Relaxed);
    }
}

/// 隐藏主窗口命令
///
/// 统一入口：所有"收起主窗口"的路径（Esc、快捷键 toggle、前端调用）都走这里，
/// 保证时序一致：先标记命令隐藏 → 恢复上一个前台窗口 → 再 hide。
#[tauri::command]
pub fn close_main_window(app: tauri::AppHandle, _state: State<WindowState>) {
    request_hide(&app);
}

/// 统一的"显示主窗口"入口
///
/// 所有唤醒路径（全局快捷键、托盘、Esc 返回、macOS 激活、前端命令）都必须走这里：
/// 1. 在显示之前捕获当前前台窗口（保证 restore 有据可依）
/// 2. 递增显示代数并同步取消待定的智能隐藏任务（杜绝陈旧任务杀死本次唤醒）
/// 3. 移动窗口到鼠标所在显示器
/// 4. 执行统一激活序列（显示 + 确定性抢前台 + 后台异步验证）
/// 5. 通知前端
pub fn request_show(app: &AppHandle) {
    // 1. 捕获当前前台窗口（必须在显示之前）
    crate::focus_manager::capture_previous_foreground(app);

    if let Some(window) = app.get_webview_window("main") {
        // 2. 递增代数 + 同步取消待定隐藏任务
        let state: State<WindowState> = app.state();
        state.show_generation.fetch_add(1, Ordering::SeqCst);
        state
            .hiding_initiated_by_command
            .store(false, Ordering::Relaxed);
        cancel_hide_task_sync(app);

        // 3. 移动窗口到鼠标所在的显示器
        move_window_to_cursor_monitor(&window);

        // 4. 统一激活序列（显示 + 抢前台 + 后台验证）
        crate::focus_manager::focus_webview_window(&window);

        // 5. 通知前端
        let _ = window.emit("window_visibility", &true);
    } else if let Some(window) = app.get_window("main") {
        let state: State<WindowState> = app.state();
        state.show_generation.fetch_add(1, Ordering::SeqCst);
        state
            .hiding_initiated_by_command
            .store(false, Ordering::Relaxed);
        cancel_hide_task_sync(app);

        crate::focus_manager::focus_window(&window);
        let _ = window.emit("window_visibility", &true);
    } else {
        eprintln!("[window_manager] Main window not found for showing");
    }
}

/// 统一的"隐藏主窗口"入口（快捷键 toggle 使用，逻辑与 close_main_window 一致）
pub fn request_hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let was_visible = window.is_visible().unwrap_or(false);
        if !was_visible {
            return;
        }
        // 只有窗口聚焦时才标记——若窗口本就无焦点，hide 不会产生 blur，打标记只会
        // 留下陈旧状态，被下一次无关的 blur 错误消费。
        if window.is_focused().unwrap_or(false) {
            let state: State<WindowState> = app.state();
            state
                .hiding_initiated_by_command
                .store(true, Ordering::Relaxed);
        }
        crate::focus_manager::restore_previous_foreground(app);
        window.hide().ok();
        let _ = window.emit("window_visibility", &false);
    } else if let Some(window) = app.get_window("main") {
        let was_visible = window.is_visible().unwrap_or(false);
        if was_visible {
            crate::focus_manager::restore_previous_foreground(app);
            window.hide().ok();
            let _ = window.emit("window_visibility", &false);
        }
    } else {
        eprintln!("[window_manager] Main window not found for hiding");
    }
}

#[allow(dead_code)]
pub fn show_main_window(app: &AppHandle) {
    request_show(app);
}

/// 暴露给前端的显示主窗口命令
#[tauri::command]
pub fn show_main_window_cmd(app: tauri::AppHandle) {
    request_show(&app);
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 同步取消任何挂起的隐藏任务
///
/// 必须在显示窗口之前完成：若待定的智能隐藏任务在窗口刚显示、焦点尚未落地时触发，
/// 会把窗口再次隐藏，导致唤醒失败。使用 `JoinHandle::abort()` 立即取消，
/// 不依赖异步竞态。
fn cancel_hide_task_sync(app: &AppHandle) {
    let state: State<HideTaskState> = app.state();
    // 绑定到具名局部变量：if-let scrutinee 的临时值（持有 MutexGuard 的 Result）
    // 会被延长到块末尾才析构，晚于 `state` 的 drop，导致借用超期（E0597）。
    let lock_result = state.handle.lock();
    if let Ok(mut guard) = lock_result {
        if let Some(handle) = guard.take() {
            handle.abort();
        }
    }
}

/// 检查并释放窗口关闭锁
fn release_lock_if_held(app_handle: &AppHandle) {
    let lock_state: State<WindowCloseLockState> = app_handle.state();
    if lock_state.0.load(Ordering::Relaxed) > 0 {
        lock_state.0.fetch_sub(1, Ordering::Relaxed);
    }
}

// ============================================================================
// 文件拖放事件处理
// ============================================================================

/// 设置文件拖放事件监听器
///
/// 在文件拖放期间锁定窗口，防止意外隐藏
fn setup_file_drop_listeners(window: &tauri::WebviewWindow, app_handle: &AppHandle) {
    // 文件悬停：获取锁并取消隐藏任务
    let app_handle_hover = app_handle.clone();
    let app_handle_hover_cancel = app_handle.clone();
    window.listen("tauri://file-drop-hover", move |_event| {
        let lock_state: State<WindowCloseLockState> = app_handle_hover.state();
        lock_state.0.fetch_add(1, Ordering::Relaxed);
        cancel_hide_task_sync(&app_handle_hover_cancel);
    });

    // 文件放下：释放锁
    let app_handle_drop = app_handle.clone();
    window.listen("tauri://file-drop", move |_event| {
        release_lock_if_held(&app_handle_drop);
    });

    // 拖放取消：释放锁
    let app_handle_cancel = app_handle.clone();
    window.listen("tauri://file-drop-cancelled", move |_event| {
        release_lock_if_held(&app_handle_cancel);
    });
}

// ============================================================================
// 智能隐藏任务
// ============================================================================

/// 创建隐藏任务
///
/// 短延迟后隐藏窗口，具有以下特性：
/// - 50ms 延迟防止闪烁
/// - 隐藏前检查"显示代数"：若期间窗口被重新唤起（`request_show` 递增了代数），
///   说明这是一次过期的隐藏任务，必须放弃——否则会把刚显示出来的窗口再次隐藏，
///   直接导致"唤醒后无法获取焦点"
/// - 隐藏前最终检查窗口状态（焦点和锁定）
/// - 文件拖拽期间由 WindowCloseLockState 保护
fn spawn_smart_hide_task(
    window: tauri::WebviewWindow,
    app_handle: AppHandle,
    show_generation_at_blur: u64,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        // 短延迟后尝试隐藏
        sleep(Duration::from_millis(50)).await;

        // 代数守卫：期间若有新的显示请求，放弃隐藏
        let window_state: State<WindowState> = app_handle.state();
        if window_state.show_generation.load(Ordering::SeqCst) != show_generation_at_blur {
            return;
        }

        // 最终检查并隐藏
        let lock_state: State<WindowCloseLockState> = app_handle.state();
        let is_focused = {
            #[allow(unused_mut)]
            let mut focused = window.is_focused().unwrap_or(false);
            #[cfg(target_os = "windows")]
            if !focused {
                use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetParent};

                if let Ok(hwnd) = window.hwnd() {
                    let fg_hwnd = unsafe { GetForegroundWindow() };
                    if fg_hwnd.0 != 0 {
                        let mut current = fg_hwnd;
                        while current.0 != 0 {
                            if current.0 == (hwnd.0 as isize) {
                                focused = true;
                                break;
                            }
                            current = unsafe { GetParent(current) };
                        }
                    }
                }
            }
            focused
        };

        let is_locked = lock_state.0.load(Ordering::Relaxed) > 0;

        if !is_focused && !is_locked {
            if window.is_visible().unwrap_or(false) {
                window.hide().ok();
                window.emit("window_visibility", &false).unwrap_or_default();
            }
        }

        // 清理任务句柄
        let hide_task_state: State<HideTaskState> = app_handle.state();
        let lock_result = hide_task_state.handle.lock();
        if let Ok(mut handle_guard) = lock_result {
            *handle_guard = None;
        }
    })
}

/// 存储隐藏任务句柄（同步，避免与取消操作竞态）
fn store_hide_task_handle(app_handle: &AppHandle, handle: tauri::async_runtime::JoinHandle<()>) {
    let state: State<HideTaskState> = app_handle.state();
    let lock_result = state.handle.lock();
    if let Ok(mut guard) = lock_result {
        *guard = Some(handle);
    }
}

// ============================================================================
// 窗口焦点事件处理
// ============================================================================

/// 处理窗口获得焦点
fn handle_window_focused(app_handle: &AppHandle) {
    // 取消隐藏任务
    cancel_hide_task_sync(app_handle);
}

/// 处理窗口失去焦点
fn handle_window_blur(app_handle: &AppHandle, window: &tauri::WebviewWindow) {
    let window_state: State<WindowState> = app_handle.state();
    let lock_state: State<WindowCloseLockState> = app_handle.state();

    // 如果窗口被锁定，跳过隐藏
    if lock_state.0.load(Ordering::Relaxed) > 0 {
        return;
    }

    // 检查是否是命令触发的隐藏
    if window_state
        .hiding_initiated_by_command
        .swap(false, Ordering::Relaxed)
    {
        cancel_hide_task_sync(app_handle);
        return;
    }

    // 记录本次失焦时的显示代数，智能隐藏任务据此判断自己是否过期
    let generation = window_state.show_generation.load(Ordering::SeqCst);

    // 启动智能隐藏任务
    let handle = spawn_smart_hide_task(window.clone(), app_handle.clone(), generation);
    store_hide_task_handle(app_handle, handle);
}

#[cfg(target_os = "windows")]
fn is_windows_11() -> bool {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion") {
        if let Ok(build_str) = key.get_value::<String, _>("CurrentBuildNumber") {
            if let Ok(build) = build_str.parse::<u32>() {
                return build >= 22000;
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
pub fn setup_windows_window_effects(window: &tauri::WebviewWindow) {
    let is_win11 = is_windows_11();

    if is_win11 {
        // Windows 11: 设置 DWM 原生大圆角属性
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE,
        };

        if let Ok(hwnd) = window.hwnd() {
            let hwnd_raw = HWND(hwnd.0 as _);
            let preference = 2u32; // DWMWCP_ROUND
            let _ = unsafe {
                DwmSetWindowAttribute(
                    hwnd_raw,
                    DWMWA_WINDOW_CORNER_PREFERENCE,
                    &preference as *const _ as *const _,
                    std::mem::size_of::<u32>() as u32,
                )
            };
        }

        // 应用 Windows 11 原生 Mica 特效
        if let Err(err) = window_vibrancy::apply_mica(window, Some(true)) {
            eprintln!("[window_manager] 无法应用 Windows 11 Mica 效果: {:?}", err);
        } else {
            println!("[window_manager] 成功为主窗口应用了 Windows 11 Mica 效果");
        }
    } else {
        // Windows 10: 保持原生透明通道，由前端 CSS 渲染抗锯齿圆角卡片。
        // 规避 Win10 DWM 亚克力拖动掉帧及直角溢出缺陷，确保丝滑 120Hz 满帧与零锯齿圆角。
        println!("[window_manager] Windows 10 环境：已启用纯净透明通道");
    }
}

#[cfg(target_os = "macos")]
pub fn setup_macos_window_effects(window: &tauri::WebviewWindow) {
    // 1. 启用系统原生窗口阴影，由 WindowServer 渲染高质量柔和边缘投射
    if let Err(err) = window.set_shadow(true) {
        eprintln!("[window_manager] 无法为主窗口启用原生阴影: {:?}", err);
    } else {
        println!("[window_manager] 成功为主窗口启用了原生阴影");
    }

    // 2. 清除可能残留的旧 Vibrancy
    let _ = window_vibrancy::clear_vibrancy(window);

    // 3. 应用 macOS 专属 HudWindow 原生磨砂毛玻璃材质
    // - 选用 HudWindow（macOS Spotlight / HUD 浮窗原生毛玻璃特效，深邃通透）
    // - 状态设为 Active，保证应用失焦时依然保持毛玻璃模糊质感，避免变灰
    // - 半径传入 Some(16.0)，与前端卡片 rounded-2xl（16px）保持严格一致
    if let Err(err) = window_vibrancy::apply_vibrancy(
        window,
        window_vibrancy::NSVisualEffectMaterial::HudWindow,
        Some(window_vibrancy::NSVisualEffectState::Active),
        Some(16.0),
    ) {
        eprintln!(
            "[window_manager] 无法为主窗口应用 macOS 毛玻璃效果: {:?}",
            err
        );
        return;
    } else {
        println!("[window_manager] 成功为主窗口应用了 macOS HudWindow 毛玻璃效果");
    }

    // 4. 对 NSVisualEffectView 及其 backing layer 执行 16px 圆角裁剪及清空底层背景
    // 彻底根除无边框窗口下直角视觉溢出（白色三角直角）的缺陷
    apply_macos_vibrancy_mask(window, 16.0);
}

#[cfg(target_os = "macos")]
fn apply_macos_vibrancy_mask(window: &tauri::WebviewWindow, radius: f64) {
    use objc2::msg_send;
    use objc2_app_kit::NSWindow;
    use objc2_foundation::NSInteger;

    let Ok(ns_window_ptr) = window.ns_window() else {
        return;
    };

    let ns_window: &NSWindow = unsafe { &*(ns_window_ptr as *const NSWindow) };

    // 确保窗口自身透明
    ns_window.setOpaque(false);

    if let Some(content_view) = ns_window.contentView() {
        const NS_VIEW_TAG_BLUR_VIEW: NSInteger = 91376254;
        if let Some(blur_view) = content_view.viewWithTag(NS_VIEW_TAG_BLUR_VIEW) {
            blur_view.setWantsLayer(true);
            if let Some(layer) = blur_view.layer() {
                unsafe {
                    let () = msg_send![&*layer, setCornerRadius: radius];
                    let () = msg_send![&*layer, setMasksToBounds: true];
                }
            }
            println!("[window_manager] 成功为 macOS 毛玻璃视图设置了 16px 圆角裁剪");
        }
    }
}

// ============================================================================
// 主设置函数
// ============================================================================

/// 设置窗口事件监听器
pub fn setup_window_events(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let app_handle = app.handle().clone();

    // 设置文件拖放事件
    setup_file_drop_listeners(&window, &app_handle);

    // 设置窗口焦点事件
    let window_for_blur = window.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(true) => {
            handle_window_focused(&app_handle);
        }
        tauri::WindowEvent::Focused(false) => {
            handle_window_blur(&app_handle, &window_for_blur);
        }
        _ => {}
    });

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn setup_activation_observer(app: &AppHandle) {
    macos_activation::setup(app);
}

#[cfg(target_os = "macos")]
mod macos_activation {
    use super::show_main_window;
    use objc2::rc::Retained;
    use objc2::{define_class, msg_send, sel, ClassType};
    use objc2_app_kit::{
        NSApplicationDidBecomeActiveNotification, NSApplicationDidUnhideNotification,
    };
    use objc2_foundation::{NSNotification, NSNotificationCenter};
    use once_cell::sync::{Lazy, OnceCell};
    use std::sync::Mutex;
    use tauri::{AppHandle, Manager};

    define_class!(
        #[unsafe(super(objc2::runtime::NSObject))]
        struct ActivationObserver;

        impl ActivationObserver {
            #[unsafe(method(handleDidBecomeActive:))]
            fn handle_did_become_active(&self, _notification: &NSNotification) {
                if let Some(app) = APP_HANDLE.get() {
                    // 检查是否有任何非 "main" 的窗口是可见的
                    let has_other_visible_window = app.windows().iter().any(|(label, window)| {
                        label != "main" && window.is_visible().unwrap_or(false)
                    });

                    if !has_other_visible_window {
                        show_main_window(app);
                    }
                }
            }
        }
    );

    static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();
    static OBSERVER: Lazy<Mutex<Option<Retained<ActivationObserver>>>> =
        Lazy::new(|| Mutex::new(None));

    pub fn setup(app: &AppHandle) {
        let _ = APP_HANDLE.set(app.clone());
        let observer: Retained<ActivationObserver> =
            unsafe { msg_send![ActivationObserver::class(), new] };
        let center = NSNotificationCenter::defaultCenter();
        unsafe {
            center.addObserver_selector_name_object(
                &observer,
                sel!(handleDidBecomeActive:),
                Some(&NSApplicationDidBecomeActiveNotification),
                None,
            );
            center.addObserver_selector_name_object(
                &observer,
                sel!(handleDidBecomeActive:),
                Some(&NSApplicationDidUnhideNotification),
                None,
            );
        }
        if let Ok(mut guard) = OBSERVER.lock() {
            *guard = Some(observer);
        }
    }
}
