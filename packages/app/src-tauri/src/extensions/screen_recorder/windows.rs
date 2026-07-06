use super::engine::{RecordConfig, RecordState, RecordStateSnapshot, ScreenRecordEngine};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// 引入拆分出的音频与视觉覆盖层子模块
#[path = "windows_audio.rs"]
mod windows_audio;
#[path = "windows_overlay.rs"]
mod windows_overlay;

use windows_audio::{
    AudioBufferPool, AudioCaptureManager, MFAUDIO_FORMAT_AAC, MFAUDIO_FORMAT_FLOAT,
};
use windows_overlay::{
    draw_keys_overlay, draw_mouse_clicks, start_keyboard_hook, stop_keyboard_hook, CachedKeyText,
    Ripple,
};

use windows::core::PCWSTR;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONULL,
};
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

// ============================================================================
// 帧数据 Buffer 复用池与后台编码消息
// ============================================================================

struct MFFrameSample {
    sample: IMFSample,
    buffer: IMFMediaBuffer,
    buffer_len: u32,
}

unsafe impl Send for MFFrameSample {}
unsafe impl Sync for MFFrameSample {}

fn create_mf_sample(buffer_len: u32) -> Result<MFFrameSample, String> {
    unsafe {
        let buffer = MFCreateMemoryBuffer(buffer_len)
            .map_err(|e| format!("MFCreateMemoryBuffer failed: {:?}", e))?;
        buffer
            .SetCurrentLength(buffer_len)
            .map_err(|e| format!("SetCurrentLength failed: {:?}", e))?;

        let sample = MFCreateSample().map_err(|e| format!("MFCreateSample failed: {:?}", e))?;
        sample
            .AddBuffer(&buffer)
            .map_err(|e| format!("AddBuffer failed: {:?}", e))?;

        Ok(MFFrameSample {
            sample,
            buffer,
            buffer_len,
        })
    }
}

struct BufferPool {
    pool: Mutex<Vec<MFFrameSample>>,
    buffer_size: usize,
}

impl BufferPool {
    fn new(buffer_size: usize, capacity: usize) -> Result<Self, String> {
        let mut pool = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            let frame_sample = create_mf_sample(buffer_size as u32)?;
            pool.push(frame_sample);
        }
        Ok(Self {
            pool: Mutex::new(pool),
            buffer_size,
        })
    }

    fn acquire(&self) -> Result<MFFrameSample, String> {
        let mut pool = self.pool.lock().unwrap();
        if let Some(sample) = pool.pop() {
            Ok(sample)
        } else {
            create_mf_sample(self.buffer_size as u32)
        }
    }

    fn release(&self, sample: MFFrameSample) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < 30 {
            pool.push(sample);
        }
    }
}

enum EncoderMessage {
    Frame {
        frame_sample: MFFrameSample,
        frame_width: u32,
        frame_height: u32,
        timestamp: Instant,
    },
    Audio {
        data: Vec<f32>,
        timestamp: Instant,
    },
    Exit,
}

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

struct MFSinkWriterWrapper {
    sink_writer: IMFSinkWriter,
    video_stream_index: u32,
    audio_stream_index: Option<u32>,
    audio_buffer_pool: Option<AudioBufferPool>,
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
    cached_key_text: Option<CachedKeyText>,
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

            let enable_audio = config.record_audio || config.record_system_sound;
            let mut audio_stream_index = None;
            let mut audio_buffer_pool = None;

