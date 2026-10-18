use super::biquad::{BiquadFilter, BiquadFilterType};
use super::{AudioProcessor, ParameterMeta, PedalMeta};
use std::f32::consts::PI;

const MAX_DELAY_SECONDS: f32 = 2.5;

pub struct Delay {
    sample_rate: f32,
    enabled: bool,
    time_ms: f32,
    feedback: f32,
    mix: f32,
    tone: f32,
    flutter: f32,
    // Buffer
    buffer: Vec<f32>,
    write_pos: usize,
    damp_filter: BiquadFilter,
    // LFO for subtle analog tape flutter
    lfo_phase: f32,
}

impl Default for Delay {
    fn default() -> Self {
        Self::new()
    }
}

impl Delay {
    pub const PARAM_TIME: u32 = 0;
    pub const PARAM_FEEDBACK: u32 = 1;
    pub const PARAM_MIX: u32 = 2;
    pub const PARAM_TONE: u32 = 3;
    pub const PARAM_FLUTTER: u32 = 4;

    pub fn new() -> Self {
        let max_samples = (MAX_DELAY_SECONDS * 192000.0) as usize;
        let mut delay = Self {
            sample_rate: 48000.0,
            enabled: true,
            time_ms: 380.0,
            feedback: 45.0,
            mix: 35.0,
            tone: 60.0,
            flutter: 10.0,
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            damp_filter: BiquadFilter::new(),
            lfo_phase: 0.0,
        };
        delay.update_filter();
        delay
    }

    fn update_filter(&mut self) {
        let cutoff = 1500.0 + (self.tone / 100.0).powf(1.5) * 6500.0;
        self.damp_filter.configure(
            BiquadFilterType::LowPass,
            self.sample_rate,
            cutoff,
            0.707,
            0.0,
        );
    }

    // Hermite 4-point cubic interpolation for fractional delay
    #[inline(always)]
    fn read_interpolated(&self, delay_samples: f32) -> f32 {
        let buf_len = self.buffer.len();
        let read_pos = (self.write_pos as f32) - delay_samples;
        let read_pos_wrapped = if read_pos < 0.0 {
            read_pos + (buf_len as f32) * ((read_pos.abs() / buf_len as f32).ceil() + 1.0)
        } else {
            read_pos
        } % (buf_len as f32);

        let idx1 = read_pos_wrapped.floor() as usize % buf_len;
        let frac = read_pos_wrapped - (idx1 as f32);

        let idx0 = if idx1 == 0 { buf_len - 1 } else { idx1 - 1 };
        let idx2 = (idx1 + 1) % buf_len;
        let idx3 = (idx2 + 1) % buf_len;

        let y0 = self.buffer[idx0];
        let y1 = self.buffer[idx1];
        let y2 = self.buffer[idx2];
        let y3 = self.buffer[idx3];

        // Hermite interpolation polynomial
        let c0 = y1;
        let c1 = 0.5 * (y2 - y0);
        let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

        ((c3 * frac + c2) * frac + c1) * frac + c0
    }
}

impl AudioProcessor for Delay {
    fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate.max(8000.0);
        let needed_size = (MAX_DELAY_SECONDS * self.sample_rate) as usize + 32;
        if self.buffer.len() < needed_size {
            self.buffer.resize(needed_size, 0.0);
        }
        self.update_filter();
        self.reset();
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.damp_filter.reset();
        self.lfo_phase = 0.0;
    }

    fn set_parameter(&mut self, param_id: u32, value: f32) {
        match param_id {
            Self::PARAM_TIME => self.time_ms = value.clamp(10.0, 2000.0),
            Self::PARAM_FEEDBACK => self.feedback = value.clamp(0.0, 110.0),
            Self::PARAM_MIX => self.mix = value.clamp(0.0, 100.0),
            Self::PARAM_TONE => {
                self.tone = value.clamp(0.0, 100.0);
                self.update_filter();
            }
            Self::PARAM_FLUTTER => self.flutter = value.clamp(0.0, 100.0),
            _ => {}
        }
    }

    fn get_parameter(&self, param_id: u32) -> f32 {
        match param_id {
            Self::PARAM_TIME => self.time_ms,
            Self::PARAM_FEEDBACK => self.feedback,
            Self::PARAM_MIX => self.mix,
            Self::PARAM_TONE => self.tone,
            Self::PARAM_FLUTTER => self.flutter,
            _ => 0.0,
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn process(&mut self, buffer: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let wet_mix = self.mix / 100.0;
        let dry_mix = 1.0 - wet_mix * 0.5;
        let fb_factor = (self.feedback / 100.0).min(1.1);
        let buf_len = self.buffer.len();

        let lfo_inc = 2.0 * PI * 1.8 / self.sample_rate; // 1.8 Hz subtle flutter
        let flutter_depth = (self.flutter / 100.0) * 1.5;

        for sample in buffer.iter_mut() {
            let dry = *sample;

            // Compute modulated delay time in samples
            let flutter_mod = (self.lfo_phase.sin()) * flutter_depth;
            self.lfo_phase += lfo_inc;
            if self.lfo_phase >= 2.0 * PI {
                self.lfo_phase -= 2.0 * PI;
            }

            let current_delay_ms = (self.time_ms + flutter_mod).clamp(5.0, 2000.0);
            let delay_samples = current_delay_ms * 0.001 * self.sample_rate;

            // Read from circular delay buffer with Hermite interpolation
            let delayed = self.read_interpolated(delay_samples);

            // Filter the delayed signal for analog warmth
            let filtered_delay = self.damp_filter.process_sample(delayed);

            // Soft-clip feedback to prevent digital harshness on self-oscillation
            let fb_sample = (dry + filtered_delay * fb_factor).tanh();

            // Write to circular buffer
            self.buffer[self.write_pos] = fb_sample;
            self.write_pos = (self.write_pos + 1) % buf_len;

            // Wet/Dry mix output
            *sample = dry * dry_mix + filtered_delay * wet_mix;
        }
    }

    fn meta(&self) -> PedalMeta {
        PedalMeta {
            id: "analog_delay".to_string(),
            name: "Analog Delay".to_string(),
            category: "Delay".to_string(),
            enabled: self.enabled,
            parameters: vec![
                ParameterMeta {
                    id: Self::PARAM_TIME,
                    name: "Time".to_string(),
                    min: 10.0,
                    max: 2000.0,
                    default_value: 380.0,
                    unit: "ms".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_FEEDBACK,
                    name: "Feedback".to_string(),
                    min: 0.0,
                    max: 110.0,
                    default_value: 45.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_MIX,
                    name: "Mix".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 35.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_TONE,
                    name: "Damp".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 60.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_FLUTTER,
                    name: "Flutter".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 10.0,
                    unit: "%".to_string(),
                },
            ],
        }
    }
}
