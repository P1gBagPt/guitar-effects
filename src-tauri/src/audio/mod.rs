pub mod device;
pub mod recorder;
pub mod ringbuffer;

use crate::decoder::DecodedAudio;
use crate::dsp::chain::PedalChain;
use crate::dsp::{AudioProcessor, PedalMeta};
use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, Stream, StreamConfig};
use device::{AudioHostInfo, DeviceManager};
use parking_lot::Mutex;
use recorder::DiskRecorder;
use ringbuffer::create_audio_ringbuffer;
use rtrb::Producer;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTelemetry {
    pub input_peak_db: f32,
    pub input_rms_db: f32,
    pub input_ch1_peak_db: f32,
    pub input_ch2_peak_db: f32,
    pub output_peak_db: f32,
    pub output_rms_db: f32,
    pub playhead_seconds: f64,
    pub is_playing: bool,
    pub is_recording: bool,
    pub input_active: bool,
    pub input_error: Option<String>,
}

pub struct AudioEngine {
    device_manager: DeviceManager,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    pedal_chain: Arc<Mutex<PedalChain>>,
    recorder: Arc<Mutex<DiskRecorder>>,
    rec_producer: Arc<Mutex<Option<Producer<f32>>>>,
    // Telemetry atomics
    input_peak: Arc<AtomicF32>,
    input_rms: Arc<AtomicF32>,
    input_ch1_peak: Arc<AtomicF32>,
    input_ch2_peak: Arc<AtomicF32>,
    output_peak: Arc<AtomicF32>,
    output_rms: Arc<AtomicF32>,
    input_active: Arc<AtomicBool>,
    input_error: Arc<Mutex<Option<String>>>,
    // Input channel selector (0 = Ch 1, 1 = Ch 2, 2 = Sum/Both)
    input_channel: Arc<AtomicU32>,
    // Transport & Volumes
    guitar_volume: Arc<AtomicF32>,
    backing_volume: Arc<AtomicF32>,
    master_volume: Arc<AtomicF32>,
    is_playing: Arc<AtomicBool>,
    playhead_sample: Arc<AtomicU64>,
    sample_rate: Arc<AtomicU64>,
    backing_audio: Arc<Mutex<Option<DecodedAudio>>>,
    dry_monitor: Arc<AtomicBool>,
}

unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            device_manager: DeviceManager::new(),
            input_stream: None,
            output_stream: None,
            pedal_chain: Arc::new(Mutex::new(PedalChain::new())),
            recorder: Arc::new(Mutex::new(DiskRecorder::new())),
            rec_producer: Arc::new(Mutex::new(None)),
            input_peak: Arc::new(AtomicF32::new(0.0)),
            input_rms: Arc::new(AtomicF32::new(0.0)),
            input_ch1_peak: Arc::new(AtomicF32::new(0.0)),
            input_ch2_peak: Arc::new(AtomicF32::new(0.0)),
            output_peak: Arc::new(AtomicF32::new(0.0)),
            output_rms: Arc::new(AtomicF32::new(0.0)),
            input_active: Arc::new(AtomicBool::new(false)),
            input_error: Arc::new(Mutex::new(None)),
            input_channel: Arc::new(AtomicU32::new(2)), // Default: 2 (Sum/Both channels)
            guitar_volume: Arc::new(AtomicF32::new(1.0)),
            backing_volume: Arc::new(AtomicF32::new(1.0)),
            master_volume: Arc::new(AtomicF32::new(1.0)),
            is_playing: Arc::new(AtomicBool::new(false)),
            playhead_sample: Arc::new(AtomicU64::new(0)),
            sample_rate: Arc::new(AtomicU64::new(48000)),
            backing_audio: Arc::new(Mutex::new(None)),
            dry_monitor: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_input_channel(&self, ch: u32) {
        self.input_channel.store(ch, Ordering::Relaxed);
    }

    pub fn get_input_channel(&self) -> u32 {
        self.input_channel.load(Ordering::Relaxed)
    }

    pub fn get_host_info(&self) -> AudioHostInfo {
        self.device_manager.get_host_info()
    }

    pub fn get_pedals(&self) -> Vec<PedalMeta> {
        self.pedal_chain.lock().get_pedal_list()
    }

    pub fn set_pedal_enabled(&self, pedal_id: &str, enabled: bool) -> bool {
        self.pedal_chain.lock().set_pedal_enabled(pedal_id, enabled)
    }

    pub fn set_pedal_param(&self, pedal_id: &str, param_id: u32, value: f32) -> bool {
        self.pedal_chain.lock().set_pedal_param(pedal_id, param_id, value)
    }

    pub fn reorder_pedals(&self, new_order: &[String]) {
        self.pedal_chain.lock().reorder_pedals(new_order);
    }

    pub fn set_guitar_volume(&self, vol: f32) {
        self.guitar_volume.store(vol.clamp(0.0, 2.0), Ordering::Relaxed);
    }

    pub fn set_backing_volume(&self, vol: f32) {
        self.backing_volume.store(vol.clamp(0.0, 2.0), Ordering::Relaxed);
    }

    pub fn set_master_volume(&self, vol: f32) {
        self.master_volume.store(vol.clamp(0.0, 2.0), Ordering::Relaxed);
    }

    pub fn set_dry_monitor(&self, enabled: bool) {
        self.dry_monitor.store(enabled, Ordering::Relaxed);
    }

    pub fn load_backing_track(&self, decoded: DecodedAudio) {
        *self.backing_audio.lock() = Some(decoded);
        self.playhead_sample.store(0, Ordering::SeqCst);
    }

    pub fn play_transport(&self) {
        self.is_playing.store(true, Ordering::SeqCst);
    }

    pub fn pause_transport(&self) {
        self.is_playing.store(false, Ordering::SeqCst);
    }

    pub fn seek_transport(&self, seconds: f64) {
        let sr = self.sample_rate.load(Ordering::Relaxed) as f64;
        let frame = (seconds.max(0.0) * sr) as u64;
        self.playhead_sample.store(frame, Ordering::SeqCst);
    }

    pub fn start_recording(&self, path: PathBuf) -> Result<(), String> {
        let (producer, consumer) = create_audio_ringbuffer::<f32>(131072); // ~1.3 seconds at 96k stereo
        *self.rec_producer.lock() = Some(producer);
        let sr = self.sample_rate.load(Ordering::Relaxed) as u32;
        self.recorder.lock().start_recording(path, sr, 2, consumer)
    }

    pub fn stop_recording(&self) -> Result<(), String> {
        self.rec_producer.lock().take();
        self.recorder.lock().stop_recording()
    }

    pub fn get_telemetry(&self) -> AudioTelemetry {
        let to_db = |lin: f32| -> f32 {
            if lin <= 1e-5 {
                -96.0
            } else {
                (20.0 * lin.log10()).max(-96.0).min(6.0)
            }
        };

        let in_peak = to_db(self.input_peak.load(Ordering::Relaxed));
        let in_rms = to_db(self.input_rms.load(Ordering::Relaxed));
        let in_ch1_peak = to_db(self.input_ch1_peak.load(Ordering::Relaxed));
        let in_ch2_peak = to_db(self.input_ch2_peak.load(Ordering::Relaxed));
        let out_peak = to_db(self.output_peak.load(Ordering::Relaxed));
        let out_rms = to_db(self.output_rms.load(Ordering::Relaxed));

        let sr = self.sample_rate.load(Ordering::Relaxed) as f64;
        let frames = self.playhead_sample.load(Ordering::Relaxed) as f64;
        let playhead_secs = if sr > 0.0 { frames / sr } else { 0.0 };

        AudioTelemetry {
            input_peak_db: in_peak,
            input_rms_db: in_rms,
            input_ch1_peak_db: in_ch1_peak,
            input_ch2_peak_db: in_ch2_peak,
            output_peak_db: out_peak,
            output_rms_db: out_rms,
            playhead_seconds: playhead_secs,
            is_playing: self.is_playing.load(Ordering::Relaxed),
            is_recording: self.recorder.lock().is_recording(),
            input_active: self.input_active.load(Ordering::Relaxed),
            input_error: self.input_error.lock().clone(),
        }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::Relaxed) as u32
    }

    pub fn start_engine(
        &mut self,
        input_device_name: Option<&str>,
        output_device_name: Option<&str>,
        target_sample_rate: u32,
        buffer_size_samples: u32,
    ) -> Result<(), String> {
        self.stop_engine();

        // 1. Output device & stream configuration (REQUIRED)
        let out_dev = self
            .device_manager
            .find_output_device(output_device_name)
            .ok_or_else(|| "Failed to find audio output device".to_string())?;

        let default_out_cfg = out_dev
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;

        let out_channels = default_out_cfg.channels();
        // Check if device supports target_sample_rate; otherwise use default device sample rate
        let out_cfg: StreamConfig = if target_sample_rate > 0 && target_sample_rate != default_out_cfg.sample_rate().0 {
            let mut supported = false;
            if let Ok(configs) = out_dev.supported_output_configs() {
                for c in configs {
                    if target_sample_rate >= c.min_sample_rate().0 && target_sample_rate <= c.max_sample_rate().0 {
                        supported = true;
                        break;
                    }
                }
            }
            if supported {
                StreamConfig {
                    channels: out_channels,
                    sample_rate: SampleRate(target_sample_rate),
                    buffer_size: BufferSize::Default,
                }
            } else {
                default_out_cfg.into()
            }
        } else {
            default_out_cfg.into()
        };

        let active_sr = out_cfg.sample_rate.0;
        self.sample_rate.store(active_sr as u64, Ordering::SeqCst);
        self.pedal_chain.lock().prepare(active_sr as f32, buffer_size_samples as usize);

        // Ringbuffer for guitar DI signal
        let (in_prod, mut out_cons) = create_audio_ringbuffer::<f32>(buffer_size_samples as usize * 16);
        let in_prod_arc = Arc::new(Mutex::new(in_prod));

        // Output stream callback
        let pedal_chain_ref = self.pedal_chain.clone();
        let out_peak_ref = self.output_peak.clone();
        let out_rms_ref = self.output_rms.clone();
        let guitar_vol_ref = self.guitar_volume.clone();
        let backing_vol_ref = self.backing_volume.clone();
        let master_vol_ref = self.master_volume.clone();
        let is_playing_ref = self.is_playing.clone();
        let playhead_ref = self.playhead_sample.clone();
        let backing_audio_ref = self.backing_audio.clone();
        let rec_prod_ref = self.rec_producer.clone();
        let dry_monitor_ref = self.dry_monitor.clone();

        let num_channels = out_cfg.channels as usize;
        let mut dsp_block = vec![0.0_f32; buffer_size_samples as usize * 2];

        let out_stream = out_dev.build_output_stream(
            &out_cfg,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let num_frames = data.len() / num_channels;
                if dsp_block.len() < num_frames {
                    dsp_block.resize(num_frames, 0.0);
                }

                // 1. Pull guitar samples from input ringbuffer
                for i in 0..num_frames {
                    dsp_block[i] = out_cons.pop().unwrap_or(0.0);
                }

                // 2. Process guitar signal through PedalChain
                if !dry_monitor_ref.load(Ordering::Relaxed) {
                    if let Some(mut chain) = pedal_chain_ref.try_lock() {
                        chain.process(&mut dsp_block[..num_frames]);
                    }
                }

                let g_vol = guitar_vol_ref.load(Ordering::Relaxed);
                let b_vol = backing_vol_ref.load(Ordering::Relaxed);
                let m_vol = master_vol_ref.load(Ordering::Relaxed);

                // 3. Mix Backing Track if playing
                let is_playing = is_playing_ref.load(Ordering::Relaxed);
                let current_frame = playhead_ref.load(Ordering::Relaxed);
                let mut next_frame = current_frame;

                let backing_guard = backing_audio_ref.try_lock();
                let backing_samples = backing_guard
                    .as_ref()
                    .and_then(|g| g.as_ref().map(|b| &b.samples));

                let mut out_peak = 0.0_f32;
                let mut out_sum_sq = 0.0_f32;

                let mut rec_guard = rec_prod_ref.try_lock();

                for i in 0..num_frames {
                    let wet_guitar = dsp_block[i] * g_vol;

                    let (b_left, b_right) = if is_playing {
                        if let Some(backing) = backing_samples {
                            let sample_idx = ((current_frame + i as u64) as usize) * 2;
                            if sample_idx + 1 < backing.len() {
                                (backing[sample_idx] * b_vol, backing[sample_idx + 1] * b_vol)
                            } else {
                                (0.0, 0.0)
                            }
                        } else {
                            (0.0, 0.0)
                        }
                    } else {
                        (0.0, 0.0)
                    };

                    let left_out = ((wet_guitar + b_left) * m_vol).clamp(-1.0, 1.0);
                    let right_out = ((wet_guitar + b_right) * m_vol).clamp(-1.0, 1.0);

                    data[i * num_channels] = left_out;
                    if num_channels > 1 {
                        data[i * num_channels + 1] = right_out;
                    }
                    for ch in 2..num_channels {
                        data[i * num_channels + ch] = 0.0;
                    }

                    // Telemetry metering
                    let abs_l = left_out.abs();
                    let abs_r = right_out.abs();
                    if abs_l > out_peak { out_peak = abs_l; }
                    if abs_r > out_peak { out_peak = abs_r; }
                    out_sum_sq += left_out * left_out + right_out * right_out;

                    // Recording path (lock-free push to disk worker)
                    if let Some(ref mut guard) = rec_guard {
                        if let Some(ref mut prod) = **guard {
                            let _ = prod.push(left_out);
                            let _ = prod.push(right_out);
                        }
                    }
                }

                if is_playing {
                    next_frame += num_frames as u64;
                    playhead_ref.store(next_frame, Ordering::Relaxed);
                }

                let out_rms = (out_sum_sq / (data.len().max(1) as f32)).sqrt();
                out_peak_ref.store(out_peak, Ordering::Relaxed);
                out_rms_ref.store(out_rms, Ordering::Relaxed);
            },
            |err| log::error!("Audio output stream error: {}", err),
            None,
        ).map_err(|e| format!("Failed to create output audio stream: {}", e))?;

        out_stream.play().map_err(|e| format!("Failed to start output audio stream: {}", e))?;
        self.output_stream = Some(out_stream);

        // 2. Input stream setup (Resilient sample rate negotiation & channel routing)
        self.input_active.store(false, Ordering::SeqCst);
        *self.input_error.lock() = None;

        if let Some(in_dev) = self.device_manager.find_input_device(input_device_name) {
            let in_dev_name = in_dev.name().unwrap_or_else(|_| "Audio Input Device".to_string());
            if let Ok(default_in_cfg) = in_dev.default_input_config() {
                let default_sr = default_in_cfg.sample_rate().0;
                let in_channels_count = default_in_cfg.channels() as usize;

                // Build list of candidate sample rates:
                // Try active_sr first if supported; then fall back to default_sr.
                let mut candidates = Vec::new();
                if let Ok(supported_configs) = in_dev.supported_input_configs() {
                    for c in supported_configs {
                        if active_sr >= c.min_sample_rate().0 && active_sr <= c.max_sample_rate().0 {
                            if !candidates.contains(&active_sr) {
                                candidates.push(active_sr);
                            }
                            break;
                        }
                    }
                }
                if !candidates.contains(&default_sr) {
                    candidates.push(default_sr);
                }

                let mut started_stream = None;
                let mut stream_err = None;

                for &candidate_sr in &candidates {
                    let in_cfg = StreamConfig {
                        channels: default_in_cfg.channels(),
                        sample_rate: SampleRate(candidate_sr),
                        buffer_size: BufferSize::Default,
                    };

                    let in_peak_ref = self.input_peak.clone();
                    let in_rms_ref = self.input_rms.clone();
                    let in_ch1_ref = self.input_ch1_peak.clone();
                    let in_ch2_ref = self.input_ch2_peak.clone();
                    let in_chan_mode = self.input_channel.clone();
                    let in_prod_ref = in_prod_arc.clone();
                    let mut resampler = LinearResampler::new(candidate_sr, active_sr);

                    let res = match default_in_cfg.sample_format() {
                        cpal::SampleFormat::F32 => in_dev.build_input_stream(
                            &in_cfg,
                            move |data: &[f32], _| {
                                let mut prod_guard = in_prod_ref.lock();
                                Self::process_input_frames::<f32>(
                                    data,
                                    in_channels_count,
                                    &in_chan_mode,
                                    &in_peak_ref,
                                    &in_rms_ref,
                                    &in_ch1_ref,
                                    &in_ch2_ref,
                                    &mut resampler,
                                    &mut *prod_guard,
                                    |&s| s,
                                );
                            },
                            |err| log::warn!("Audio input stream error: {}", err),
                            None,
                        ),
                        cpal::SampleFormat::I16 => in_dev.build_input_stream(
                            &in_cfg,
                            move |data: &[i16], _| {
                                let mut prod_guard = in_prod_ref.lock();
                                Self::process_input_frames::<i16>(
                                    data,
                                    in_channels_count,
                                    &in_chan_mode,
                                    &in_peak_ref,
                                    &in_rms_ref,
                                    &in_ch1_ref,
                                    &in_ch2_ref,
                                    &mut resampler,
                                    &mut *prod_guard,
                                    |&s| (s as f32) / 32768.0,
                                );
                            },
                            |err| log::warn!("Audio input stream error: {}", err),
                            None,
                        ),
                        cpal::SampleFormat::U16 => in_dev.build_input_stream(
                            &in_cfg,
                            move |data: &[u16], _| {
                                let mut prod_guard = in_prod_ref.lock();
                                Self::process_input_frames::<u16>(
                                    data,
                                    in_channels_count,
                                    &in_chan_mode,
                                    &in_peak_ref,
                                    &in_rms_ref,
                                    &in_ch1_ref,
                                    &in_ch2_ref,
                                    &mut resampler,
                                    &mut *prod_guard,
                                    |&s| ((s as f32) - 32768.0) / 32768.0,
                                );
                            },
                            |err| log::warn!("Audio input stream error: {}", err),
                            None,
                        ),
                        _ => Err(cpal::BuildStreamError::StreamConfigNotSupported),
                    };

                    match res {
                        Ok(stream) => {
                            if let Err(e) = stream.play() {
                                stream_err = Some(format!("Failed to start input stream: {}", e));
                            } else {
                                log::info!(
                                    "Guitar DI input stream active on '{}' ({} Hz, format: {:?})",
                                    in_dev_name,
                                    candidate_sr,
                                    default_in_cfg.sample_format()
                                );
                                started_stream = Some(stream);
                                break;
                            }
                        }
                        Err(e) => {
                            stream_err = Some(format!("Input config rejected ({} Hz): {}", candidate_sr, e));
                        }
                    }
                }

                if let Some(stream) = started_stream {
                    self.input_stream = Some(stream);
                    self.input_active.store(true, Ordering::SeqCst);
                } else if let Some(err) = stream_err {
                    log::warn!("Could not start input stream for '{}': {}", in_dev_name, err);
                    *self.input_error.lock() = Some(err);
                }
            } else {
                let err = format!("Could not read default input config for '{}'", in_dev_name);
                log::warn!("{}", err);
                *self.input_error.lock() = Some(err);
            }
        } else {
            let err = "No audio input device available".to_string();
            log::warn!("{}", err);
            *self.input_error.lock() = Some(err);
        }

        Ok(())
    }

    fn process_input_frames<T: Copy>(
        data: &[T],
        in_channels: usize,
        in_chan_mode: &AtomicU32,
        in_peak_ref: &AtomicF32,
        in_rms_ref: &AtomicF32,
        in_ch1_ref: &AtomicF32,
        in_ch2_ref: &AtomicF32,
        resampler: &mut LinearResampler,
        in_prod: &mut Producer<f32>,
        to_f32: impl Fn(&T) -> f32,
    ) {
        let mode = in_chan_mode.load(Ordering::Relaxed);
        let mut overall_peak = 0.0_f32;
        let mut sum_sq = 0.0_f32;
        let mut ch1_peak = 0.0_f32;
        let mut ch2_peak = 0.0_f32;

        let num_frames = data.len() / in_channels.max(1);

        for frame in data.chunks(in_channels) {
            let ch1 = frame.get(0).map(&to_f32).unwrap_or(0.0);
            let ch2 = frame.get(1).map(&to_f32).unwrap_or(0.0);

            let abs1 = ch1.abs();
            let abs2 = ch2.abs();
            if abs1 > ch1_peak {
                ch1_peak = abs1;
            }
            if abs2 > ch2_peak {
                ch2_peak = abs2;
            }

            let sample = match mode {
                0 => ch1, // Channel 1 (Input 1 / Left)
                1 => ch2, // Channel 2 (Input 2 / Right / Guitar)
                _ => {
                    // Mode 2 (Sum / Both)
                    if in_channels > 1 {
                        // If one channel is receiving guitar and other is idle/silent,
                        // pass the active signal at full level (0 dB) for optimal signal-to-noise
                        if abs1 > 0.0005 && abs2 < 0.0002 {
                            ch1
                        } else if abs2 > 0.0005 && abs1 < 0.0002 {
                            ch2
                        } else {
                            (ch1 + ch2) * 0.707
                        }
                    } else {
                        ch1
                    }
                }
            };

            let abs = sample.abs();
            if abs > overall_peak {
                overall_peak = abs;
            }
            sum_sq += sample * sample;

            resampler.process(sample, |resampled| {
                let _ = in_prod.push(resampled);
            });
        }

        let rms = (sum_sq / num_frames.max(1) as f32).sqrt();
        in_peak_ref.store(overall_peak, Ordering::Relaxed);
        in_rms_ref.store(rms, Ordering::Relaxed);
        in_ch1_ref.store(ch1_peak, Ordering::Relaxed);
        in_ch2_ref.store(ch2_peak, Ordering::Relaxed);
    }

    pub fn stop_engine(&mut self) {
        self.input_stream.take();
        self.output_stream.take();
        self.input_active.store(false, Ordering::SeqCst);
    }
}

/// Ultra low-latency linear resampler for sample rate bridging
struct LinearResampler {
    in_sr: f64,
    out_sr: f64,
    phase: f64,
    last_sample: f32,
}

impl LinearResampler {
    fn new(in_sr: u32, out_sr: u32) -> Self {
        Self {
            in_sr: in_sr as f64,
            out_sr: out_sr as f64,
            phase: 0.0,
            last_sample: 0.0,
        }
    }

    #[inline]
    fn process<F>(&mut self, curr_sample: f32, mut push_fn: F)
    where
        F: FnMut(f32),
    {
        if (self.in_sr - self.out_sr).abs() < 1.0 {
            push_fn(curr_sample);
            return;
        }

        let ratio = self.in_sr / self.out_sr;
        while self.phase < 1.0 {
            let interpolated = self.last_sample + (curr_sample - self.last_sample) * (self.phase as f32);
            push_fn(interpolated);
            self.phase += ratio;
        }
        self.phase -= 1.0;
        self.last_sample = curr_sample;
    }
}





