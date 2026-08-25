//! Windows 低级键盘钩子：拦截"按住 Alt 再按 Space"的系统键路径
//!
//! ## 背景
//! 全局快捷键 Alt+Space 通过 `RegisterHotKey(MOD_ALT|VK_SPACE, MOD_NOREPEAT)` 注册。
//! 当主窗口已获得焦点后，**按住 Alt 再按 Space** 时 Windows 不会再触发 `WM_HOTKEY`
//! （`MOD_NOREPEAT` 关闭了重复触发），按键以 `WM_SYSKEYDOWN` 直接派发给前台窗口
//! （WebView2）。实测该按键不会以可拦截的 DOM keydown 形式到达页面——空格字符由
//! 浏览器进程直接插入输入框，前端 `preventDefault` 无效。
//!
//! 因此在 OS 层用 `WH_KEYBOARD_LL` 钩子拦截该组合。实测前台窗口是 WebView2
//! 浏览器进程（msedgewebview2.exe）的 `Chrome_WidgetWin_1` 窗口，既不属于本进程
//! 也不是本窗口的后代，因此"前台判定"采用三重检查：同进程 / 主窗口祖先链 /
//! 前台窗口矩形位于主窗口矩形内部（WebView2 窗口恰好覆盖主窗口客户区）。
//!
//! 消费条件（全部满足才拦截并触发 `request_hide`）：
//! 1. 空格键 + Alt 按下（`LLKHF_ALTDOWN`）
//! 2. 主窗口当前可见（隐藏状态下放行，交给 `RegisterHotKey` 显示）
//! 3. 前台窗口属于本应用（上述三重检查）
//! 4. 非快捷键录制中（设置页录制 Alt+Space 时放行）
//! 5. "显示/隐藏窗口"快捷键配置为 Alt+Space
//!
//! ## 性能与稳定性
//! `WH_KEYBOARD_LL` 回调期间系统键盘处理会暂停，回调必须极简且**禁止阻塞**
//! （不得在回调内写日志/格式化/做重活），否则键盘输入会全局卡顿。

use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_MENU, VK_SPACE};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, GetMessageW, GetParent, GetWindowRect,
    GetWindowThreadProcessId, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

use super::state::ShortcutState;
use super::utils::normalize_shortcut_string;

static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();
static HOOK: Mutex<Option<HHOOK>> = Mutex::new(None);

/// 设置页快捷键录制输入框聚焦期间置位，钩子跳过 Alt+Space（允许录制该组合）
static RECORDING: AtomicBool = AtomicBool::new(false);

/// `KBDLLHOOKSTRUCT.flags` 中表示 Alt 处于按下状态的标志位（LLKHF_ALTDOWN = 0x20）
const LLKHF_ALTDOWN: u32 = 0x20;

/// 设置快捷键录制状态（由设置页快捷键输入框的 focus/blur 调用）
pub fn set_recording(active: bool) {
    RECORDING.store(active, Ordering::Relaxed);
}

/// 前台窗口是否属于本应用（三重检查）：
/// 1. 前台窗口与本进程同 PID
/// 2. 前台窗口（或其后代链）是主窗口
/// 3. 前台窗口矩形位于主窗口矩形内部——WebView2 浏览器进程（msedgewebview2.exe）
///    的前台窗口 `Chrome_WidgetWin_1` 既不同进程也非本窗口后代，但它恰好覆盖
///    在主窗口客户区上，用矩形包含关系识别
fn is_foreground_ours(main_hwnd: HWND) -> bool {
    let fg = unsafe { GetForegroundWindow() };
    if fg.0 == 0 {
        return false;
    }

    let mut fg_pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(fg, Some(&mut fg_pid)) };
    if fg_pid == std::process::id() {
        return true;
    }

    // 从前景窗口向上回溯祖先链
    let mut current = fg;
    let mut guard = 0;
    while current.0 != 0 && guard < 32 {
        if current == main_hwnd {
            return true;
        }
        let parent = unsafe { GetParent(current) };
        if parent.0 == 0 {
            break;
        }
        current = parent;
        guard += 1;
    }

    // 矩形包含检查（覆盖 WebView2 跨进程窗口）
    if main_hwnd.0 != 0 {
        let mut fg_rect = RECT::default();
        let mut main_rect = RECT::default();
        let fg_ok = unsafe { GetWindowRect(fg, &mut fg_rect) }.is_ok();
        let main_ok = unsafe { GetWindowRect(main_hwnd, &mut main_rect) }.is_ok();
        if fg_ok && main_ok {
            if fg_rect.left >= main_rect.left
                && fg_rect.top >= main_rect.top
                && fg_rect.right <= main_rect.right
                && fg_rect.bottom <= main_rect.bottom
            {
                return true;
            }
        }
    }

    false
}

