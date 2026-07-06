use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use std::sync::atomic::Ordering;
use windows::Win32::Foundation::{COLORREF, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
};

pub struct Ripple {
    pub x: i32,
    pub y: i32,
    pub start_time: Instant,
    pub is_right: bool,
}

pub struct CachedKeyText {
    pub text: String,
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u8>,
}

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

pub fn start_keyboard_hook() {
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

pub fn stop_keyboard_hook() {
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

pub fn draw_mouse_clicks(
    ripples: &mut Vec<Ripple>,
    last_lbutton: &mut bool,
    last_rbutton: &mut bool,
    p_data: *mut u8,
    width: u32,
    height: u32,
    is_window_mode: bool,
    window_handle: Option<isize>,
    monitor_x: i32,
    monitor_y: i32,
    rect_x: u32,
    rect_y: u32,
) {
    let l_down = unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 };
    let r_down = unsafe { (GetAsyncKeyState(VK_RBUTTON.0 as i32) as u16 & 0x8000) != 0 };

    let mut point = POINT::default();
    let got_pos =
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point).is_ok() };

    if got_pos {
        let (rx, ry) = if is_window_mode {
            let mut rect = windows::Win32::Foundation::RECT::default();
            let mut rx = -1;
            let mut ry = -1;
            if let Some(hwnd) = window_handle {
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
                            width as f32 / rect_w as f32
                        } else {
                            1.0
                        };
                        let win_scale_y = if rect_h > 0 {
                            height as f32 / rect_h as f32
                        } else {
                            1.0
                        };

                        rx = ((point.x - rect.left) as f32 * win_scale_x) as i32;
                        ry = (height as i32 - 1)
                            - ((point.y - rect.top) as f32 * win_scale_y) as i32;
                    }
                }
            }
            (rx, ry)
        } else {
            let rx = point.x - monitor_x - rect_x as i32;
            let ry = (height as i32 - 1) - (point.y - monitor_y - rect_y as i32);
            (rx, ry)
        };

        if rx >= 0 && rx < width as i32 && ry >= 0 && ry < height as i32 {
            if l_down && !*last_lbutton {
                ripples.push(Ripple {
                    x: rx,
                    y: ry,
                    start_time: Instant::now(),
                    is_right: false,
                });
            }
            if r_down && !*last_rbutton {
                ripples.push(Ripple {
                    x: rx,
                    y: ry,
                    start_time: Instant::now(),
                    is_right: true,
                });
            }
        }
    }

    *last_lbutton = l_down;
    *last_rbutton = r_down;

    let now = Instant::now();
    ripples.retain(|ripple| now.duration_since(ripple.start_time) < Duration::from_millis(400));

    for ripple in ripples {
        let elapsed = now.duration_since(ripple.start_time).as_secs_f32();
        let alpha = 1.0 - (elapsed / 0.4);

        if ripple.is_right {
            let r_max_outer = 34.0;
            let radius_outer = (elapsed / 0.4) * r_max_outer;
            let radius_inner = (elapsed / 0.4) * 20.0;

            let r_out_inner = radius_outer - 1.8;
            let r_out_outer = radius_outer + 1.8;
            let r_in_inner = radius_inner - 1.8;
            let r_in_outer = radius_inner + 1.8;

            let center_halo_r_sq = 9.0f32 * 9.0f32;
            let r_outer_i = r_max_outer.ceil() as i32;

            for dy in -r_outer_i..=r_outer_i {
                let py = ripple.y + dy;
                if py < 0 || py >= height as i32 {
                    continue;
                }
                let dy_sq = (dy * dy) as f32;
                for dx in -r_outer_i..=r_outer_i {
                    let px = ripple.x + dx;
                    if px < 0 || px >= width as i32 {
                        continue;
                    }

                    let dist_sq = (dx * dx) as f32 + dy_sq;
                    let in_outer_ring = dist_sq >= r_out_inner * r_out_inner
                        && dist_sq <= r_out_outer * r_out_outer;
                    let in_inner_ring =
                        dist_sq >= r_in_inner * r_in_inner && dist_sq <= r_in_outer * r_in_outer;
                    let in_halo = dist_sq <= center_halo_r_sq;

                    if in_outer_ring || in_inner_ring || in_halo {
                        let offset = ((py as u32 * width) + px as u32) as usize * 4;
                        let src_r = 60u32;
                        let src_g = 130u32;
                        let src_b = 255u32;

                        let current_alpha = if in_outer_ring || in_inner_ring {
                            alpha
                        } else {
                            alpha * 0.45
                        };

                        let alpha_q8 = (current_alpha * 256.0) as u32;
                        let inv_alpha_q8 = 256 - alpha_q8;

                        unsafe {
                            let dest_b = *p_data.add(offset) as u32;
                            let dest_g = *p_data.add(offset + 1) as u32;
                            let dest_r = *p_data.add(offset + 2) as u32;

                            *p_data.add(offset) =
                                ((src_b * alpha_q8 + dest_b * inv_alpha_q8) >> 8) as u8;
                            *p_data.add(offset + 1) =
                                ((src_g * alpha_q8 + dest_g * inv_alpha_q8) >> 8) as u8;
                            *p_data.add(offset + 2) =
                                ((src_r * alpha_q8 + dest_r * inv_alpha_q8) >> 8) as u8;
                        }
                    }
                }
            }
        } else {
            let r_max = 28.0;
            let radius = (elapsed / 0.4) * r_max;

            let r_inner = radius - 2.0;
            let r_outer = radius + 2.0;
            let r_inner_sq = r_inner * r_inner;
            let r_outer_sq = r_outer * r_outer;

            let center_halo_r_sq = 8.0f32 * 8.0f32;
            let r_outer_i = r_max.ceil() as i32;

            for dy in -r_outer_i..=r_outer_i {
                let py = ripple.y + dy;
                if py < 0 || py >= height as i32 {
                    continue;
                }
                let dy_sq = (dy * dy) as f32;
                for dx in -r_outer_i..=r_outer_i {
                    let px = ripple.x + dx;
                    if px < 0 || px >= width as i32 {
                        continue;
                    }

                    let dist_sq = (dx * dx) as f32 + dy_sq;
                    let in_ring = dist_sq >= r_inner_sq && dist_sq <= r_outer_sq;
                    let in_halo = dist_sq <= center_halo_r_sq;

                    if in_ring || in_halo {
                        let offset = ((py as u32 * width) + px as u32) as usize * 4;
                        let src_r = 255u32;
                        let src_g = 60u32;
                        let src_b = 60u32;

                        let current_alpha = if in_ring { alpha } else { alpha * 0.45 };

                        let alpha_q8 = (current_alpha * 256.0) as u32;
                        let inv_alpha_q8 = 256 - alpha_q8;

                        unsafe {
                            let dest_b = *p_data.add(offset) as u32;
                            let dest_g = *p_data.add(offset + 1) as u32;
                            let dest_r = *p_data.add(offset + 2) as u32;

                            *p_data.add(offset) =
                                ((src_b * alpha_q8 + dest_b * inv_alpha_q8) >> 8) as u8;
                            *p_data.add(offset + 1) =
                                ((src_g * alpha_q8 + dest_g * inv_alpha_q8) >> 8) as u8;
                            *p_data.add(offset + 2) =
                                ((src_r * alpha_q8 + dest_r * inv_alpha_q8) >> 8) as u8;
                        }
                    }
                }
            }
        }
    }
}

