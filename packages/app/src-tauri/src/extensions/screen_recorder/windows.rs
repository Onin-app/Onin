use super::engine::{RecordConfig, RecordState, RecordStateSnapshot, ScreenRecordEngine};
use once_cell::sync::Lazy;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush, DeleteDC,
    DeleteObject, FillRect, GetDC, GetDIBits, GetMonitorInfoW, GetTextExtentPoint32W,
    MonitorFromPoint, ReleaseDC, SelectObject, SetBkMode, SetTextColor, TextOutW, BACKGROUND_MODE,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    DEFAULT_PITCH, DIB_USAGE, FF_DONTCARE, FW_BOLD, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONULL,
    OUT_DEFAULT_PRECIS,
};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LCONTROL,
    VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_NEXT, VK_OEM_1, VK_OEM_2, VK_OEM_3,
    VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS,
    VK_PRIOR, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT, VK_SPACE,
    VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
};

use windows_capture::{
    capture::{CaptureControl, Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
    window::Window,
};

// 辅助打包函数，用于设置 MF_MT_FRAME_SIZE 和 MF_MT_FRAME_RATE
fn pack_u32_to_u64(high: u32, low: u32) -> u64 {
    ((high as u64) << 32) | (low as u64)
}

fn get_monitor_phys_pos(hmonitor: *mut std::ffi::c_void) -> (i32, i32) {
    unsafe {
        let mut info = MONITORINFO::default();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(HMONITOR(hmonitor as isize), &mut info).as_bool() {
            (info.rcMonitor.left, info.rcMonitor.top)
        } else {
            (0, 0)
        }
    }
}

// ============================================================================
// 全局键盘 Hook 逻辑与状态管理
// ============================================================================

#[derive(Clone)]
struct KeyState {
    text: String,
    last_update: Instant,
}

static KEYBOARD_STATE: Lazy<Mutex<Option<KeyState>>> = Lazy::new(|| Mutex::new(None));
static KEYBOARD_HOOK: Mutex<Option<HHOOK>> = Mutex::new(None);
static HOOK_THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn is_key_pressed(vk: i32) -> bool {
    unsafe { (GetKeyState(vk) as u16 & 0x8000) != 0 }
}

fn parse_vk_code(vk: u32) -> Option<String> {
    let is_ctrl = is_key_pressed(VK_CONTROL.0 as i32)
        || is_key_pressed(VK_LCONTROL.0 as i32)
        || is_key_pressed(VK_RCONTROL.0 as i32);
    let is_shift = is_key_pressed(VK_SHIFT.0 as i32)
        || is_key_pressed(VK_LSHIFT.0 as i32)
        || is_key_pressed(VK_RSHIFT.0 as i32);
    let is_alt = is_key_pressed(VK_MENU.0 as i32)
        || is_key_pressed(VK_LMENU.0 as i32)
        || is_key_pressed(VK_RMENU.0 as i32);
    let is_win = is_key_pressed(VK_LWIN.0 as i32) || is_key_pressed(VK_RWIN.0 as i32);

    if vk == VK_CONTROL.0 as u32
        || vk == VK_LCONTROL.0 as u32
        || vk == VK_RCONTROL.0 as u32
        || vk == VK_SHIFT.0 as u32
        || vk == VK_LSHIFT.0 as u32
        || vk == VK_RSHIFT.0 as u32
        || vk == VK_MENU.0 as u32
        || vk == VK_LMENU.0 as u32
        || vk == VK_RMENU.0 as u32
        || vk == VK_LWIN.0 as u32
        || vk == VK_RWIN.0 as u32
    {
        return None;
    }

    let main_key = match vk {
        0x30..=0x39 => ((vk - 0x30) as u8 as char).to_string(),
        0x41..=0x5A => ((vk - 0x41 + 65) as u8 as char).to_string(),
        0x60..=0x69 => format!("Num {}", vk - 0x60),
        0x70..=0x7B => format!("F{}", vk - 0x70 + 1),
        v if v == VK_BACK.0 as u32 => "Backspace".to_string(),
        v if v == VK_TAB.0 as u32 => "Tab".to_string(),
        v if v == VK_RETURN.0 as u32 => "Enter".to_string(),
        v if v == VK_ESCAPE.0 as u32 => "Esc".to_string(),
        v if v == VK_SPACE.0 as u32 => "Space".to_string(),
        v if v == VK_PRIOR.0 as u32 => "PgUp".to_string(),
        v if v == VK_NEXT.0 as u32 => "PgDn".to_string(),
        v if v == VK_END.0 as u32 => "End".to_string(),
        v if v == VK_HOME.0 as u32 => "Home".to_string(),
        v if v == VK_LEFT.0 as u32 => "Left".to_string(),
        v if v == VK_UP.0 as u32 => "Up".to_string(),
        v if v == VK_RIGHT.0 as u32 => "Right".to_string(),
        v if v == VK_DOWN.0 as u32 => "Down".to_string(),
        v if v == VK_DELETE.0 as u32 => "Del".to_string(),
        v if v == VK_OEM_1.0 as u32 => ";".to_string(),
        v if v == VK_OEM_PLUS.0 as u32 => "+".to_string(),
        v if v == VK_OEM_COMMA.0 as u32 => ",".to_string(),
        v if v == VK_OEM_MINUS.0 as u32 => "-".to_string(),
        v if v == VK_OEM_PERIOD.0 as u32 => ".".to_string(),
        v if v == VK_OEM_2.0 as u32 => "/".to_string(),
        v if v == VK_OEM_3.0 as u32 => "`".to_string(),
        v if v == VK_OEM_4.0 as u32 => "[".to_string(),
        v if v == VK_OEM_5.0 as u32 => "\\".to_string(),
        v if v == VK_OEM_6.0 as u32 => "]".to_string(),
        v if v == VK_OEM_7.0 as u32 => "'".to_string(),
        _ => return None,
    };

    let mut combo = Vec::new();
    if is_ctrl {
        combo.push("Ctrl");
    }
    if is_shift {
        combo.push("Shift");
    }
    if is_alt {
        combo.push("Alt");
    }
    if is_win {
        combo.push("Win");
    }
    combo.push(&main_key);

    Some(combo.join(" + "))
}

unsafe extern "system" fn low_level_keyboard_proc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if ncode >= 0 {
        let event_type = wparam.0 as u32;
        if event_type == WM_KEYDOWN || event_type == WM_SYSKEYDOWN {
            let kbd_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;

            if let Some(key_str) = parse_vk_code(vk_code) {
                let mut state_guard = KEYBOARD_STATE.lock().unwrap();
                *state_guard = Some(KeyState {
                    text: key_str,
                    last_update: Instant::now(),
                });
            }
        }
    }
    CallNextHookEx(None, ncode, wparam, lparam)
}