unsafe extern "system" fn keyboard_proc(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode >= 0 {
        let msg = wparam.0 as u32;
        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
            let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            // 1) 空格键且 Alt 处于按下状态（系统键路径）。
            //    注意：显示窗口时 focus_manager 会用 SendInput 注入一次 ALT 抬起，
            //    可能把系统 Alt 状态复位（用户物理上仍按着 Alt），导致 LLKHF_ALTDOWN
            //    丢失，因此用 GetAsyncKeyState 的物理状态作兜底。
            let alt_down = (kbd.flags.0 & LLKHF_ALTDOWN) != 0
                || unsafe { (GetAsyncKeyState(VK_MENU.0 as i32) as i16) < 0 };
            if kbd.vkCode == VK_SPACE.0 as u32 && alt_down {
                if let Some(app) = APP_HANDLE.get() {
                    // 2) 主窗口可见时才有"隐藏"语义；隐藏状态下放行并自愈残留的录制标志，
                    //    让首次 Alt+Space 交给 RegisterHotKey 完成显示
                    let main_visible = app
                        .get_webview_window("main")
                        .map(|w| w.is_visible().unwrap_or(false))
                        .unwrap_or(false);
                    if !main_visible {
                        RECORDING.store(false, Ordering::Relaxed);
                        return unsafe { CallNextHookEx(None, ncode, wparam, lparam) };
                    }

                    // 3) 前台窗口属于本应用（避免劫持其他应用的 Alt+Space 系统菜单）
                    let main_hwnd = app
                        .get_webview_window("main")
                        .and_then(|w| w.hwnd().ok())
                        .map(|h| HWND(h.0 as isize))
                        .unwrap_or(HWND(0));
                    if !is_foreground_ours(main_hwnd) {
                        // 窗口失焦（可能未触发 DOM blur）时同样自愈录制标志
                        RECORDING.store(false, Ordering::Relaxed);
                        return unsafe { CallNextHookEx(None, ncode, wparam, lparam) };
                    }

                    // 4) 设置页录制快捷键期间放行，允许把 Alt+Space 录制成快捷键
                    if RECORDING.load(Ordering::Relaxed) {
                        return unsafe { CallNextHookEx(None, ncode, wparam, lparam) };
                    }

                    // 5) 切换快捷键确实配置为 Alt+Space 时才拦截
                    if toggle_is_alt_space(app) {
                        let app2 = app.clone();
                        let _ = app.run_on_main_thread(move || {
                            crate::window_manager::request_hide(&app2);
                        });
                        // 消费按键：空格不会落入输入框，也不会触发其他处理
                        return LRESULT(1);
                    }
                }
            }
        }
    }
    unsafe { CallNextHookEx(None, ncode, wparam, lparam) }
}

fn toggle_is_alt_space(app: &AppHandle) -> bool {
    let state: State<ShortcutState> = app.state();
    // 绑定到具名局部变量：match 直接锁取的临时值（持有 MutexGuard 的 Result）
    // 会被延长到函数末尾才析构，晚于 `state` 的 drop，导致借用超期（E0597）。
    let lock_result = state.shortcuts.lock();
    match lock_result {
        Ok(shortcuts) => shortcuts.iter().any(|s| {
            s.command_name == "toggle_window"
                && normalize_shortcut_string(&s.shortcut).eq_ignore_ascii_case("alt+space")
        }),
        Err(_) => false,
    }
}

/// 安装低级键盘钩子（仅 Windows）。钩子运行在独立线程的消息循环中。
pub fn install(app: &AppHandle) {
    if HOOK.lock().map(|guard| guard.is_some()).unwrap_or(true) {
        return; // 已安装
    }
    let _ = APP_HANDLE.set(app.clone());

    std::thread::spawn(|| unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0);
        if let Ok(h) = hook {
            *HOOK.lock().unwrap() = Some(h);
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {}
            // 线程退出时注销钩子（进程退出前不会走到这里）
            if let Some(h_val) = HOOK.lock().unwrap().take() {
                let _ = UnhookWindowsHookEx(h_val);
            }
        } else {
            eprintln!("[shortcut_manager] Failed to install WH_KEYBOARD_LL hook");
        }
    });
}
