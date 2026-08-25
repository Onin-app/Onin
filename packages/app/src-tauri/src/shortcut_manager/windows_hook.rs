//! Windows 低级键盘钩子：拦截"按住 Alt 再按 Space"的系统键路径
//!
//! ## 背景
//! 全局快捷键 Alt+Space 通过 `RegisterHotKey(MOD_ALT|VK_SPACE, MOD_NOREPEAT)` 注册。
//! 当主窗口已获得焦点后，**按住 Alt 再按 Space** 时 Windows 不会再触发 `WM_HOTKEY`
//! （`MOD_NOREPEAT` 关闭了重复触发），按键以 `WM_SYSKEYDOWN` 直接派发给前台窗口
//! （WebView2）。实测该按键不会以可拦截的 DOM keydown 形式到达页面——空格字符由
//! 浏览器进程直接插入输入框，前端 `preventDefault` 无效。
//!
//! 因此在 OS 层用 `WH_KEYBOARD_LL` 钩子拦截该组合，仅在满足以下条件时消费按键并
//! 触发与全局快捷键一致的隐藏逻辑（`request_hide`）：
//! 1. 按下的键是空格（`VK_SPACE`）且 Alt 处于按下状态（`LLKHF_ALTDOWN`）
//! 2. 前台窗口属于本进程（避免劫持其他应用的 Alt+Space 系统菜单）
//! 3. 主窗口当前可见（隐藏状态下放行，交给 `RegisterHotKey` 完成显示）
//! 4. "显示/隐藏窗口"快捷键确实配置为 Alt+Space
//!
//! 窗口隐藏时钩子一律放行，保证首次 Alt+Space 仍能通过 `WM_HOTKEY` 显示窗口。

use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SPACE;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId,
    SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_SYSKEYDOWN,
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

unsafe extern "system" fn keyboard_proc(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode >= 0 {
        let msg = wparam.0 as u32;
        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
            let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            // 1) 空格键且 Alt 处于按下状态（系统键路径）
            if kbd.vkCode == VK_SPACE.0 as u32 && (kbd.flags.0 & LLKHF_ALTDOWN) != 0 {
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

                    // 3) 仅当本进程处于前台（避免劫持其他应用的 Alt+Space 系统菜单）
                    let foreground = unsafe { GetForegroundWindow() };
                    let ours = foreground.0 != 0 && is_our_process(foreground);
                    if !ours {
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

fn is_our_process(hwnd: HWND) -> bool {
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid != 0 && pid == std::process::id()
}

fn toggle_is_alt_space(app: &AppHandle) -> bool {
    let state: State<ShortcutState> = app.state();
    // 绑定到具名局部变量：match 直接锁取的临时值（持有 MutexGuard 的 Result）
    // 会被延长到函数末尾才析构，晚于 `state` 的 drop，导致借用超期（E0597）。
    let lock_result = state.shortcuts.lock();
    match lock_result {
        Ok(shortcuts) => shortcuts.iter().any(|s| {
            s.command_name == "toggle_window"
                && normalize_shortcut_string(&s.shortcut) == "alt+space"
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
