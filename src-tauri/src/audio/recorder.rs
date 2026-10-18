use hound::{WavSpec, WavWriter};
use rtrb::Consumer;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub struct DiskRecorder {
    is_recording: Arc<AtomicBool>,
    worker_handle: Option<JoinHandle<()>>,
}

impl DiskRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            worker_handle: None,
        }
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    pub fn start_recording(
        &mut self,
        file_path: PathBuf,
        sample_rate: u32,
        channels: u16,
        mut consumer: Consumer<f32>,
    ) -> Result<(), String> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Err("Already recording".to_string());
        }

        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = WavWriter::create(&file_path, spec)
            .map_err(|e| format!("Failed to create WAV file: {}", e))?;

        let is_recording_flag = self.is_recording.clone();
        is_recording_flag.store(true, Ordering::SeqCst);

        let handle = thread::Builder::new()
            .name("disk-recorder-thread".to_string())
            .spawn(move || {
                let mut buf_writer: WavWriter<BufWriter<File>> = writer;
                let max_int24 = 8388607.0_f32; // 2^23 - 1

                while is_recording_flag.load(Ordering::Relaxed) {
                    while let Ok(sample) = consumer.pop() {
                        let sample_clamped = sample.clamp(-1.0, 1.0);
                        let sample_int = (sample_clamped * max_int24) as i32;
                        let _ = buf_writer.write_sample(sample_int);
                    }
                    thread::sleep(std::time::Duration::from_millis(15));
                }

                // Drain remaining samples in queue
                while let Ok(sample) = consumer.pop() {
                    let sample_clamped = sample.clamp(-1.0, 1.0);
                    let sample_int = (sample_clamped * max_int24) as i32;
                    let _ = buf_writer.write_sample(sample_int);
                }

                let _ = buf_writer.flush();
            })
            .map_err(|e| format!("Failed to spawn disk recorder thread: {}", e))?;

        self.worker_handle = Some(handle);
        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<(), String> {
        if !self.is_recording.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_recording.store(false, Ordering::SeqCst);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
        Ok(())
    }
}
