use std::sync::Mutex;
use tauri::{App, AppHandle, Manager, WebviewWindow, Window};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId,
    IsIconic, SetForegroundWindow, ShowWindow, ASFW_ANY, SW_RESTORE, SW_SHOW,
};

pub struct PreviousForegroundWindow(pub Mutex<Option<isize>>);

pub fn setup(app: &mut App) {
    app.manage(PreviousForegroundWindow(Mutex::new(None)));
}

pub fn capture_previous_foreground(app: &AppHandle) {
    let Some(state) = app.try_state::<PreviousForegroundWindow>() else {
        return;
    };

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0 == 0 {
        return;
    }

    let mut foreground_process_id = 0;
    unsafe {
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut foreground_process_id));
    }

    let current_process_id = std::process::id();
    if foreground_process_id == current_process_id {
        return;
    }

    *state.0.lock().unwrap() = Some(hwnd.0 as isize);
}

pub fn restore_previous_foreground(app: &AppHandle) {
    let Some(state) = app.try_state::<PreviousForegroundWindow>() else {
        return;
    };

    let hwnd = state.0.lock().ok().and_then(|guard| *guard);
    if let Some(hwnd) = hwnd {
        let hwnd = HWND(hwnd as _);
        unsafe {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            } else {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

pub fn focus_webview_window(window: &WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();

    if let Ok(hwnd) = window.hwnd() {
        force_set_foreground_window(hwnd.0 as isize);
    }

    // 直接对 WebView2 子控件 SetFocus
    if let Ok(hwnd) = window.hwnd() {
        focus_webview_child(hwnd.0 as isize);
    }

    let _ = window.eval("window.focus()");

    // 多轮延迟重试：WebView2 子控件可能在 show 之后才异步创建/初始化，
    // 单次 focus_webview_child 可能找不到目标。在 50ms / 150ms / 300ms
    // 三个时间窗口各做一次 FindWindowExW → SetFocus + eval 的组合拳。
    let window1 = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _ = window1.eval("window.focus()");
        if let Ok(hwnd) = window1.hwnd() {
            focus_webview_child(hwnd.0 as isize);
        }
    });
    let window2 = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let _ = window2.eval("window.focus()");
        if let Ok(hwnd) = window2.hwnd() {
            focus_webview_child(hwnd.0 as isize);
        }
    });
    let window3 = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let _ = window3.eval("window.focus()");
        if let Ok(hwnd) = window3.hwnd() {
            focus_webview_child(hwnd.0 as isize);
        }
    });
}

pub fn focus_window(window: &Window) {
    let _ = window.unminimize();
    let _ = window.show();

    if let Ok(hwnd) = window.hwnd() {
        let isize_hwnd = hwnd.0 as isize;
        force_set_foreground_window(isize_hwnd);
    }

    let _ = window.set_focus();
}

