use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::super::engine::RecordConfig;
use super::EncoderMessage;

use windows::Win32::Media::MediaFoundation::*;

// MFAudioFormat_AAC: 00001610-0000-0010-8000-00AA00389B71
pub const MFAUDIO_FORMAT_AAC: windows::core::GUID = windows::core::GUID::from_values(
    0x00001610,
    0x0000,
    0x0010,
    [0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71],
);
// MFAudioFormat_Float: 00000003-0000-0010-8000-00AA00389B71
pub const MFAUDIO_FORMAT_FLOAT: windows::core::GUID = windows::core::GUID::from_values(
    0x00000003,
    0x0000,
    0x0010,
    [0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71],
);

pub struct MFAudioSample {
    pub sample: IMFSample,
    pub buffer: IMFMediaBuffer,
    pub _buffer_len: u32,
}

unsafe impl Send for MFAudioSample {}
unsafe impl Sync for MFAudioSample {}

fn create_mf_audio_sample(buffer_len: u32) -> Result<MFAudioSample, String> {
    unsafe {
        let buffer = MFCreateMemoryBuffer(buffer_len)
            .map_err(|e| format!("MFCreateMemoryBuffer Audio failed: {:?}", e))?;
        buffer
            .SetCurrentLength(buffer_len)
            .map_err(|e| format!("SetCurrentLength Audio failed: {:?}", e))?;

        let sample =
            MFCreateSample().map_err(|e| format!("MFCreateSample Audio failed: {:?}", e))?;
        sample
            .AddBuffer(&buffer)
            .map_err(|e| format!("AddBuffer Audio failed: {:?}", e))?;

        Ok(MFAudioSample {
            sample,
            buffer,
            _buffer_len: buffer_len,
        })
    }
}

pub struct AudioBufferPool {
    pool: Mutex<Vec<MFAudioSample>>,
    buffer_size: usize,
}

impl AudioBufferPool {
    pub fn new(buffer_size: usize, capacity: usize) -> Result<Self, String> {
        let mut pool = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            let sample = create_mf_audio_sample(buffer_size as u32)?;
            pool.push(sample);
        }
        Ok(Self {
            pool: Mutex::new(pool),
            buffer_size,
        })
    }

    pub fn acquire(&self) -> Result<MFAudioSample, String> {
        let mut pool = self.pool.lock().unwrap();
        if let Some(sample) = pool.pop() {
            Ok(sample)
        } else {
            create_mf_audio_sample(self.buffer_size as u32)
        }
    }

    pub fn release(&self, sample: MFAudioSample) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < 30 {
            pool.push(sample);
        }
    }
}

pub struct AudioResampler {
    in_rate: u32,
    out_rate: u32,
    phase: f64,
}

impl AudioResampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Self {
        Self {
            in_rate,
            out_rate,
            phase: 0.0,
        }
    }

    pub fn process(&mut self, input: &[f32], in_channels: u16, output: &mut Vec<f32>) {
        if input.is_empty() {
            return;
        }

        let mut frames_2ch = Vec::with_capacity(input.len() / in_channels as usize * 2);
        for chunk in input.chunks_exact(in_channels as usize) {
            if in_channels == 1 {
                frames_2ch.push(chunk[0]);
                frames_2ch.push(chunk[0]);
            } else {
                frames_2ch.push(chunk[0]);
                frames_2ch.push(chunk[1]);
            }
        }

        let ratio = self.in_rate as f64 / self.out_rate as f64;
        let num_input_frames = frames_2ch.len() / 2;

        while self.phase < num_input_frames as f64 {
            let idx = self.phase.floor() as usize;
            let next_idx = if idx + 1 < num_input_frames {
                idx + 1
            } else {
                idx
            };
            let frac = self.phase - idx as f64;

            let left_cur = frames_2ch[idx * 2];
            let left_next = frames_2ch[next_idx * 2];
            let left_val = left_cur + (left_next - left_cur) * frac as f32;

            let right_cur = frames_2ch[idx * 2 + 1];
            let right_next = frames_2ch[next_idx * 2 + 1];
            let right_val = right_cur + (right_next - right_cur) * frac as f32;

            output.push(left_val);
            output.push(right_val);

            self.phase += ratio;
        }

        self.phase -= num_input_frames as f64;
        if self.phase < 0.0 {
            self.phase = 0.0;
        }
    }
}

