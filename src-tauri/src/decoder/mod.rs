use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub sample_rate: u32,
    pub channels: usize,
    pub samples: Arc<[f32]>, // Interleaved stereo audio
    pub duration_seconds: f64,
}

pub fn decode_audio_bytes(
    bytes: Vec<u8>,
    extension_hint: Option<&str>,
    target_sample_rate: u32,
) -> Result<DecodedAudio, String> {
    let cursor = Cursor::new(bytes);
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
    decode_from_media_source_stream(mss, extension_hint, target_sample_rate)
}

pub fn decode_audio_file(file_path: &Path, target_sample_rate: u32) -> Result<DecodedAudio, String> {
    let src = File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let ext = file_path.extension().and_then(|e| e.to_str());
    decode_from_media_source_stream(mss, ext, target_sample_rate)
}

fn decode_from_media_source_stream(
    mss: MediaSourceStream,
    extension_hint: Option<&str>,
    target_sample_rate: u32,
) -> Result<DecodedAudio, String> {
    let mut hint = Hint::new();
    if let Some(ext) = extension_hint {
        hint.with_extension(ext);
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| format!("Unsupported format or failed probe: {}", e))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "No supported audio tracks found".to_string())?;

    let track_id = track.id;
    let original_sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| "Unknown sample rate in audio track".to_string())?;
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(2)
        .max(1);

    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| format!("Unsupported codec: {}", e))?;

    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut raw_interleaved: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(Error::ResetRequired) => continue,
            Err(e) => return Err(format!("Error decoding packet: {}", e)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                if sample_buf.is_none() {
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;
                    sample_buf = Some(SampleBuffer::new(duration, spec));
                }

                if let Some(ref mut buf) = sample_buf {
                    buf.copy_interleaved_ref(decoded);
                    raw_interleaved.extend_from_slice(buf.samples());
                }
            }
            Err(Error::DecodeError(e)) => log::warn!("Decode frame error: {}", e),
            Err(e) => return Err(format!("Fatal decode error: {}", e)),
        }
    }

    if raw_interleaved.is_empty() {
        return Err("Decoded buffer is empty".to_string());
    }

    // Convert mono to stereo if needed
    let stereo_samples = if channels == 1 {
        let mut stereo = Vec::with_capacity(raw_interleaved.len() * 2);
        for &s in &raw_interleaved {
            stereo.push(s);
            stereo.push(s);
        }
        stereo
    } else {
        raw_interleaved
    };

    // Resample if original sample rate differs from hardware target
    let final_samples = if original_sample_rate != target_sample_rate {
        resample_stereo_audio(&stereo_samples, original_sample_rate, target_sample_rate)?
    } else {
        stereo_samples
    };

    let total_frames = final_samples.len() / 2;
    let duration_seconds = (total_frames as f64) / (target_sample_rate as f64);

    Ok(DecodedAudio {
        sample_rate: target_sample_rate,
        channels: 2,
        samples: final_samples.into(),
        duration_seconds,
    })
}

fn resample_stereo_audio(
    interleaved: &[f32],
    from_rate: u32,
    to_rate: u32,
) -> Result<Vec<f32>, String> {
    let num_frames = interleaved.len() / 2;
    let mut left_in = Vec::with_capacity(num_frames);
    let mut right_in = Vec::with_capacity(num_frames);

    for chunk in interleaved.chunks_exact(2) {
        left_in.push(chunk[0]);
        right_in.push(chunk[1]);
    }

    let params = SincInterpolationParameters {
        sinc_len: 64,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 128,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = SincFixedIn::<f32>::new(
        to_rate as f64 / from_rate as f64,
        2.0,
        params,
        left_in.len(),
        2,
    ).map_err(|e| format!("Failed to create resampler: {}", e))?;

    let waves_in = vec![left_in, right_in];
    let waves_out = resampler.process(&waves_in, None)
        .map_err(|e| format!("Resampling error: {}", e))?;

    let out_len = waves_out[0].len();
    let mut out_interleaved = Vec::with_capacity(out_len * 2);
    for i in 0..out_len {
        out_interleaved.push(waves_out[0][i]);
        out_interleaved.push(waves_out[1][i]);
    }

    Ok(out_interleaved)
}