pub fn draw_keys_overlay(
    cached_key_text: &mut Option<CachedKeyText>,
    p_data: *mut u8,
    width: u32,
    height: u32,
) {
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

            draw_keyboard_overlay(cached_key_text, p_data, &info.text, alpha, width, height);
        }
    }
}

fn draw_keyboard_overlay(
    cached_key_text: &mut Option<CachedKeyText>,
    p_data: *mut u8,
    text: &str,
    alpha: f32,
    width: u32,
    height: u32,
) {
    let mut hit_cache = false;

    if let Some(ref cached) = cached_key_text {
        if cached.text == text {
            hit_cache = true;
        }
    }

    if !hit_cache {
        let wide_text: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        unsafe {
            let screen_hdc = GetDC(None);
            if !screen_hdc.is_invalid() {
                let mem_hdc = CreateCompatibleDC(screen_hdc);
                if !mem_hdc.is_invalid() {
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
                    if GetTextExtentPoint32W(mem_hdc, &wide_text[..text_len], &mut text_size)
                        .as_bool()
                    {
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
                            bmp_info.bmiHeader.biSize =
                                std::mem::size_of::<BITMAPINFOHEADER>() as u32;
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
                                DIB_RGB_COLORS,
                            );

                            *cached_key_text = Some(CachedKeyText {
                                text: text.to_string(),
                                width: text_width,
                                height: text_height,
                                pixels: bmp_pixels,
                            });

                            SelectObject(mem_hdc, old_bmp);
                            let _ = DeleteObject(hbmp);
                        }
                    }

                    SelectObject(mem_hdc, old_font);
                    let _ = DeleteObject(font);
                    let _ = DeleteDC(mem_hdc);
                }
                let _ = ReleaseDC(None, screen_hdc);
            }
        }
    }

    if let Some(ref cached) = cached_key_text {
        let bmp_pixels = &cached.pixels;
        let text_width = cached.width;
        let text_height = cached.height;

        let padding_x = 32;
        let padding_y = 16;
        let box_width = text_width + padding_x * 2;
        let box_height = text_height + padding_y * 2;

        let box_x = 70;
        let box_y = height as i32 - box_height - 70;

        let radius = 16;
        let bg_opacity = 0.72 * alpha;
        let factor_q8 = ((1.0 - bg_opacity) * 256.0) as u32;
        let text_alpha_q8_base = (alpha * 256.0) as u32;

        unsafe {
            for y in 0..box_height {
                let py = height as i32 - 1 - (box_y + y);
                if py < 0 || py >= height as i32 {
                    continue;
                }
                for x in 0..box_width {
                    let px = box_x + x;
                    if px < 0 || px >= width as i32 {
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
                        let dest_offset = ((py as u32 * width) + px as u32) as usize * 4;
                        let b = *p_data.add(dest_offset) as u32;
                        let g = *p_data.add(dest_offset + 1) as u32;
                        let r = *p_data.add(dest_offset + 2) as u32;

                        *p_data.add(dest_offset) = ((b * factor_q8) >> 8) as u8;
                        *p_data.add(dest_offset + 1) = ((g * factor_q8) >> 8) as u8;
                        *p_data.add(dest_offset + 2) = ((r * factor_q8) >> 8) as u8;
                    }
                }
            }

            let text_start_x = box_x + padding_x;
            let text_start_y = box_y + padding_y;

            for y in 0..text_height {
                let py = height as i32 - 1 - (text_start_y + y);
                if py < 0 || py >= height as i32 {
                    continue;
                }
                for x in 0..text_width {
                    let px = text_start_x + x;
                    if px < 0 || px >= width as i32 {
                        continue;
                    }

                    let bmp_offset = ((y * text_width) + x) as usize * 4;
                    let intensity = bmp_pixels[bmp_offset + 2] as u32;

                    if intensity > 0 {
                        let dest_offset = ((py as u32 * width) + px as u32) as usize * 4;
                        let alpha_q8 = (intensity * text_alpha_q8_base) / 255;
                        let inv_alpha_q8 = 256 - alpha_q8;

                        let dest_b = *p_data.add(dest_offset) as u32;
                        let dest_g = *p_data.add(dest_offset + 1) as u32;
                        let dest_r = *p_data.add(dest_offset + 2) as u32;

                        *p_data.add(dest_offset) =
                            ((255 * alpha_q8 + dest_b * inv_alpha_q8) >> 8) as u8;
                        *p_data.add(dest_offset + 1) =
                            ((255 * alpha_q8 + dest_g * inv_alpha_q8) >> 8) as u8;
                        *p_data.add(dest_offset + 2) =
                            ((255 * alpha_q8 + dest_r * inv_alpha_q8) >> 8) as u8;
                    }
                }
            }
        }
    }
}
