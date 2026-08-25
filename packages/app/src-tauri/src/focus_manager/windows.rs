use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{App, AppHandle, Manager, WebviewWindow, Window};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, SetFocus, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, VK_MENU,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, BringWindowToTop, EnumChildWindows, GetClassNameW,
    GetForegroundWindow, GetWindowThreadProcessId, IsIconic, SetForegroundWindow, ShowWindow,
    ASFW_ANY, SW_RESTORE, SW_SHOW,
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

/// 统一焦点入口：显示窗口 + 确定性前台激活 + 后台异步验证。
///
/// 分层设计（事件驱动，不再依赖前端轮询 document.hasFocus()）：
/// 1. 同步段（主线程，微秒级）：显示窗口，并立即执行确定性激活配方
///    （ALT 键模拟 + AttachThreadInput + SetForegroundWindow），处理常见情况。
/// 2. 异步验证段（后台定时器，不阻塞主线程消息泵）：WebView2 渲染与 OS 激活
///    存在几十毫秒的过渡期，前台可能被抢占或 SetForegroundWindow 被前台锁拒绝。
///    后台在 40/120/300ms 各验证一次，失败则用 ALT 键技巧重试；
///    确认成为前台后，再对 WebView2 子控件 SetFocus，并把焦点交给页面。
pub fn focus_webview_window(window: &WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();

    if let Ok(hwnd) = window.hwnd() {
        activate_window(hwnd.0 as isize);
    }

    let window_verify = window.clone();
    tauri::async_runtime::spawn(async move {
        let Ok(hwnd) = window_verify.hwnd() else {
            return;
        };
        let hwnd_isize = hwnd.0 as isize;

        if !verify_foreground_with_retry(&window_verify, hwnd_isize).await {
            return;
        }

        // 前台已确认：聚焦 WebView2 子控件（键盘焦点落点），并通知页面
        focus_webview_child(hwnd_isize);
        let _ = window_verify.eval("window.focus()");
    });
}

pub fn focus_window(window: &Window) {
    let _ = window.unminimize();
    let _ = window.show();

    if let Ok(hwnd) = window.hwnd() {
        activate_window(hwnd.0 as isize);
    }

    let _ = window.set_focus();
}

/// 后台验证窗口是否已成为前台；失败时用 ALT 键技巧重试。
///
/// 注意：此函数在 tokio 后台线程上执行，所有 Win32 调用均不依赖调用线程归属，
/// 且不会阻塞主线程的消息泵（阻塞消息泵会延迟 WebView2 的 WM_ACTIVATE 处理，
/// 反而让激活更慢）。
async fn verify_foreground_with_retry(window: &WebviewWindow, hwnd_isize: isize) -> bool {
    const DELAYS_MS: [u64; 3] = [40, 120, 300];

    for delay in DELAYS_MS {
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;

        // 窗口若已被隐藏（用户快速切换/其他逻辑隐藏），放弃本次唤醒
        if !window.is_visible().unwrap_or(false) {
            return false;
        }

        let fg = unsafe { GetForegroundWindow() };
        if fg.0 as isize == hwnd_isize {
            return true;
        }

        // 前台被抢占或前台锁拒绝：ALT 键技巧 + 强置前台
        unsafe {
            let _ = AllowSetForegroundWindow(ASFW_ANY);
            if !is_alt_pressed() {
                send_alt_key_trick();
            }
            let _ = BringWindowToTop(HWND(hwnd_isize as _));
            let _ = SetForegroundWindow(HWND(hwnd_isize as _));
        }
    }

    unsafe { GetForegroundWindow().0 as isize == hwnd_isize }
}

/// 确定性前台激活配方（同步段，主线程调用）。
///
/// 组合两种已被广泛验证的技术来绕过 Windows 的前台锁：
/// - `SendInput` 模拟一次 ALT 键按下/抬起：让系统认为本进程刚收到用户输入，
///   从而解除 SetForegroundWindow 的前台锁限制（PowerToys Run、Flow Launcher
///   等启动器均使用此技巧）。
/// - `AttachThreadInput` 将本窗口线程临时附着到当前前台线程：绕过"只有前台进程
///   才能设置前台"的限制。
fn activate_window(hwnd_isize: isize) {
    let hwnd_val = HWND(hwnd_isize as _);

    unsafe {
        let _ = AllowSetForegroundWindow(ASFW_ANY);

        let foreground_hwnd = GetForegroundWindow();
        let foreground_thread = GetWindowThreadProcessId(foreground_hwnd, None);
        let window_thread = GetWindowThreadProcessId(hwnd_val, None);

        if foreground_thread != window_thread && foreground_thread != 0 {
            let _ = AttachThreadInput(window_thread, foreground_thread, true);

            if !is_alt_pressed() {
                send_alt_key_trick();
            }
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

/// 检查物理 Alt 键当前是否处于按下状态
fn is_alt_pressed() -> bool {
    unsafe {
        (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(VK_MENU.0 as i32) as i16) < 0
    }
}

/// 模拟一次 ALT 键按下并抬起（虚拟键码 VK_MENU）。
///
/// 这是绕过 Windows 前台锁的经典技巧：系统在处理键盘输入时会放宽
/// SetForegroundWindow 的限制，模拟一次按键后，本进程即被允许设置前台窗口。
fn send_alt_key_trick() {
    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_MENU,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_MENU,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];

    unsafe {
        let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

/// 在 WebView2 子窗口树中查找键盘焦点落点并对其 SetFocus。
///
/// Tauri 的 `WebviewWindow::set_focus()` 实际聚焦的是 tao 原生窗口，
/// 键盘焦点无法传递到 WebView2 子控件。通过 `EnumChildWindows` 遍历
/// 整个子窗口树（无需递归实现），优先聚焦 WebView2 的渲染宿主
/// `Chrome_RenderWidgetHostHWND`，其次回退到任何 WebView/Edge 相关类名。
///
/// WebView2 子窗口可能运行在不同线程上，SetFocus 前需要 AttachThreadInput
/// 临时附着以允许跨线程聚焦。
fn focus_webview_child(parent_isize: isize) -> bool {
    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let target = lparam.0 as *mut AtomicBool;
        let mut buf = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut buf);
        if len > 0 {
            let name = String::from_utf16_lossy(&buf[..len as usize]);
            // 优先 WebView2 渲染宿主；其次任何 WebView/Edge 相关类名
            let is_target = name == "Chrome_RenderWidgetHostHWND"
                || name.starts_with("Chrome_WidgetWin_")
                || name.contains("WebView")
                || name.contains("Edge");
            if is_target && focus_one(hwnd) {
                (*target).store(true, Ordering::Relaxed);
                return BOOL(0); // 停止枚举
            }
        }
        BOOL(1) // 继续枚举
    }

    unsafe {
        let target = AtomicBool::new(false);
        let _ = EnumChildWindows(
            HWND(parent_isize as _),
            Some(enum_proc),
            LPARAM(&target as *const AtomicBool as isize),
        );
        target.load(Ordering::Relaxed)
    }
}

/// 对指定 HWND 调用 SetFocus，必要时先 AttachThreadInput 以跨线程聚焦。
unsafe fn focus_one(hwnd: HWND) -> bool {
    let target_thread = GetWindowThreadProcessId(hwnd, None);
    let current_thread = GetCurrentThreadId();
    let need_detach = target_thread != 0 && target_thread != current_thread;

    if need_detach {
        let _ = AttachThreadInput(current_thread, target_thread, true);
    }

    let result = SetFocus(hwnd);

    if need_detach {
        let _ = AttachThreadInput(current_thread, target_thread, false);
    }

    result.0 != 0
}
