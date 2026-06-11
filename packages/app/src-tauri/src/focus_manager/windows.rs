use std::sync::Mutex;
use tauri::{App, AppHandle, Manager, WebviewWindow, Window};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::AttachThreadInput;
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
}

pub fn focus_window(window: &Window) {
    let _ = window.unminimize();
    let _ = window.show();

    if let Ok(hwnd) = window.hwnd() {
        let isize_hwnd = hwnd.0 as isize;
        force_set_foreground_window(isize_hwnd);
    }
}

/// 强制将指定窗口带到前台，处理 Windows 的防抢焦点限制。
///
/// 为避免主线程同步 Win32 SetFocus 到 WebView2 子 HWND 造成死锁/阻塞，
/// 这里只负责系统窗口层面的前台抢权：
/// - AttachThreadInput 临时附着到当前前台线程，绕过防抢焦点限制。
/// - BringWindowToTop、ShowWindow 和 SetForegroundWindow 强力激活宿主窗口。
/// - DOM 焦点由前端 requestInputFocusWithRetry 重试轮询机制负责。
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
    }
}