/// 强制将指定窗口带到前台，处理 Windows 的防抢焦点限制。
///
/// 为避免主线程同步 Win32 SetFocus 到 WebView2 子 HWND 造成死锁/阻塞，
/// 这里只负责系统窗口层面的前台抢权：
/// - AttachThreadInput 临时附着到当前前台线程，绕过防抢焦点限制。
/// - BringWindowToTop、ShowWindow 和 SetForegroundWindow 强力激活宿主窗口。
/// - WebView2 子控件的键盘焦点由 focus_webview_child 单独处理。
///
/// 内建验证+一次重试：SetForegroundWindow 在 Windows 上可能因前台锁超时静默失败，
/// 特别是窗口刚被 show() 之后短暂处于过渡态时。验证后若仍未成为前台，则延迟后重试一次。
fn force_set_foreground_window(hwnd_isize: isize) {
    let hwnd_val = HWND(hwnd_isize as _);

    unsafe {
        let _ = AllowSetForegroundWindow(ASFW_ANY);

        let foreground_hwnd = GetForegroundWindow();
        let foreground_thread = GetWindowThreadProcessId(foreground_hwnd, None);
        let window_thread = GetWindowThreadProcessId(hwnd_val, None);

        if foreground_thread != window_thread && foreground_thread != 0 {
            let _ = AttachThreadInput(window_thread, foreground_thread, true);

            let _ = BringWindowToTop(hwnd_val);
            let _ = ShowWindow(hwnd_val, SW_SHOW);
            let _ = SetForegroundWindow(hwnd_val);

            let _ = AttachThreadInput(window_thread, foreground_thread, false);
        } else {
            let _ = BringWindowToTop(hwnd_val);
            let _ = ShowWindow(hwnd_val, SW_SHOW);
            let _ = SetForegroundWindow(hwnd_val);
        }

        // Verify the window really became foreground; retry once with a brief
        // delay if it didn't — Windows may reject SetForegroundWindow when the
        // target window is still transitioning from hidden to shown.
        let fg = GetForegroundWindow();
        if fg.0 != hwnd_val.0 {
            std::thread::sleep(std::time::Duration::from_millis(15));
            let _ = AllowSetForegroundWindow(ASFW_ANY);
            let _ = BringWindowToTop(hwnd_val);
            let _ = SetForegroundWindow(hwnd_val);
        }
    }
}

/// 递归查找 WebView2 子控件并对其调用 SetFocus。
///
/// Tauri 的 WebviewWindow::set_focus() 实际聚焦的是 tao 原生窗口，
/// 键盘焦点无法传递到 WebView2 子控件。通过遍历子窗口树找到
/// WebView2 的 HWND 并直接 SetFocus。
///
/// WebView2 可能在不同的线程上（wry 创建的 webview 线程），
/// 因此需要 AttachThreadInput 临时附着以允许跨线程 SetFocus。
fn focus_webview_child(parent_isize: isize) {
    extern "system" {
        fn FindWindowExW(
            parent: isize,
            child_after: isize,
            class: *const u16,
            window: *const u16,
        ) -> isize;
        fn GetClassNameW(hwnd: isize, class: *mut u16, max_count: i32) -> i32;
        fn SetFocus(hwnd: isize) -> isize;
    }

    unsafe fn focus_one(hwnd: isize) -> bool {
        // Attach to the target window's thread so SetFocus can cross thread boundaries
        let target_thread = GetWindowThreadProcessId(HWND(hwnd as _), None);
        let current_thread = GetCurrentThreadId();
        let need_detach = target_thread != 0 && target_thread != current_thread;

        if need_detach {
            let _ = AttachThreadInput(current_thread, target_thread, true);
        }

        let result = SetFocus(hwnd);

        if need_detach {
            let _ = AttachThreadInput(current_thread, target_thread, false);
        }

        result != 0
    }

    unsafe fn find_and_focus(parent: isize) -> bool {
        let mut child = FindWindowExW(parent, 0, std::ptr::null(), std::ptr::null());
        while child != 0 {
            let mut buf = [0u16; 64];
            let len = GetClassNameW(child, buf.as_mut_ptr(), buf.len() as i32);
            if len > 0 {
                let name = String::from_utf16_lossy(&buf[..len as usize]);
                // WebView2 在不同版本/配置下可能有不同的类名前缀
                if name.starts_with("Chrome_WidgetWin_")
                    || name.contains("WebView")
                    || name.contains("Edge")
                {
                    if focus_one(child) {
                        return true;
                    }
                }
                // 递归搜索子窗口（WebView2 可能嵌套多层）
                if find_and_focus(child) {
                    return true;
                }
            }
            child = FindWindowExW(parent, child, std::ptr::null(), std::ptr::null());
        }
        false
    }

    unsafe {
        // 先尝试查找 WebView2
        if !find_and_focus(parent_isize) {
            // 找不到特定子窗口时，直接 focus 第一个子窗口（兜底）
            let child = FindWindowExW(parent_isize, 0, std::ptr::null(), std::ptr::null());
            if child != 0 {
                focus_one(child);
            }
        }
    }
}