pub struct AudioMixer {
    mic_queue: VecDeque<f32>,
    sys_queue: VecDeque<f32>,
    sender: SyncSender<EncoderMessage>,
    enable_mic: bool,
    enable_sys: bool,
}

impl AudioMixer {
    pub fn new(sender: SyncSender<EncoderMessage>, enable_mic: bool, enable_sys: bool) -> Self {
        Self {
            mic_queue: VecDeque::new(),
            sys_queue: VecDeque::new(),
            sender,
            enable_mic,
            enable_sys,
        }
    }

    pub fn push_mic(&mut self, data: &[f32]) {
        self.mic_queue.extend(data);
        self.try_mix();
    }

    pub fn push_sys(&mut self, data: &[f32]) {
        self.sys_queue.extend(data);
        self.try_mix();
    }

    fn try_mix(&mut self) {
        let chunk_size = 1920; // 20ms chunk @ 48000Hz stereo

        // 如果只开了麦克风
        if self.enable_mic && !self.enable_sys {
            while self.mic_queue.len() >= chunk_size {
                let data: Vec<f32> = self
                    .mic_queue
                    .drain(..chunk_size)
                    .map(|v| v * 0.7)
                    .collect();
                let _ = self.sender.send(EncoderMessage::Audio {
                    data,
                    timestamp: Instant::now(),
                });
            }
            return;
        }

        // 如果只开了系统音
        if !self.enable_mic && self.enable_sys {
            while self.sys_queue.len() >= chunk_size {
                let data: Vec<f32> = self
                    .sys_queue
                    .drain(..chunk_size)
                    .map(|v| v * 0.7)
                    .collect();
                let _ = self.sender.send(EncoderMessage::Audio {
                    data,
                    timestamp: Instant::now(),
                });
            }
            return;
        }

        // 如果两者都开了，对齐混音
        loop {
            let mic_len = self.mic_queue.len();
            let sys_len = self.sys_queue.len();

            if mic_len >= chunk_size && sys_len >= chunk_size {
                let mic_drain = self.mic_queue.drain(..chunk_size);
                let sys_drain = self.sys_queue.drain(..chunk_size);
                let mixed: Vec<f32> = mic_drain
                    .zip(sys_drain)
                    .map(|(m, s)| (m + s) * 0.7)
                    .collect();

                let _ = self.sender.send(EncoderMessage::Audio {
                    data: mixed,
                    timestamp: Instant::now(),
                });
                continue;
            }

            let max_accum = 48000 * 2 * 15 / 100; // 150ms
            if mic_len > max_accum && sys_len < chunk_size {
                let count = std::cmp::min(mic_len, chunk_size);
                let data: Vec<f32> = self.mic_queue.drain(..count).map(|v| v * 0.7).collect();
                let _ = self.sender.send(EncoderMessage::Audio {
                    data,
                    timestamp: Instant::now(),
                });
                continue;
            }

            if sys_len > max_accum && mic_len < chunk_size {
                let count = std::cmp::min(sys_len, chunk_size);
                let data: Vec<f32> = self.sys_queue.drain(..count).map(|v| v * 0.7).collect();
                let _ = self.sender.send(EncoderMessage::Audio {
                    data,
                    timestamp: Instant::now(),
                });
                continue;
            }

            break;
        }
    }
}

fn build_stream<F>(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    mut data_callback: F,
) -> Result<cpal::Stream, String>
where
    F: FnMut(&[f32]) + Send + 'static,
{
    use cpal::traits::DeviceTrait;
    let err_fn = |err| eprintln!("Audio stream error: {}", err);
    let stream_config = config.config();
    match config.sample_format() {
        cpal::SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _| {
                use cpal::Sample;
                let f32_data: Vec<f32> = data.iter().map(|&s| s.to_float_sample()).collect();
                data_callback(&f32_data);
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &stream_config,
            move |data: &[u16], _| {
                use cpal::Sample;
                let f32_data: Vec<f32> = data.iter().map(|&s| s.to_float_sample()).collect();
                data_callback(&f32_data);
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                data_callback(data);
            },
            err_fn,
            None,
        ),
        _sample_format => Err(cpal::BuildStreamError::DeviceNotAvailable),
    }
    .map_err(|e| e.to_string())
}

