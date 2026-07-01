use super::engine::{RecordConfig, RecordState, RecordStateSnapshot, ScreenRecordEngine};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows::core::PCWSTR;
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

    fn write_frame(&mut self, frame_data: &[u8], timestamp: Instant) -> Result<(), String> {
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

            let row_pitch = (self.width * 4) as usize;
            for y in 0..self.height as usize {
                let src_offset = (self.height as usize - 1 - y) * row_pitch;
                let dest_offset = y * row_pitch;
                std::ptr::copy_nonoverlapping(
                    frame_data.as_ptr().add(src_offset),
                    p_data.add(dest_offset),
                    row_pitch,
                );
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

        // 获取视频去填充后的原始像素 (BGRA 格式)
        let mut frame_buffer = frame
            .buffer()
            .map_err(|e| format!("Get frame buffer failed: {:?}", e))?;
        let no_padding_slice = frame_buffer
            .as_nopadding_buffer()
            .map_err(|e| format!("Remove padding failed: {:?}", e))?;

        let mut writer_guard = self.writer.lock().unwrap();
        if let Some(ref mut writer) = *writer_guard {
            writer.write_frame(no_padding_slice, Instant::now())?;
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
        _app: &tauri::AppHandle,
    ) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if *state != RecordState::Idle {
            return Err("Engine is already recording or paused".into());
        }

        // 1. 获取主屏幕参数（分辨率）
        let primary_monitor =
            Monitor::primary().map_err(|e| format!("Failed to get primary monitor: {:?}", e))?;
        let width = primary_monitor.width().map_err(|e| e.to_string())?;
        let height = primary_monitor.height().map_err(|e| e.to_string())?;

        // 2. 初始化 Media Foundation 编码写入器
        let writer = MFSinkWriterWrapper::new(output_path, width, height, config.fps)?;
        {
            let mut writer_guard = self.writer.lock().unwrap();
            *writer_guard = Some(writer);
        }

        // 3. 配置录像捕捉设置
        // windows-capture 的 settings 配置
        let settings = Settings::new(
            primary_monitor,
            CursorCaptureSettings::WithCursor,
            DrawBorderSettings::WithoutBorder,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Bgra8,
            (self.writer.clone(), self.is_paused.clone()),
        );

        // 4. 异步启动画面捕获线程 (start_free_threaded 不会阻塞主线程并返回控制句柄)
        self.is_paused.store(false, Ordering::SeqCst);
        let capture_control = match CaptureHandler::start_free_threaded(settings) {
            Ok(control) => control,
            Err(e) => {
                if matches!(
                    e,
                    windows_capture::capture::GraphicsCaptureApiError::GraphicsCaptureApiError(
                        windows_capture::graphics_capture_api::Error::BorderConfigUnsupported
                    )
                ) {
                    // 低版本 Windows 10/11 不支持禁用黄色录屏框，进行 Fallback 降级，开启边框重试
                    let primary_monitor_fallback = Monitor::primary()
                        .map_err(|err| format!("Failed to get primary monitor: {}", err))?;
                    let fallback_settings = Settings::new(
                        primary_monitor_fallback,
                        CursorCaptureSettings::WithCursor,
                        DrawBorderSettings::Default, // 使用系统默认配置，避开低版本系统缺失接口的底层调用
                        SecondaryWindowSettings::Default,
                        MinimumUpdateIntervalSettings::Default,
                        DirtyRegionSettings::Default,
                        ColorFormat::Bgra8,
                        (self.writer.clone(), self.is_paused.clone()),
                    );
                    match CaptureHandler::start_free_threaded(fallback_settings) {
                        Ok(control) => control,
                        Err(fallback_err) => {
                            // 清理并 finalize 已经创建的 SinkWriter，防止 Media Foundation 泄漏
                            let mut writer_guard = self.writer.lock().unwrap();
                            if let Some(w) = writer_guard.take() {
                                let _ = w.finalize();
                            }
                            return Err(format!(
                                "Failed to start windows-capture with fallback: {}",
                                fallback_err
                            ));
                        }
                    }
                } else {
                    // 清理并 finalize 已经创建的 SinkWriter，防止 Media Foundation 泄漏
                    let mut writer_guard = self.writer.lock().unwrap();
                    if let Some(w) = writer_guard.take() {
                        let _ = w.finalize();
                    }
                    return Err(format!("Failed to start windows-capture: {}", e));
                }
            }
        };

        // 5. 更新状态和时间计数
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
