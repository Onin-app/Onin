use std::sync::Mutex;
use tauri::{App, AppHandle, Manager, WebviewWindow, Window};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::System::Threading::AttachThreadInput;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, BringWindowToTop, EnumChildWindows, GetForegroundWindow,
    GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow,
    ShowWindow, ASFW_ANY, SW_RESTORE, SW_SHOW,
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
    if let Ok(hwnd) = window.hwnd() {
        let isize_hwnd = hwnd.0 as isize;
        force_set_foreground_window(isize_hwnd);
    }

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn focus_window(window: &Window) {
    if let Ok(hwnd) = window.hwnd() {
        let isize_hwnd = hwnd.0 as isize;
        force_set_foreground_window(isize_hwnd);
    }

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

/// 强制将指定窗口带到前台，处理 Windows 的防抢焦点限制。
///
/// 这里不能简化成普通的 show() + set_focus()。
/// Tauri 在 Windows 下是外层宿主窗口 + 内层 WebView2 子窗口的双层结构：
/// - 宿主 HWND 负责系统窗口层面的显示、层级、移动和缩放。
/// - WebView2 子 HWND 才是真正接收键盘输入和 DOM 焦点的目标。
///
/// 如果只把宿主窗口 show 出来，或者仅对宿主 HWND 调用 SetFocus，常见回归是：
/// - 窗口只在任务栏闪烁，仍然抢不到前台。
/// - 页面看起来有光标，但实际输入仍落在上一个应用里。
/// - 全局快捷键带 Alt 时，系统菜单状态残留，导致焦点异常。
///
/// 这段实现依赖几个关键步骤共同生效：
/// - AttachThreadInput 临时附着到当前前台线程，绕过防抢焦点限制。
/// - EnumChildWindows 找到内部 WebView2 子窗口，而不是只操作外层宿主窗口。
/// - SetFocus 最终打到真实接收输入的子 HWND，保证键盘事件进入 WebView DOM。
///
/// 当存在 inline 插件 child webview 时，不能再拿“枚举到的第一个 child”直接聚焦，
/// 否则隐藏但未销毁的插件 webview 可能会长期截获键盘焦点。
/// 这里优先选择可见且面积最大的 child，通常就是主 WebView2。
fn force_set_foreground_window(hwnd_isize: isize) {
    let hwnd_val = HWND(hwnd_isize as _);

    unsafe {
        let _ = AllowSetForegroundWindow(ASFW_ANY);

        let foreground_hwnd = GetForegroundWindow();
        let foreground_thread = GetWindowThreadProcessId(foreground_hwnd, None);
        let window_thread = GetWindowThreadProcessId(hwnd_val, None);

        #[derive(Clone, Copy)]
        struct ChildWindowCandidate {
            first_child: HWND,
            best_visible_child: HWND,
            best_visible_area: i64,
        }

        unsafe extern "system" fn enum_child_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let state = &mut *(lparam.0 as *mut ChildWindowCandidate);

            if state.first_child.0 == 0 {
                state.first_child = hwnd;
            }

            if IsWindowVisible(hwnd).as_bool() {
                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    let width = (rect.right - rect.left).max(0) as i64;
                    let height = (rect.bottom - rect.top).max(0) as i64;
                    let area = width * height;
                    if area > state.best_visible_area {
                        state.best_visible_area = area;
                        state.best_visible_child = hwnd;
                    }
                }
            }

            BOOL(1)
        }

        let mut child_state = ChildWindowCandidate {
            first_child: HWND(0),
            best_visible_child: HWND(0),
            best_visible_area: -1,
        };
        let _ = EnumChildWindows(
            hwnd_val,
            Some(enum_child_proc),
            LPARAM(&mut child_state as *mut _ as isize),
        );

        let target_focus_hwnd = if child_state.best_visible_child.0 != 0 {
            child_state.best_visible_child
        } else if child_state.first_child.0 != 0 {
            child_state.first_child
        } else {
            hwnd_val
        };

        if foreground_thread != window_thread && foreground_thread != 0 {
            let _ = AttachThreadInput(window_thread, foreground_thread, true);

            let _ = BringWindowToTop(hwnd_val);
            let _ = ShowWindow(hwnd_val, SW_SHOW);
            let _ = SetForegroundWindow(hwnd_val);
            let _ = SetFocus(target_focus_hwnd);

            let _ = AttachThreadInput(window_thread, foreground_thread, false);
        } else {
            let _ = BringWindowToTop(hwnd_val);
            let _ = ShowWindow(hwnd_val, SW_SHOW);
            let _ = SetForegroundWindow(hwnd_val);
            let _ = SetFocus(target_focus_hwnd);
        }
    }
}
