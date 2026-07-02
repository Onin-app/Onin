use super::engine::{RecordConfig, RecordState, RecordStateSnapshot, ScreenRecordEngine};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows::core::PCWSTR;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, MONITOR_DEFAULTTONULL};
use windows::Win32::Media::MediaFoundation::*;

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

struct MFSinkWriterWrapper {
    sink_writer: IMFSinkWriter,
    video_stream_index: u32,
    width: u32,
    height: u32,
    fps: u32,
    start_time: Option<Instant>,
}

// 手动实现 Send & Sync，使 COM 指针可以在线程间共享和传递
unsafe impl Send for MFSinkWriterWrapper {}
unsafe impl Sync for MFSinkWriterWrapper {}

impl MFSinkWriterWrapper {
    fn new(output_path: &Path, width: u32, height: u32, fps: u32) -> Result<Self, String> {
        unsafe {
            // 初始化 Media Foundation
            MFStartup(MF_VERSION, MFSTARTUP_FULL)
                .map_err(|e| format!("MFStartup failed: {:?}", e))?;

            // 转换文件路径
            let path_u16: Vec<u16> = output_path
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            let path_pcwstr = PCWSTR(path_u16.as_ptr());

            // 创建 SinkWriter
            let sink_writer = MFCreateSinkWriterFromURL(path_pcwstr, None, None)
                .map_err(|e| format!("MFCreateSinkWriterFromURL failed: {:?}", e))?;

            // 1. 创建并设置输出媒体类型 (H.264 视频)
            let out_media_type =
                MFCreateMediaType().map_err(|e| format!("MFCreateMediaType failed: {:?}", e))?;
            out_media_type
                .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
                .map_err(|e| format!("SetGUID MajorType failed: {:?}", e))?;
            out_media_type
                .SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)
                .map_err(|e| format!("SetGUID Subtype H264 failed: {:?}", e))?;
            out_media_type
                .SetUINT32(&MF_MT_AVG_BITRATE, 5_000_000) // 5 Mbps (常量在 windows 0.57 里为 MF_MT_AVG_BITRATE)
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

            // 2. 创建并设置输入媒体类型 (BGRA 32位像素流)
            let in_media_type =
                MFCreateMediaType().map_err(|e| format!("MFCreateMediaType failed: {:?}", e))?;
            in_media_type
                .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
                .map_err(|e| format!("SetGUID Input MajorType failed: {:?}", e))?;
            in_media_type
                .SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32) // BGRA 格式对应 RGB32
                .map_err(|e| format!("SetGUID Input Subtype RGB32 failed: {:?}", e))?;
            in_media_type
                .SetUINT64(&MF_MT_FRAME_SIZE, pack_u32_to_u64(width, height))
                .map_err(|e| format!("SetUINT64 Input FrameSize failed: {:?}", e))?;
            in_media_type
                .SetUINT64(&MF_MT_FRAME_RATE, pack_u32_to_u64(fps, 1))
                .map_err(|e| format!("SetUINT64 Input FrameRate failed: {:?}", e))?;

            // 添加视频流并关联输入类型
            let video_stream_index = sink_writer
                .AddStream(&out_media_type)
                .map_err(|e| format!("AddStream failed: {:?}", e))?;
            sink_writer
                .SetInputMediaType(video_stream_index, &in_media_type, None)
                .map_err(|e| format!("SetInputMediaType failed: {:?}", e))?;

            // 开始写入
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

            // 1. 创建 Media Buffer
            let buffer_len = self.width * self.height * 4;
            let buffer = MFCreateMemoryBuffer(buffer_len)
                .map_err(|e| format!("MFCreateMemoryBuffer failed: {:?}", e))?;

            // 2. 锁存并拷贝像素
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

            // 将整个缓冲区填充为 0 (黑色背景)，以防捕获源的宽高比配置小
            std::ptr::write_bytes(p_data, 0, buffer_len as usize);

            let src_row_pitch = (frame_width * 4) as usize;
            let dest_row_pitch = (self.width * 4) as usize;
            let copy_width_bytes = (self.width.min(frame_width) * 4) as usize;
            let copy_height = self.height.min(frame_height) as usize;

            for y in 0..copy_height {
                // 翻转 Y 轴拷贝
                let src_offset = (frame_height as usize - 1 - y) * src_row_pitch;
                let dest_offset = y * dest_row_pitch;

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

            buffer
                .SetCurrentLength(buffer_len)
                .map_err(|e| format!("SetCurrentLength failed: {:?}", e))?;
            buffer
                .Unlock()
                .map_err(|e| format!("Buffer Unlock failed: {:?}", e))?;

            // 3. 创建 Sample 并关联 Buffer
            let sample = MFCreateSample().map_err(|e| format!("MFCreateSample failed: {:?}", e))?;
            sample
                .AddBuffer(&buffer)
                .map_err(|e| format!("AddBuffer failed: {:?}", e))?;

            // 4. 计算并设定时间戳（100纳秒单位）
            let sample_time = (elapsed.as_nanos() / 100) as i64;
            sample
                .SetSampleTime(sample_time)
                .map_err(|e| format!("SetSampleTime failed: {:?}", e))?;
            let sample_duration = (1_000_000_000 / self.fps) as i64 / 100;
            sample
                .SetSampleDuration(sample_duration)
                .map_err(|e| format!("SetSampleDuration failed: {:?}", e))?;

            // 5. 写入流
            self.sink_writer
                .WriteSample(self.video_stream_index, &sample)
                .map_err(|e| format!("WriteSample failed: {:?}", e))?;

            Ok(())
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

        let (width, height) = if is_window_mode {
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
            (w, h)
        } else {
            let idx = config.monitor_index.unwrap_or(-1);
            let monitor = get_selected_monitor(idx, app)?;
            let w = monitor.width().map_err(|e| e.to_string())? as u32;
            let h = monitor.height().map_err(|e| e.to_string())? as u32;
            (w, h)
        };

        // 强制偶数化对齐，防止硬件 H.264 编码器因奇数分辨率引发 MF_E_INVALIDMEDIATYPE (0xC00D36B4) 错误
        let width = width & !1;
        let height = height & !1;

        // 2. 初始化 Media Foundation 编码写入器并保存到 self.writer
        let writer = MFSinkWriterWrapper::new(output_path, width, height, config.fps)?;
        {
            let mut writer_guard = self.writer.lock().unwrap();
            *writer_guard = Some(writer);
        }

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
                CursorCaptureSettings::WithCursor,
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
                            CursorCaptureSettings::WithCursor,
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
                CursorCaptureSettings::WithCursor,
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
                            CursorCaptureSettings::WithCursor,
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