fn start_keyboard_hook() {
    if KEYBOARD_HOOK.lock().unwrap().is_some() {
        return;
    }

    {
        let mut state_guard = KEYBOARD_STATE.lock().unwrap();
        *state_guard = None;
    }

    std::thread::spawn(|| unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), None, 0);
        if let Ok(h) = hook {
            *KEYBOARD_HOOK.lock().unwrap() = Some(h);
            HOOK_THREAD_ID.store(
                windows::Win32::System::Threading::GetCurrentThreadId(),
                Ordering::SeqCst,
            );

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {}
        }
    });
}

fn stop_keyboard_hook() {
    let hook = KEYBOARD_HOOK.lock().unwrap().take();
    if let Some(h) = hook {
        unsafe {
            let _ = UnhookWindowsHookEx(h);
        }
    }
    let tid = HOOK_THREAD_ID.swap(0, Ordering::SeqCst);
    if tid != 0 {
        unsafe {
            let _ = PostThreadMessageW(tid, WM_QUIT, None, None);
        }
    }
}

struct Ripple {
    x: i32,
    y: i32,
    start_time: Instant,
    is_right: bool,
}

struct MFSinkWriterWrapper {
    sink_writer: IMFSinkWriter,
    video_stream_index: u32,
    width: u32,
    height: u32,
    fps: u32,
    start_time: Option<Instant>,
    rect_x: u32,
    rect_y: u32,
    show_mouse_click: bool,
    show_keys: bool,
    is_window_mode: bool,
    window_handle: Option<isize>,
    monitor_x: i32,
    monitor_y: i32,
    last_lbutton: bool,
    last_rbutton: bool,
    ripples: Vec<Ripple>,
}

unsafe impl Send for MFSinkWriterWrapper {}
unsafe impl Sync for MFSinkWriterWrapper {}

impl MFSinkWriterWrapper {
    fn new(
        output_path: &Path,
        width: u32,
        height: u32,
        fps: u32,
        rect_x: u32,
        rect_y: u32,
        config: &RecordConfig,
        is_window_mode: bool,
        window_handle: Option<isize>,
        monitor_x: i32,
        monitor_y: i32,
    ) -> Result<Self, String> {
        unsafe {
            MFStartup(MF_VERSION, MFSTARTUP_FULL)
                .map_err(|e| format!("MFStartup failed: {:?}", e))?;

            let path_u16: Vec<u16> = output_path
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            let path_pcwstr = PCWSTR(path_u16.as_ptr());

            let sink_writer = MFCreateSinkWriterFromURL(path_pcwstr, None, None)
                .map_err(|e| format!("MFCreateSinkWriterFromURL failed: {:?}", e))?;

            let out_media_type =
                MFCreateMediaType().map_err(|e| format!("MFCreateMediaType failed: {:?}", e))?;
            out_media_type
                .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
                .map_err(|e| format!("SetGUID MajorType failed: {:?}", e))?;
            out_media_type
                .SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)
                .map_err(|e| format!("SetGUID Subtype H264 failed: {:?}", e))?;
            out_media_type
                .SetUINT32(&MF_MT_AVG_BITRATE, 5_000_000)
                .map_err(|e| format!("SetUINT32 AvgBitRate failed: {:?}", e))?;
            out_media_type
                .SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
                .map_err(|e| format!("SetUINT32 InterlaceMode failed: {:?}", e))?;
            out_media_type
                .SetUINT64(&MF_MT_FRAME_SIZE, pack_u32_to_u64(width, height))
                .map_err(|e| format!("SetUINT64 FrameSize failed: {:?}", e))?;
            out_media_type
                .SetUINT64(&MF_MT_FRAME_RATE, pack_u32_to_u64(fps, 1))
                .map_err(|e| format!("SetUINT64 FrameRate failed: {:?}", e))?;

            let in_media_type =
                MFCreateMediaType().map_err(|e| format!("MFCreateMediaType failed: {:?}", e))?;
            in_media_type
                .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
                .map_err(|e| format!("SetGUID Input MajorType failed: {:?}", e))?;
            in_media_type
                .SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)
                .map_err(|e| format!("SetGUID Input Subtype RGB32 failed: {:?}", e))?;
            in_media_type
                .SetUINT64(&MF_MT_FRAME_SIZE, pack_u32_to_u64(width, height))
                .map_err(|e| format!("SetUINT64 Input FrameSize failed: {:?}", e))?;
            in_media_type
                .SetUINT64(&MF_MT_FRAME_RATE, pack_u32_to_u64(fps, 1))
                .map_err(|e| format!("SetUINT64 Input FrameRate failed: {:?}", e))?;

            let video_stream_index = sink_writer
                .AddStream(&out_media_type)
                .map_err(|e| format!("AddStream failed: {:?}", e))?;
            sink_writer
                .SetInputMediaType(video_stream_index, &in_media_type, None)
                .map_err(|e| format!("SetInputMediaType failed: {:?}", e))?;