pub struct AudioCaptureManager {
    mic_stream: Option<cpal::Stream>,
    sys_stream: Option<cpal::Stream>,
}

unsafe impl Send for AudioCaptureManager {}
unsafe impl Sync for AudioCaptureManager {}

impl AudioCaptureManager {
    pub fn start(
        config: &RecordConfig,
        sender: SyncSender<EncoderMessage>,
        is_paused: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let enable_mic = config.record_audio;
        let enable_sys = config.record_system_sound;

        if !enable_mic && !enable_sys {
            return Ok(Self {
                mic_stream: None,
                sys_stream: None,
            });
        }

        use cpal::traits::HostTrait;
        let host = cpal::default_host();
        let mixer = Arc::new(Mutex::new(AudioMixer::new(
            sender.clone(),
            enable_mic,
            enable_sys,
        )));

        let mic_stream = if enable_mic {
            let device = host
                .default_input_device()
                .ok_or_else(|| "无法获取默认麦克风输入设备".to_string())?;
            use cpal::traits::DeviceTrait;
            let supported_config = device
                .default_input_config()
                .map_err(|e| format!("获取麦克风输入配置失败: {:?}", e))?;

            let channels = supported_config.channels();
            let sample_rate = supported_config.sample_rate().0;
            let mut resampler = AudioResampler::new(sample_rate, 48000);
            let mixer_clone = mixer.clone();
            let is_paused_clone = is_paused.clone();

            let data_callback = move |data: &[f32]| {
                if is_paused_clone.load(Ordering::Acquire) {
                    return;
                }
                let mut output = Vec::new();
                resampler.process(data, channels, &mut output);
                mixer_clone.lock().unwrap().push_mic(&output);
            };

            let stream = build_stream(&device, &supported_config, data_callback)
                .map_err(|e| format!("打开麦克风流失败: {}", e))?;
            Some(stream)
        } else {
            None
        };

        let sys_stream = if enable_sys {
            let device = host
                .default_output_device()
                .ok_or_else(|| "无法获取默认输出设备以录制系统声音".to_string())?;
            use cpal::traits::DeviceTrait;
            let supported_config = device
                .default_input_config()
                .map_err(|e| format!("获取系统声音捕获配置失败: {:?}", e))?;

            let channels = supported_config.channels();
            let sample_rate = supported_config.sample_rate().0;
            let mut resampler = AudioResampler::new(sample_rate, 48000);
            let mixer_clone = mixer.clone();
            let is_paused_clone = is_paused.clone();

            let data_callback = move |data: &[f32]| {
                if is_paused_clone.load(Ordering::Acquire) {
                    return;
                }
                let mut output = Vec::new();
                resampler.process(data, channels, &mut output);
                mixer_clone.lock().unwrap().push_sys(&output);
            };

            let stream = build_stream(&device, &supported_config, data_callback)
                .map_err(|e| format!("打开系统回环音频捕获流失败: {}", e))?;
            Some(stream)
        } else {
            None
        };

        if let Some(ref s) = mic_stream {
            use cpal::traits::StreamTrait;
            let _ = s.play().map_err(|e| format!("启动麦克风失败: {:?}", e))?;
        }
        if let Some(ref s) = sys_stream {
            use cpal::traits::StreamTrait;
            let _ = s
                .play()
                .map_err(|e| format!("启动系统音频捕获失败: {:?}", e))?;
        }

        Ok(Self {
            mic_stream,
            sys_stream,
        })
    }

    pub fn stop(&mut self) {
        if let Some(s) = self.mic_stream.take() {
            use cpal::traits::StreamTrait;
            let _ = s.pause();
        }
        if let Some(s) = self.sys_stream.take() {
            use cpal::traits::StreamTrait;
            let _ = s.pause();
        }
    }
}
