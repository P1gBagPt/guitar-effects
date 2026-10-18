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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTelemetry {
    pub input_peak_db: f32,
    pub input_rms_db: f32,
    pub output_peak_db: f32,
    pub output_rms_db: f32,
    pub playhead_seconds: f64,
    pub is_playing: bool,
    pub is_recording: bool,
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
    output_peak: Arc<AtomicF32>,
    output_rms: Arc<AtomicF32>,
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
            output_peak: Arc::new(AtomicF32::new(0.0)),
            output_rms: Arc::new(AtomicF32::new(0.0)),
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
        let out_peak = to_db(self.output_peak.load(Ordering::Relaxed));
        let out_rms = to_db(self.output_rms.load(Ordering::Relaxed));

        let sr = self.sample_rate.load(Ordering::Relaxed) as f64;
        let frames = self.playhead_sample.load(Ordering::Relaxed) as f64;
        let playhead_secs = if sr > 0.0 { frames / sr } else { 0.0 };

        AudioTelemetry {
            input_peak_db: in_peak,
            input_rms_db: in_rms,
            output_peak_db: out_peak,
            output_rms_db: out_rms,
            playhead_seconds: playhead_secs,
            is_playing: self.is_playing.load(Ordering::Relaxed),
            is_recording: self.recorder.lock().is_recording(),
        }
    }

    pub fn start_engine(
        &mut self,
        input_device_name: Option<&str>,
        output_device_name: Option<&str>,
        target_sample_rate: u32,
        buffer_size_samples: u32,
    ) -> Result<(), String> {
        self.stop_engine();

        let in_dev = self
            .device_manager
            .find_input_device(input_device_name)
            .ok_or_else(|| "Failed to find audio input device".to_string())?;

        let out_dev = self
            .device_manager
            .find_output_device(output_device_name)
            .ok_or_else(|| "Failed to find audio output device".to_string())?;

        let in_cfg = StreamConfig {
            channels: 1, // Guitar DI mono
            sample_rate: SampleRate(target_sample_rate),
            buffer_size: BufferSize::Fixed(buffer_size_samples),
        };

        let out_cfg = StreamConfig {
            channels: 2, // Stereo output
            sample_rate: SampleRate(target_sample_rate),
            buffer_size: BufferSize::Fixed(buffer_size_samples),
        };

        self.sample_rate.store(target_sample_rate as u64, Ordering::SeqCst);
        self.pedal_chain.lock().prepare(target_sample_rate as f32, buffer_size_samples as usize);

        // Lock-free input -> output communication ringbuffer
        let (mut in_prod, mut out_cons) = create_audio_ringbuffer::<f32>(buffer_size_samples as usize * 16);

        // Input stream callback
        let in_peak_ref = self.input_peak.clone();
        let in_rms_ref = self.input_rms.clone();

        let in_stream = in_dev
            .build_input_stream(
                &in_cfg,
                move |data: &[f32], _| {
                    let mut peak = 0.0_f32;
                    let mut sum_sq = 0.0_f32;

                    for &sample in data {
                        let abs = sample.abs();
                        if abs > peak {
                            peak = abs;
                        }
                        sum_sq += sample * sample;
                        let _ = in_prod.push(sample);
                    }

                    let rms = (sum_sq / data.len().max(1) as f32).sqrt();
                    in_peak_ref.store(peak, Ordering::Relaxed);
                    in_rms_ref.store(rms, Ordering::Relaxed);
                },
                |err| log::error!("Audio input stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to create input audio stream: {}", e))?;

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

        // Local processing buffer
        let mut dsp_block = vec![0.0_f32; buffer_size_samples as usize * 2];

        let out_stream = out_dev
            .build_output_stream(
                &out_cfg,
                move |data: &mut [f32], _| {
                    let num_frames = data.len() / 2;
                    if dsp_block.len() < num_frames {
                        dsp_block.resize(num_frames, 0.0);
                    }

                    // 1. Pull guitar samples from ringbuffer
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

                        data[i * 2] = left_out;
                        data[i * 2 + 1] = right_out;

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
            )
            .map_err(|e| format!("Failed to create output audio stream: {}", e))?;

        in_stream.play().map_err(|e| format!("Failed to start input audio stream: {}", e))?;
        out_stream.play().map_err(|e| format!("Failed to start output audio stream: {}", e))?;

        self.input_stream = Some(in_stream);
        self.output_stream = Some(out_stream);

        Ok(())
    }

    pub fn stop_engine(&mut self) {
        self.input_stream.take();
        self.output_stream.take();
    }
}