            sink_writer
                .BeginWriting()
                .map_err(|e| format!("BeginWriting failed: {:?}", e))?;

            Ok(Self {
                sink_writer,
                video_stream_index,
                width,
                height,
                fps,
                start_time: None,
                rect_x,
                rect_y,
                show_mouse_click: config.show_mouse_click,
                show_keys: config.show_keys,
                is_window_mode,
                window_handle,
                monitor_x,
                monitor_y,
                last_lbutton: false,
                last_rbutton: false,
                ripples: Vec::new(),
            })
        }
    }

    fn write_frame(
        &mut self,
        frame_data: &[u8],
        frame_width: u32,
        frame_height: u32,
        timestamp: Instant,
    ) -> Result<(), String> {
        unsafe {
            if self.start_time.is_none() {
                self.start_time = Some(timestamp);
            }
            let elapsed = timestamp.duration_since(self.start_time.unwrap_or(timestamp));

            let buffer_len = self.width * self.height * 4;
            let buffer = MFCreateMemoryBuffer(buffer_len)
                .map_err(|e| format!("MFCreateMemoryBuffer failed: {:?}", e))?;

            let mut p_data: *mut u8 = std::ptr::null_mut();
            let mut max_length = 0u32;
            let mut current_length = 0u32;
            buffer
                .Lock(
                    &mut p_data,
                    Some(&mut max_length),
                    Some(&mut current_length),
                )
                .map_err(|e| format!("Buffer Lock failed: {:?}", e))?;

            std::ptr::write_bytes(p_data, 0, buffer_len as usize);

            let src_row_pitch = (frame_width * 4) as usize;
            let dest_row_pitch = (self.width * 4) as usize;
            let copy_width_bytes = (self.width * 4) as usize;

            for y in 0..self.height {
                let src_y = frame_height as i32 - 1 - (self.rect_y + y) as i32;
                if src_y >= 0 && src_y < frame_height as i32 {
                    let src_offset = (src_y as usize) * src_row_pitch + (self.rect_x as usize * 4);
                    let dest_offset = (y as usize) * dest_row_pitch;

                    if src_offset + copy_width_bytes <= frame_data.len()
                        && dest_offset + copy_width_bytes <= buffer_len as usize
                    {
                        std::ptr::copy_nonoverlapping(
                            frame_data.as_ptr().add(src_offset),
                            p_data.add(dest_offset),
                            copy_width_bytes,
                        );
                    }
                }
            }

            // 软渲染定制效果 overlays
            if self.show_mouse_click {
                self.draw_mouse_clicks(p_data);
            }

            if self.show_keys {
                self.draw_keys_overlay(p_data);
            }

            buffer
                .SetCurrentLength(buffer_len)
                .map_err(|e| format!("SetCurrentLength failed: {:?}", e))?;
            buffer
                .Unlock()
                .map_err(|e| format!("Buffer Unlock failed: {:?}", e))?;

            let sample = MFCreateSample().map_err(|e| format!("MFCreateSample failed: {:?}", e))?;
            sample
                .AddBuffer(&buffer)
                .map_err(|e| format!("AddBuffer failed: {:?}", e))?;

            let sample_time = (elapsed.as_nanos() / 100) as i64;
            sample
                .SetSampleTime(sample_time)
                .map_err(|e| format!("SetSampleTime failed: {:?}", e))?;
            let sample_duration = (1_000_000_000 / self.fps) as i64 / 100;
            sample
                .SetSampleDuration(sample_duration)
                .map_err(|e| format!("SetSampleDuration failed: {:?}", e))?;

            self.sink_writer
                .WriteSample(self.video_stream_index, &sample)
                .map_err(|e| format!("WriteSample failed: {:?}", e))?;

            Ok(())
        }
    }

    fn draw_mouse_clicks(&mut self, p_data: *mut u8) {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON,
        };

        let l_down = unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 };
        let r_down = unsafe { (GetAsyncKeyState(VK_RBUTTON.0 as i32) as u16 & 0x8000) != 0 };

        let mut point = POINT::default();
        let got_pos =
            unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point).is_ok() };

        if got_pos {
            let (rx, ry) = if self.is_window_mode {
                let mut rect = windows::Win32::Foundation::RECT::default();
                let mut rx = -1;
                let mut ry = -1;
                if let Some(hwnd) = self.window_handle {
                    unsafe {
                        if windows::Win32::UI::WindowsAndMessaging::GetWindowRect(
                            windows::Win32::Foundation::HWND(hwnd),
                            &mut rect,
                        )
                        .is_ok()
                        {
                            let rect_w = rect.right - rect.left;
                            let rect_h = rect.bottom - rect.top;
                            let win_scale_x = if rect_w > 0 {
                                self.width as f32 / rect_w as f32
                            } else {
                                1.0
                            };
                            let win_scale_y = if rect_h > 0 {
                                self.height as f32 / rect_h as f32
                            } else {
                                1.0
                            };

                            rx = ((point.x - rect.left) as f32 * win_scale_x) as i32;
                            // 窗口模式同样需要 Y 轴反向映射
                            ry = (self.height as i32 - 1)
                                - ((point.y - rect.top) as f32 * win_scale_y) as i32;
                        }
                    }
                }
                (rx, ry)
            } else {
                let rx = point.x - self.monitor_x - self.rect_x as i32;
                let ry = (self.height as i32 - 1) - (point.y - self.monitor_y - self.rect_y as i32);
                (rx, ry)
            };

            if rx >= 0 && rx < self.width as i32 && ry >= 0 && ry < self.height as i32 {
                if l_down && !self.last_lbutton {
                    self.ripples.push(Ripple {
                        x: rx,
                        y: ry,
                        start_time: Instant::now(),
                        is_right: false,
                    });
                }
                if r_down && !self.last_rbutton {
                    self.ripples.push(Ripple {
                        x: rx,
                        y: ry,
                        start_time: Instant::now(),
                        is_right: true,
                    });
                }
            }
        }

        self.last_lbutton = l_down;
        self.last_rbutton = r_down;

        let now = Instant::now();
        // 点击残留维持 400 毫秒，使视觉焦点更明显且驻留时间更优
        self.ripples
            .retain(|ripple| now.duration_since(ripple.start_time) < Duration::from_millis(400));

        for ripple in &self.ripples {
            let elapsed = now.duration_since(ripple.start_time).as_secs_f32();
            let alpha = 1.0 - (elapsed / 0.4);

            if ripple.is_right {
                // 右击：亮蓝色双同心圆 + 中心实心圆光晕
                let r_max_outer = 34.0;
                let radius_outer = (elapsed / 0.4) * r_max_outer;
                let radius_inner = (elapsed / 0.4) * 20.0;

                let r_out_inner = radius_outer - 1.8;
                let r_out_outer = radius_outer + 1.8;
                let r_in_inner = radius_inner - 1.8;
                let r_in_outer = radius_inner + 1.8;

                // 中心实心光晕大小
                let center_halo_r_sq = 9.0f32 * 9.0f32;

                let r_outer_i = r_max_outer.ceil() as i32;

                for dy in -r_outer_i..=r_outer_i {
                    let py = ripple.y + dy;
                    if py < 0 || py >= self.height as i32 {
                        continue;
                    }
                    let dy_sq = (dy * dy) as f32;
                    for dx in -r_outer_i..=r_outer_i {
                        let px = ripple.x + dx;
                        if px < 0 || px >= self.width as i32 {
                            continue;
                        }

                        let dist_sq = (dx * dx) as f32 + dy_sq;
                        let in_outer_ring = dist_sq >= r_out_inner * r_out_inner
                            && dist_sq <= r_out_outer * r_out_outer;
                        let in_inner_ring = dist_sq >= r_in_inner * r_in_inner
                            && dist_sq <= r_in_outer * r_in_outer;
                        let in_halo = dist_sq <= center_halo_r_sq;

                        if in_outer_ring || in_inner_ring || in_halo {
                            let offset = ((py as u32 * self.width) + px as u32) as usize * 4;
                            let src_r = 60;
                            let src_g = 130;
                            let src_b = 255;

                            let current_alpha = if in_outer_ring || in_inner_ring {
                                alpha
                            } else {
                                alpha * 0.45
                            };

                            unsafe {
                                let dest_b = *p_data.add(offset) as f32;
                                let dest_g = *p_data.add(offset + 1) as f32;
                                let dest_r = *p_data.add(offset + 2) as f32;

                                *p_data.add(offset) = (src_b as f32 * current_alpha
                                    + dest_b * (1.0 - current_alpha))
                                    as u8;
                                *p_data.add(offset + 1) = (src_g as f32 * current_alpha
                                    + dest_g * (1.0 - current_alpha))
                                    as u8;
                                *p_data.add(offset + 2) = (src_r as f32 * current_alpha
                                    + dest_r * (1.0 - current_alpha))
                                    as u8;
                            }
                        }
                    }
                }
            } else {
                // 左击：单亮红圈涟漪（粗圆圈） + 中心实心圆光晕
                let r_max = 28.0;
                let radius = (elapsed / 0.4) * r_max;

                let r_inner = radius - 2.0;
                let r_outer = radius + 2.0;
                let r_inner_sq = r_inner * r_inner;
                let r_outer_sq = r_outer * r_outer;

                // 中心实心光晕大小
                let center_halo_r_sq = 8.0f32 * 8.0f32;

                let r_outer_i = r_max.ceil() as i32;

                for dy in -r_outer_i..=r_outer_i {
                    let py = ripple.y + dy;
                    if py < 0 || py >= self.height as i32 {
                        continue;
                    }
                    let dy_sq = (dy * dy) as f32;
                    for dx in -r_outer_i..=r_outer_i {
                        let px = ripple.x + dx;
                        if px < 0 || px >= self.width as i32 {
                            continue;
                        }

                        let dist_sq = (dx * dx) as f32 + dy_sq;
                        let in_ring = dist_sq >= r_inner_sq && dist_sq <= r_outer_sq;
                        let in_halo = dist_sq <= center_halo_r_sq;

                        if in_ring || in_halo {
                            let offset = ((py as u32 * self.width) + px as u32) as usize * 4;
                            let src_r = 255;
                            let src_g = 60;
                            let src_b = 60;

                            let current_alpha = if in_ring { alpha } else { alpha * 0.45 };

                            unsafe {
                                let dest_b = *p_data.add(offset) as f32;
                                let dest_g = *p_data.add(offset + 1) as f32;
                                let dest_r = *p_data.add(offset + 2) as f32;

                                *p_data.add(offset) = (src_b as f32 * current_alpha
                                    + dest_b * (1.0 - current_alpha))
                                    as u8;
                                *p_data.add(offset + 1) = (src_g as f32 * current_alpha
                                    + dest_g * (1.0 - current_alpha))
                                    as u8;
                                *p_data.add(offset + 2) = (src_r as f32 * current_alpha
                                    + dest_r * (1.0 - current_alpha))
                                    as u8;
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_keys_overlay(&self, p_data: *mut u8) {
        let key_info = {
            let state_guard = KEYBOARD_STATE.lock().unwrap();
            state_guard.clone()
        };

        if let Some(ref info) = key_info {
            let elapsed = info.last_update.elapsed();
            if elapsed < Duration::from_millis(2000) {
                let alpha = if elapsed.as_millis() > 1500 {
                    1.0 - ((elapsed.as_millis() - 1500) as f32 / 500.0)
                } else {
                    1.0
                };

                self.draw_keyboard_overlay(p_data, &info.text, alpha);
            }
        }
    }

    fn draw_keyboard_overlay(&self, p_data: *mut u8, text: &str, alpha: f32) {
        let wide_text: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();

        unsafe {
            let screen_hdc = GetDC(None);
            if screen_hdc.is_invalid() {
                return;
            }
            let mem_hdc = CreateCompatibleDC(screen_hdc);
            if mem_hdc.is_invalid() {
                let _ = ReleaseDC(None, screen_hdc);
                return;
            }

            let font_height = 36;
            let font = CreateFontW(
                font_height,
                0,
                0,
                0,
                FW_BOLD.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET.0 as u32,
                OUT_DEFAULT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32,
                CLEARTYPE_QUALITY.0 as u32,
                DEFAULT_PITCH.0 as u32 | FF_DONTCARE.0 as u32,
                windows::core::w!("Segoe UI"),
            );

            let old_font = SelectObject(mem_hdc, font);

            let mut text_size = windows::Win32::Foundation::SIZE::default();
            let text_len = wide_text.len() - 1;
            if GetTextExtentPoint32W(mem_hdc, &wide_text[..text_len], &mut text_size).as_bool() {
                let text_width = text_size.cx;
                let text_height = text_size.cy;

                let hbmp = CreateCompatibleBitmap(screen_hdc, text_width, text_height);
                if !hbmp.is_invalid() {
                    let old_bmp = SelectObject(mem_hdc, hbmp);

                    let black_brush = CreateSolidBrush(COLORREF(0));
                    let rect = windows::Win32::Foundation::RECT {
                        left: 0,
                        top: 0,
                        right: text_width,
                        bottom: text_height,
                    };
                    FillRect(mem_hdc, &rect, black_brush);
                    let _ = DeleteObject(black_brush);

                    let _ = SetTextColor(mem_hdc, COLORREF(0x00FFFFFF));
                    let _ = SetBkMode(mem_hdc, BACKGROUND_MODE(1));
                    let _ = TextOutW(mem_hdc, 0, 0, &wide_text[..text_len]);

                    let mut bmp_info = BITMAPINFO::default();
                    bmp_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                    bmp_info.bmiHeader.biWidth = text_width;
                    bmp_info.bmiHeader.biHeight = -text_height;
                    bmp_info.bmiHeader.biPlanes = 1;
                    bmp_info.bmiHeader.biBitCount = 32;
                    bmp_info.bmiHeader.biCompression = BI_RGB.0 as u32;

                    let mut bmp_pixels = vec![0u8; (text_width * text_height * 4) as usize];
                    GetDIBits(
                        mem_hdc,
                        hbmp,
                        0,
                        text_height as u32,
                        Some(bmp_pixels.as_mut_ptr() as *mut _),
                        &mut bmp_info,
                        DIB_USAGE(0),
                    );

                    let padding_x = 32;
                    let padding_y = 16;
                    let box_width = text_width + padding_x * 2;
                    let box_height = text_height + padding_y * 2;

                    // 暂时将按键显示放在左下角 (box_x = 70, 距离底部 = 70)
                    let box_x = 70;
                    let box_y = self.height as i32 - box_height - 70;

                    let radius = 16;
                    let bg_opacity = 0.72 * alpha;

                    for y in 0..box_height {
                        // 镜像反转行映射
                        let py = self.height as i32 - 1 - (box_y + y);
                        if py < 0 || py >= self.height as i32 {
                            continue;
                        }
                        for x in 0..box_width {
                            let px = box_x + x;
                            if px < 0 || px >= self.width as i32 {
                                continue;
                            }

                            let mut inside = true;
                            if x < radius && y < radius {
                                let dx = radius - x;
                                let dy = radius - y;
                                if dx * dx + dy * dy > radius * radius {
                                    inside = false;
                                }
                            } else if x >= box_width - radius && y < radius {
                                let dx = x - (box_width - radius);
                                let dy = radius - y;
                                if dx * dx + dy * dy > radius * radius {
                                    inside = false;
                                }
                            } else if x < radius && y >= box_height - radius {
                                let dx = radius - x;
                                let dy = y - (box_height - radius);
                                if dx * dx + dy * dy > radius * radius {
                                    inside = false;
                                }
                            } else if x >= box_width - radius && y >= box_height - radius {
                                let dx = x - (box_width - radius);
                                let dy = y - (box_height - radius);
                                if dx * dx + dy * dy > radius * radius {
                                    inside = false;
                                }
                            }

                            if inside {
                                let dest_offset =
                                    ((py as u32 * self.width) + px as u32) as usize * 4;
                                let factor = 1.0 - bg_opacity;
                                *p_data.add(dest_offset) =
                                    (*p_data.add(dest_offset) as f32 * factor) as u8;
                                *p_data.add(dest_offset + 1) =
                                    (*p_data.add(dest_offset + 1) as f32 * factor) as u8;
                                *p_data.add(dest_offset + 2) =
                                    (*p_data.add(dest_offset + 2) as f32 * factor) as u8;
                            }
                        }
                    }

                    let text_start_x = box_x + padding_x;
                    let text_start_y = box_y + padding_y;

                    for y in 0..text_height {
                        // 镜像反转行映射
                        let py = self.height as i32 - 1 - (text_start_y + y);
                        if py < 0 || py >= self.height as i32 {
                            continue;
                        }
                        for x in 0..text_width {
                            let px = text_start_x + x;
                            if px < 0 || px >= self.width as i32 {
                                continue;
                            }

                            let bmp_offset = ((y * text_width) + x) as usize * 4;
                            let intensity = bmp_pixels[bmp_offset + 2] as f32 / 255.0;

                            if intensity > 0.0 {
                                let dest_offset =
                                    ((py as u32 * self.width) + px as u32) as usize * 4;
                                let current_alpha = intensity * alpha;

                                let dest_b = *p_data.add(dest_offset) as f32;
                                let dest_g = *p_data.add(dest_offset + 1) as f32;
                                let dest_r = *p_data.add(dest_offset + 2) as f32;

                                *p_data.add(dest_offset) =
                                    (255.0 * current_alpha + dest_b * (1.0 - current_alpha)) as u8;
                                *p_data.add(dest_offset + 1) =
                                    (255.0 * current_alpha + dest_g * (1.0 - current_alpha)) as u8;
                                *p_data.add(dest_offset + 2) =
                                    (255.0 * current_alpha + dest_r * (1.0 - current_alpha)) as u8;
                            }
                        }
                    }

                    SelectObject(mem_hdc, old_bmp);
                    let _ = DeleteObject(hbmp);
                }
            }

            SelectObject(mem_hdc, old_font);
            let _ = DeleteObject(font);
            let _ = DeleteDC(mem_hdc);
            let _ = ReleaseDC(None, screen_hdc);
        }
    }

    fn finalize(&self) -> Result<(), String> {
        unsafe {
            self.sink_writer
                .Finalize()
                .map_err(|e| format!("SinkWriter Finalize failed: {:?}", e))?;
            let _ = MFShutdown();
            Ok(())
        }
    }
}

// ============================================================================
// windows-capture 事件处理器
// ============================================================================

struct CaptureHandler {
    writer: Arc<Mutex<Option<MFSinkWriterWrapper>>>,
    is_paused: Arc<AtomicBool>,
}

impl GraphicsCaptureApiHandler for CaptureHandler {
    type Flags = (Arc<Mutex<Option<MFSinkWriterWrapper>>>, Arc<AtomicBool>);
    type Error = String;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let (writer, is_paused) = ctx.flags;
        Ok(Self { writer, is_paused })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        // 如果处于暂停状态，直接跳过帧
        if self.is_paused.load(Ordering::SeqCst) {
            return Ok(());
        }

        let frame_width = frame.width();
        let frame_height = frame.height();

        // 获取视频去填充后的原始像素 (BGRA 格式)
        let mut frame_buffer = frame
            .buffer()
            .map_err(|e| format!("Get frame buffer failed: {:?}", e))?;
        let no_padding_slice = frame_buffer
            .as_nopadding_buffer()
            .map_err(|e| format!("Remove padding failed: {:?}", e))?;

        let mut writer_guard = self.writer.lock().unwrap();
        if let Some(ref mut writer) = *writer_guard {
            writer.write_frame(no_padding_slice, frame_width, frame_height, Instant::now())?;
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// ============================================================================
// Windows 录屏引擎实现
// ============================================================================

pub struct WindowsRecordEngine {
    state: Arc<Mutex<RecordState>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    accumulated_duration: Arc<Mutex<Duration>>,
    writer: Arc<Mutex<Option<MFSinkWriterWrapper>>>,
    is_paused: Arc<AtomicBool>,
    capture_thread_control: Arc<Mutex<Option<CaptureControl<CaptureHandler, String>>>>,
}

impl WindowsRecordEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RecordState::Idle)),
            start_time: Arc::new(Mutex::new(None)),
            accumulated_duration: Arc::new(Mutex::new(Duration::ZERO)),
            writer: Arc::new(Mutex::new(None)),
            is_paused: Arc::new(AtomicBool::new(false)),
            capture_thread_control: Arc::new(Mutex::new(None)),
        }
    }
}

impl ScreenRecordEngine for WindowsRecordEngine {
    fn start(
        &self,
        config: &RecordConfig,
        output_path: &Path,
        app: &tauri::AppHandle,
    ) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if *state != RecordState::Idle {
            return Err("Engine is already recording or paused".into());
        }

        // 1. 判断并获取捕获目标以及尺寸
        let is_window_mode = config.record_target_type.as_deref() == Some("window")
            && config.window_handle.is_some();
        let is_area_mode =
            config.record_target_type.as_deref() == Some("area") && config.area_rect.is_some();

        let (width, height, rect_x, rect_y) = if is_area_mode {
            let area = config.area_rect.as_ref().ok_or("No area rect provided")?;
            let idx = config.monitor_index.unwrap_or(0);
            let tauri_monitors = app.available_monitors().unwrap_or_default();
            let scale_factor = if idx >= 0 && (idx as usize) < tauri_monitors.len() {
                tauri_monitors[idx as usize].scale_factor()
            } else {
                1.0
            };

            let p_x = (area.x as f64 * scale_factor).round() as u32;
            let p_y = (area.y as f64 * scale_factor).round() as u32;
            let p_w = (area.width as f64 * scale_factor).round() as u32;
            let p_h = (area.height as f64 * scale_factor).round() as u32;

            (p_w, p_h, p_x, p_y)
        } else if is_window_mode {
            let hwnd_str = config
                .window_handle
                .as_deref()
                .ok_or("No window handle provided")?;
            let hwnd_val = hwnd_str
                .parse::<isize>()
                .map_err(|e| format!("Invalid window handle format: {}", e))?;
            let (w, h) = get_window_size(hwnd_val);
            if w == 0 || h == 0 {
                return Err("Failed to get window size or window is closed".into());
            }
            (w, h, 0, 0)
        } else {
            let idx = config.monitor_index.unwrap_or(-1);
            let monitor = get_selected_monitor(idx, app)?;
            let w = monitor.width().map_err(|e| e.to_string())? as u32;
            let h = monitor.height().map_err(|e| e.to_string())? as u32;
            (w, h, 0, 0)
        };

        // 强制偶数化对齐，防止硬件 H.264 幕编码器因奇数分辨率引发 MF_E_INVALIDMEDIATYPE (0xC00D36B4) 错误
        let width = width & !1;
        let height = height & !1;

        let (monitor_x, monitor_y) = if !is_window_mode {
            if let Ok(monitor) = get_selected_monitor(config.monitor_index.unwrap_or(-1), app) {
                get_monitor_phys_pos(monitor.as_raw_hmonitor())
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        };

        let hwnd_val = if is_window_mode {
            let hwnd_str = config.window_handle.as_deref().unwrap_or("0");
            hwnd_str.parse::<isize>().ok()
        } else {
            None
        };

        // 2. 初始化 Media Foundation 编码写入器并保存到 self.writer
        let writer = MFSinkWriterWrapper::new(
            output_path,
            width,
            height,
            config.fps,
            rect_x,
            rect_y,
            config,
            is_window_mode,
            hwnd_val,
            monitor_x,
            monitor_y,
        )?;
        {
            let mut writer_guard = self.writer.lock().unwrap();
            *writer_guard = Some(writer);
        }

        let cursor_settings = if config.show_mouse_cursor {
            CursorCaptureSettings::WithCursor
        } else {
            CursorCaptureSettings::WithoutCursor
        };

        // 3. 根据捕获模式分别实例化 settings 并启动捕获线程
        let capture_control = if is_window_mode {
            let hwnd_str = config
                .window_handle
                .as_deref()
                .ok_or("No window handle provided")?;
            let hwnd_val = hwnd_str
                .parse::<isize>()
                .map_err(|e| format!("Invalid window handle format: {}", e))?;
            let hwnd = hwnd_val as *mut std::ffi::c_void;
            let window = Window::from_raw_hwnd(hwnd);

            let settings = Settings::new(
                window.clone(),
                cursor_settings,
                DrawBorderSettings::WithoutBorder,
                SecondaryWindowSettings::Default,
                MinimumUpdateIntervalSettings::Default,
                DirtyRegionSettings::Default,
                ColorFormat::Bgra8,
                (self.writer.clone(), self.is_paused.clone()),
            );

            self.is_paused.store(false, Ordering::SeqCst);
            match CaptureHandler::start_free_threaded(settings) {
                Ok(control) => control,
                Err(e) => {
                    if matches!(
                        e,
                        windows_capture::capture::GraphicsCaptureApiError::GraphicsCaptureApiError(
                            windows_capture::graphics_capture_api::Error::BorderConfigUnsupported
                        )
                    ) {
                        let fallback_settings = Settings::new(
                            window,
                            cursor_settings,
                            DrawBorderSettings::Default,
                            SecondaryWindowSettings::Default,
                            MinimumUpdateIntervalSettings::Default,
                            DirtyRegionSettings::Default,
                            ColorFormat::Bgra8,
                            (self.writer.clone(), self.is_paused.clone()),
                        );
                        match CaptureHandler::start_free_threaded(fallback_settings) {
                            Ok(control) => control,
                            Err(fallback_err) => {
                                let mut writer_guard = self.writer.lock().unwrap();
                                if let Some(w) = writer_guard.take() {
                                    let _ = w.finalize();
                                }
                                let err_msg = fallback_err.to_string();
                                if err_msg.contains("Failed to convert item to GraphicsCaptureItem")
                                {
                                    return Err("启动窗口捕获失败：目标窗口不支持录像。\n\n常见原因：\n1. 目标窗口已被用户关闭或销毁；\n2. 目标窗口是以管理员权限运行的（可尝试以管理员身份重新运行 Onin）；\n3. 目标窗口由于 Windows 系统的安全保护策略被禁止录制。".into());
                                }
                                return Err(format!(
                                    "Failed to start window capture with fallback: {}",
                                    err_msg
                                ));
                            }
                        }
                    } else {
                        let mut writer_guard = self.writer.lock().unwrap();
                        if let Some(w) = writer_guard.take() {
                            let _ = w.finalize();
                        }
                        let err_msg = e.to_string();
                        if err_msg.contains("Failed to convert item to GraphicsCaptureItem") {
                            return Err("启动窗口捕获失败：目标窗口不支持录像。\n\n常见原因：\n1. 目标窗口已被用户关闭或销毁；\n2. 目标窗口是以管理员权限运行的（可尝试以管理员身份重新运行 Onin）；\n3. 目标窗口由于 Windows 系统的安全保护策略被禁止录制。".into());
                        }
                        return Err(format!("Failed to start window capture: {}", err_msg));
                    }
                }
            }
        } else {
            let idx = config.monitor_index.unwrap_or(-1);
            let monitor = get_selected_monitor(idx, app)?;

            let settings = Settings::new(
                monitor.clone(),
                cursor_settings,
                DrawBorderSettings::WithoutBorder,
                SecondaryWindowSettings::Default,
                MinimumUpdateIntervalSettings::Default,
                DirtyRegionSettings::Default,
                ColorFormat::Bgra8,
                (self.writer.clone(), self.is_paused.clone()),
            );

            self.is_paused.store(false, Ordering::SeqCst);
            match CaptureHandler::start_free_threaded(settings) {
                Ok(control) => control,
                Err(e) => {
                    if matches!(
                        e,
                        windows_capture::capture::GraphicsCaptureApiError::GraphicsCaptureApiError(
                            windows_capture::graphics_capture_api::Error::BorderConfigUnsupported
                        )
                    ) {
                        let fallback_settings = Settings::new(
                            monitor,
                            cursor_settings,
                            DrawBorderSettings::Default,
                            SecondaryWindowSettings::Default,
                            MinimumUpdateIntervalSettings::Default,
                            DirtyRegionSettings::Default,
                            ColorFormat::Bgra8,
                            (self.writer.clone(), self.is_paused.clone()),
                        );
                        match CaptureHandler::start_free_threaded(fallback_settings) {
                            Ok(control) => control,
                            Err(fallback_err) => {
                                let mut writer_guard = self.writer.lock().unwrap();
                                if let Some(w) = writer_guard.take() {
                                    let _ = w.finalize();
                                }
                                return Err(format!(
                                    "Failed to start monitor capture with fallback: {}",
                                    fallback_err
                                ));
                            }
                        }
                    } else {
                        let mut writer_guard = self.writer.lock().unwrap();
                        if let Some(w) = writer_guard.take() {
                            let _ = w.finalize();
                        }
                        return Err(format!("Failed to start monitor capture: {}", e));
                    }
                }
            }
        };

        // 启动键盘 Hook
        if config.show_keys {
            start_keyboard_hook();
        }

        // 4. 更新状态和时间计数
        {
            let mut capture_control_guard = self.capture_thread_control.lock().unwrap();
            *capture_control_guard = Some(capture_control);
        }
        *state = RecordState::Recording;

        let mut start_time_guard = self.start_time.lock().unwrap();
        *start_time_guard = Some(Instant::now());

        let mut accum_guard = self.accumulated_duration.lock().unwrap();
        *accum_guard = Duration::ZERO;

        Ok(())
    }

    fn pause(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if *state != RecordState::Recording {
            return Err("Engine is not recording".into());
        }

        self.is_paused.store(true, Ordering::SeqCst);
        *state = RecordState::Paused;

        // 累加已记录的时间
        let mut start_time_guard = self.start_time.lock().unwrap();
        if let Some(start) = *start_time_guard {
            let mut accum_guard = self.accumulated_duration.lock().unwrap();
            *accum_guard += start.elapsed();
            *start_time_guard = None;
        }

        Ok(())
    }

    fn resume(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if *state != RecordState::Paused {
            return Err("Engine is not paused".into());
        }

        self.is_paused.store(false, Ordering::SeqCst);
        *state = RecordState::Recording;

        let mut start_time_guard = self.start_time.lock().unwrap();
        *start_time_guard = Some(Instant::now());

        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        // 主动停止键盘 Hook
        stop_keyboard_hook();

        {
            let mut state = self.state.lock().unwrap();
            if *state == RecordState::Idle {
                return Err("Engine is already idle".into());
            }
            *state = RecordState::Idle;
        }

        // 1. 停止捕获线程
        {
            let mut capture_control_guard = self.capture_thread_control.lock().unwrap();
            if let Some(control) = capture_control_guard.take() {
                // 等待并优雅停止捕获
                let _ = control.stop();
            }
        }

        // 2. 完成并封存视频写入
        {
            let mut writer_guard = self.writer.lock().unwrap();
            if let Some(writer) = writer_guard.take() {
                writer.finalize()?;
            }
        }

        // 3. 重置时间与状态
        let mut start_time_guard = self.start_time.lock().unwrap();
        *start_time_guard = None;
        let mut accum_guard = self.accumulated_duration.lock().unwrap();
        *accum_guard = Duration::ZERO;

        Ok(())
    }

    fn get_state(&self) -> RecordStateSnapshot {
        let state_val = *self.state.lock().unwrap();

        let elapsed = if state_val == RecordState::Recording {
            let start_time_guard = self.start_time.lock().unwrap();
            let accum_guard = self.accumulated_duration.lock().unwrap();
            start_time_guard
                .map(|start| start.elapsed() + *accum_guard)
                .unwrap_or(*accum_guard)
        } else {
            *self.accumulated_duration.lock().unwrap()
        };

        RecordStateSnapshot {
            state: state_val,
            duration_secs: elapsed.as_secs(),
        }
    }
}

fn clean_monitor_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn get_selected_monitor(idx: i32, app: &tauri::AppHandle) -> Result<Monitor, String> {
    let selected_monitor = if idx == -1 {
        // 跟随鼠标录屏：自动捕捉当前鼠标指针所在的显示器
        unsafe {
            let mut point = POINT::default();
            if windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point).is_ok() {
                let hmonitor = MonitorFromPoint(point, MONITOR_DEFAULTTONULL);
                if !hmonitor.is_invalid() {
                    Some(Monitor::from_raw_hmonitor(
                        hmonitor.0 as *mut std::ffi::c_void,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        }
    } else {
        let win_monitors =
            Monitor::enumerate().map_err(|e| format!("Failed to enumerate monitors: {:?}", e))?;

        // 1. 优先尝试通过设备名称比对进行精确屏幕关联
        let tauri_monitors = app.available_monitors().unwrap_or_default();
        let target_clean_name = if idx >= 0 && (idx as usize) < tauri_monitors.len() {
            tauri_monitors[idx as usize]
                .name()
                .map(|n| clean_monitor_name(n))
        } else {
            None
        };

        let mut matched = None;
        if let Some(ref clean_name) = target_clean_name {
            matched = win_monitors
                .iter()
                .find(|m| {
                    m.device_name()
                        .map(|n| clean_monitor_name(&n) == *clean_name)
                        .unwrap_or(false)
                })
                .cloned();
        }

        // 2. 名称比对匹配失败时的 fallback：安全退化至系统主显示器 (Primary Monitor)
        matched.or_else(|| {
            win_monitors
                .iter()
                .find(|m| {
                    if let Ok(pri) = Monitor::primary() {
                        m.as_raw_hmonitor() == pri.as_raw_hmonitor()
                    } else {
                        false
                    }
                })
                .cloned()
        })
    };

    let selected_monitor = match selected_monitor {
        Some(m) => m,
        None => {
            Monitor::primary().map_err(|e| format!("Failed to get primary monitor: {:?}", e))?
        }
    };

    Ok(selected_monitor)
}

fn get_window_size(hwnd_val: isize) -> (u32, u32) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
    let hwnd = HWND(hwnd_val);
    let mut rect = windows::Win32::Foundation::RECT::default();
    unsafe {
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w > 0 && h > 0 {
                return (w as u32, h as u32);
            }
        }
    }
    (0, 0)
}