            if enable_audio {
                let out_audio_type = MFCreateMediaType()
                    .map_err(|e| format!("MFCreateMediaType Audio failed: {:?}", e))?;
                out_audio_type
                    .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)
                    .map_err(|e| format!("SetGUID Audio MajorType failed: {:?}", e))?;
                out_audio_type
                    .SetGUID(&MF_MT_SUBTYPE, &MFAUDIO_FORMAT_AAC)
                    .map_err(|e| format!("SetGUID Audio Subtype AAC failed: {:?}", e))?;
                out_audio_type
                    .SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, 2)
                    .map_err(|e| format!("SetUINT32 Audio NumChannels failed: {:?}", e))?;
                out_audio_type
                    .SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, 48000)
                    .map_err(|e| format!("SetUINT32 Audio SamplesPerSecond failed: {:?}", e))?;
                out_audio_type
                    .SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 16000) // 128 kbps
                    .map_err(|e| format!("SetUINT32 Audio AvgBytesPerSecond failed: {:?}", e))?;
                out_audio_type
                    .SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)
                    .map_err(|e| format!("SetUINT32 Audio BitsPerSample failed: {:?}", e))?;

                let in_audio_type = MFCreateMediaType()
                    .map_err(|e| format!("MFCreateMediaType Input Audio failed: {:?}", e))?;
                in_audio_type
                    .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)
                    .map_err(|e| format!("SetGUID Input Audio MajorType failed: {:?}", e))?;
                in_audio_type
                    .SetGUID(&MF_MT_SUBTYPE, &MFAUDIO_FORMAT_FLOAT)
                    .map_err(|e| format!("SetGUID Input Audio Subtype Float failed: {:?}", e))?;
                in_audio_type
                    .SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, 2)
                    .map_err(|e| format!("SetUINT32 Input Audio NumChannels failed: {:?}", e))?;
                in_audio_type
                    .SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, 48000)
                    .map_err(|e| {
                        format!("SetUINT32 Input Audio SamplesPerSecond failed: {:?}", e)
                    })?;
                in_audio_type
                    .SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 32)
                    .map_err(|e| format!("SetUINT32 Input Audio BitsPerSample failed: {:?}", e))?;
                in_audio_type
                    .SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, 8) // 2 channels * 4 bytes
                    .map_err(|e| format!("SetUINT32 Input Audio BlockAlignment failed: {:?}", e))?;

                let idx = sink_writer
                    .AddStream(&out_audio_type)
                    .map_err(|e| format!("AddStream Audio failed: {:?}", e))?;
                sink_writer
                    .SetInputMediaType(idx, &in_audio_type, None)
                    .map_err(|e| format!("SetInputMediaType Audio failed: {:?}", e))?;

                audio_stream_index = Some(idx);
                // 预分配 30 个 20ms 的采样缓冲 (1920 samples * 4 bytes = 7680)
                audio_buffer_pool = Some(AudioBufferPool::new(7680, 30)?);
            }

            sink_writer
                .BeginWriting()
                .map_err(|e| format!("BeginWriting failed: {:?}", e))?;

            Ok(Self {
                sink_writer,
                video_stream_index,
                audio_stream_index,
                audio_buffer_pool,
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
                cached_key_text: None,
            })
        }
    }

    fn write_audio_sample(&mut self, data: &[f32], timestamp: Instant) -> Result<(), String> {
        let stream_idx = match self.audio_stream_index {
            Some(idx) => idx,
            None => return Ok(()),
        };

        let pool = self
            .audio_buffer_pool
            .as_ref()
            .ok_or_else(|| "AudioBufferPool not initialized".to_string())?;

        let audio_sample = pool.acquire()?;

        unsafe {
            if self.start_time.is_none() {
                self.start_time = Some(timestamp);
            }
            let elapsed = timestamp.duration_since(self.start_time.unwrap_or(timestamp));
            let sample_time = (elapsed.as_nanos() / 100) as i64;

            let byte_len = (data.len() * 4) as u32;

            let mut p_data: *mut u8 = std::ptr::null_mut();
            audio_sample
                .buffer
                .Lock(&mut p_data, None, None)
                .map_err(|e| format!("IMFMediaBuffer Lock Audio failed: {:?}", e))?;

            std::ptr::copy_nonoverlapping(data.as_ptr() as *const u8, p_data, byte_len as usize);

            audio_sample
                .buffer
                .Unlock()
                .map_err(|e| format!("IMFMediaBuffer Unlock Audio failed: {:?}", e))?;

            audio_sample
                .buffer
                .SetCurrentLength(byte_len)
                .map_err(|e| format!("SetCurrentLength Audio failed: {:?}", e))?;

            audio_sample
                .sample
                .SetSampleTime(sample_time)
                .map_err(|e| format!("SetSampleTime Audio failed: {:?}", e))?;

            let samples_count = data.len() as i64 / 2;
            let sample_duration = (samples_count * 10_000_000) / 48000;
            audio_sample
                .sample
                .SetSampleDuration(sample_duration)
                .map_err(|e| format!("SetSampleDuration Audio failed: {:?}", e))?;

            self.sink_writer
                .WriteSample(stream_idx, &audio_sample.sample)
                .map_err(|e| format!("WriteSample Audio failed: {:?}", e))?;
        }

        pool.release(audio_sample);
        Ok(())
    }

    fn draw_overlays(
        &mut self,
        p_data: *mut u8,
        _frame_width: u32,
        _frame_height: u32,
        _timestamp: Instant,
    ) {
        if self.show_mouse_click {
            draw_mouse_clicks(
                &mut self.ripples,
                &mut self.last_lbutton,
                &mut self.last_rbutton,
                p_data,
                self.width,
                self.height,
                self.is_window_mode,
                self.window_handle,
                self.monitor_x,
                self.monitor_y,
                self.rect_x,
                self.rect_y,
            );
        }

        if self.show_keys {
            draw_keys_overlay(&mut self.cached_key_text, p_data, self.width, self.height);
        }
    }

    fn write_sample(&mut self, sample: &IMFSample, timestamp: Instant) -> Result<(), String> {
        unsafe {
            if self.start_time.is_none() {
                self.start_time = Some(timestamp);
            }
            let elapsed = timestamp.duration_since(self.start_time.unwrap_or(timestamp));

            let sample_time = (elapsed.as_nanos() / 100) as i64;
            sample
                .SetSampleTime(sample_time)
                .map_err(|e| format!("SetSampleTime failed: {:?}", e))?;
            let sample_duration = (1_000_000_000 / self.fps) as i64 / 100;
            sample
                .SetSampleDuration(sample_duration)
                .map_err(|e| format!("SetSampleDuration failed: {:?}", e))?;

            self.sink_writer
                .WriteSample(self.video_stream_index, sample)
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
    sender: SyncSender<EncoderMessage>,
    buffer_pool: Arc<BufferPool>,
    is_paused: Arc<AtomicBool>,
    width: u32,
    height: u32,
    rect_x: u32,
    rect_y: u32,
}

impl GraphicsCaptureApiHandler for CaptureHandler {
    type Flags = (
        SyncSender<EncoderMessage>,
        Arc<BufferPool>,
        Arc<AtomicBool>,
        u32,
        u32,
        u32,
        u32,
    );
    type Error = String;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let (sender, buffer_pool, is_paused, width, height, rect_x, rect_y) = ctx.flags;
        Ok(Self {
            sender,
            buffer_pool,
            is_paused,
            width,
            height,
            rect_x,
            rect_y,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        // 使用 Acquire 内存序替换 SeqCst，消除多核强内存屏障开销
        if self.is_paused.load(Ordering::Acquire) {
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

        // 从 BufferPool 中获取一个已预分配的复用 MFFrameSample
        let frame_sample = self.buffer_pool.acquire().map_err(|e| e.to_string())?;

        // 直接锁定 IMFMediaBuffer 指针进行 in-place 垂直翻转和裁剪拷贝 (这是唯一的一次内存拷贝！)
        unsafe {
            let mut p_data: *mut u8 = std::ptr::null_mut();
            frame_sample
                .buffer
                .Lock(&mut p_data, None, None)
                .map_err(|e| format!("IMFMediaBuffer Lock failed: {:?}", e))?;

            let src_row_pitch = (frame_width * 4) as usize;
            let dest_row_pitch = (self.width * 4) as usize;
            let copy_width_bytes = (self.width * 4) as usize;

            for y in 0..self.height {
                let src_y = frame_height as i32 - 1 - (self.rect_y + y) as i32;
                if src_y >= 0 && src_y < frame_height as i32 {
                    let src_offset = (src_y as usize) * src_row_pitch + (self.rect_x as usize * 4);
                    let dest_offset = (y as usize) * dest_row_pitch;

                    if src_offset + copy_width_bytes <= no_padding_slice.len()
                        && dest_offset + copy_width_bytes <= frame_sample.buffer_len as usize
                    {
                        std::ptr::copy_nonoverlapping(
                            no_padding_slice.as_ptr().add(src_offset),
                            p_data.add(dest_offset),
                            copy_width_bytes,
                        );
                    }
                }
            }

            frame_sample
                .buffer
                .Unlock()
                .map_err(|e| format!("IMFMediaBuffer Unlock failed: {:?}", e))?;
        }

        // 发送给后台编码线程
        if let Err(e) = self.sender.try_send(EncoderMessage::Frame {
            frame_sample,
            frame_width,
            frame_height,
            timestamp: Instant::now(),
        }) {
            match e {
                TrySendError::Full(EncoderMessage::Frame { frame_sample, .. }) => {
                    // 积压丢帧，释放归还池中
                    self.buffer_pool.release(frame_sample);
                }
                _ => {}
            }
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
    is_paused: Arc<AtomicBool>,
    capture_thread_control: Arc<Mutex<Option<CaptureControl<CaptureHandler, String>>>>,
    encoder_thread: Mutex<Option<std::thread::JoinHandle<Result<(), String>>>>,
    sender: Mutex<Option<SyncSender<EncoderMessage>>>,
    audio_capture: Mutex<Option<AudioCaptureManager>>,
}

impl WindowsRecordEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RecordState::Idle)),
            start_time: Arc::new(Mutex::new(None)),
            accumulated_duration: Arc::new(Mutex::new(Duration::ZERO)),
            is_paused: Arc::new(AtomicBool::new(false)),
            capture_thread_control: Arc::new(Mutex::new(None)),
            encoder_thread: Mutex::new(None),
            sender: Mutex::new(None),
            audio_capture: Mutex::new(None),
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

        // 2. 初始化 Media Foundation 编码写入器并启动后台编码线程
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

        let buffer_size = (width * height * 4) as usize;
        let buffer_pool = Arc::new(BufferPool::new(buffer_size, 30)?);
        let (sender, receiver) = sync_channel::<EncoderMessage>(30);

        let buffer_pool_clone = buffer_pool.clone();
        let handle = std::thread::spawn(move || {
            let mut writer = writer;
            while let Ok(msg) = receiver.recv() {
                match msg {
                    EncoderMessage::Frame {
                        frame_sample,
                        frame_width,
                        frame_height,
                        timestamp,
                    } => {
                        unsafe {
                            let mut p_data: *mut u8 = std::ptr::null_mut();
                            if frame_sample.buffer.Lock(&mut p_data, None, None).is_ok() {
                                writer.draw_overlays(p_data, frame_width, frame_height, timestamp);
                                let _ = frame_sample.buffer.Unlock();
                            }
                        }
                        let _ = writer.write_sample(&frame_sample.sample, timestamp);
                        buffer_pool_clone.release(frame_sample);
                    }
                    EncoderMessage::Audio { data, timestamp } => {
                        let _ = writer.write_audio_sample(&data, timestamp);
                    }
                    EncoderMessage::Exit => {
                        break;
                    }
                }
            }
            writer.finalize()?;
            Ok::<(), String>(())
        });

        *self.encoder_thread.lock().unwrap() = Some(handle);
        *self.sender.lock().unwrap() = Some(sender.clone());

        let cleanup_encoder = || {
            let sender = self.sender.lock().unwrap().take();
            if let Some(s) = sender {
                let _ = s.send(EncoderMessage::Exit);
            }
            let handle = self.encoder_thread.lock().unwrap().take();
            if let Some(h) = handle {
                let _ = h.join();
            }
        };

        let cursor_settings = if config.show_mouse_cursor {
            CursorCaptureSettings::WithCursor
        } else {
            CursorCaptureSettings::WithoutCursor
        };

        let flags = (
            sender.clone(),
            buffer_pool.clone(),
            self.is_paused.clone(),
            width,
            height,
            rect_x,
            rect_y,
        );

        // 3. 根据捕获模式分别实例化 settings 并启动捕获线程
        let capture_control = if is_window_mode {
            let hwnd_str = config.window_handle.as_deref().ok_or_else(|| {
                cleanup_encoder();
                "No window handle provided".to_string()
            })?;
            let hwnd_val = hwnd_str.parse::<isize>().map_err(|e| {
                cleanup_encoder();
                format!("Invalid window handle format: {}", e)
            })?;
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
                flags.clone(),
            );

            self.is_paused.store(false, Ordering::Release);
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
                            flags.clone(),
                        );
                        match CaptureHandler::start_free_threaded(fallback_settings) {
                            Ok(control) => control,
                            Err(fallback_err) => {
                                cleanup_encoder();
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
                        cleanup_encoder();
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
            let monitor = get_selected_monitor(idx, app).map_err(|e| {
                cleanup_encoder();
                e
            })?;

            let settings = Settings::new(
                monitor.clone(),
                cursor_settings,
                DrawBorderSettings::WithoutBorder,
                SecondaryWindowSettings::Default,
                MinimumUpdateIntervalSettings::Default,
                DirtyRegionSettings::Default,
                ColorFormat::Bgra8,
                flags.clone(),
            );

            self.is_paused.store(false, Ordering::Release);
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
                            flags.clone(),
                        );
                        match CaptureHandler::start_free_threaded(fallback_settings) {
                            Ok(control) => control,
                            Err(fallback_err) => {
                                cleanup_encoder();
                                return Err(format!(
                                    "Failed to start monitor capture with fallback: {}",
                                    fallback_err
                                ));
                            }
                        }
                    } else {
                        cleanup_encoder();
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

        // 启动音频捕获
        let audio_capture =
            match AudioCaptureManager::start(config, sender.clone(), self.is_paused.clone()) {
                Ok(ac) => ac,
                Err(e) => {
                    stop_keyboard_hook();
                    let mut capture_control_guard = self.capture_thread_control.lock().unwrap();
                    if let Some(control) = capture_control_guard.take() {
                        let _ = control.stop();
                    }
                    cleanup_encoder();
                    return Err(e);
                }
            };
        *self.audio_capture.lock().unwrap() = Some(audio_capture);

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

        self.is_paused.store(true, Ordering::Release);
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

        self.is_paused.store(false, Ordering::Release);
        *state = RecordState::Recording;

        let mut start_time_guard = self.start_time.lock().unwrap();
        *start_time_guard = Some(Instant::now());

        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        // 主动停止键盘 Hook
        stop_keyboard_hook();

        // 停止音频捕获
        let mut audio_capture_guard = self.audio_capture.lock().unwrap();
        if let Some(mut audio) = audio_capture_guard.take() {
            audio.stop();
        }

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

        // 2. 发送 Exit 信号给后台编码线程并 join 等待其退出
        let sender = self.sender.lock().unwrap().take();
        if let Some(s) = sender {
            let _ = s.send(EncoderMessage::Exit);
        }

        let handle = self.encoder_thread.lock().unwrap().take();
        if let Some(h) = handle {
            if let Ok(res) = h.join() {
                res?;
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
