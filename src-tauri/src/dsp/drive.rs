use super::biquad::{BiquadFilter, BiquadFilterType};
use super::{AudioProcessor, ParameterMeta, PedalMeta};

pub struct Overdrive {
    sample_rate: f32,
    enabled: bool,
    drive: f32,
    tone: f32,
    level: f32,
    mode: f32, // 0 = Vintage Overdrive, 1 = Distortion
    // Tone filtering
    pre_filter: BiquadFilter,
    tone_filter: BiquadFilter,
    post_filter: BiquadFilter,
    // Oversampling anti-aliasing filters (2x oversampling)
    up_filter: BiquadFilter,
    down_filter: BiquadFilter,
    prev_sample: f32,
}

impl Default for Overdrive {
    fn default() -> Self {
        Self::new()
    }
}

impl Overdrive {
    pub const PARAM_DRIVE: u32 = 0;
    pub const PARAM_TONE: u32 = 1;
    pub const PARAM_LEVEL: u32 = 2;
    pub const PARAM_MODE: u32 = 3;

    pub fn new() -> Self {
        let mut od = Self {
            sample_rate: 48000.0,
            enabled: true,
            drive: 35.0,
            tone: 50.0,
            level: 70.0,
            mode: 0.0,
            pre_filter: BiquadFilter::new(),
            tone_filter: BiquadFilter::new(),
            post_filter: BiquadFilter::new(),
            up_filter: BiquadFilter::new(),
            down_filter: BiquadFilter::new(),
            prev_sample: 0.0,
        };
        od.update_filters();
        od
    }

    fn update_filters(&mut self) {
        let sr = self.sample_rate;
        // Guitar DI pre-shaping: gentle high-pass to tighten flubby bass (720 Hz cutoff)
        self.pre_filter.configure(
            BiquadFilterType::HighPass,
            sr,
            720.0,
            0.707,
            0.0,
        );

        // Tone knob controls low-pass / high-shelf cutoff from 1.5 kHz to 6.5 kHz
        let tone_freq = 1200.0 + (self.tone / 100.0).powf(1.8) * 6000.0;
        self.tone_filter.configure(
            BiquadFilterType::LowPass,
            sr,
            tone_freq,
            0.8,
            0.0,
        );

        // Smoothing post filter at 8 kHz to remove fizzy digital high end
        self.post_filter.configure(
            BiquadFilterType::LowPass,
            sr,
            7800.0,
            0.707,
            0.0,
        );

        // 2x oversampling filters (cutoff at nyquist / 2)
        self.up_filter.configure(
            BiquadFilterType::LowPass,
            sr * 2.0,
            sr * 0.45,
            0.707,
            0.0,
        );
        self.down_filter.configure(
            BiquadFilterType::LowPass,
            sr * 2.0,
            sr * 0.45,
            0.707,
            0.0,
        );
    }

    #[inline(always)]
    fn waveshape(&self, x: f32) -> f32 {
        if self.mode < 0.5 {
            // Vintage Tube Screamer / Bluesbreaker symmetrical soft-clipping
            let pre_gain = 1.0 + (self.drive / 10.0).powf(1.6);
            let driven = x * pre_gain;
            if driven.abs() < 1e-6 {
                driven
            } else {
                // Diode soft clipping curve
                (driven / (1.0 + driven.abs())).tanh()
            }
        } else {
            // Modern High-Gain Distortion with asymmetric diode pair clipping
            let pre_gain = 2.0 + (self.drive / 6.0).powf(1.8);
            let driven = x * pre_gain;
            if driven > 0.0 {
                (driven * 1.5).tanh()
            } else {
                (driven * 0.8).tanh() * 1.2
            }
        }
    }
}

impl AudioProcessor for Overdrive {
    fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate.max(8000.0);
        self.update_filters();
        self.reset();
    }

    fn reset(&mut self) {
        self.pre_filter.reset();
        self.tone_filter.reset();
        self.post_filter.reset();
        self.up_filter.reset();
        self.down_filter.reset();
        self.prev_sample = 0.0;
    }

    fn set_parameter(&mut self, param_id: u32, value: f32) {
        match param_id {
            Self::PARAM_DRIVE => self.drive = value.clamp(0.0, 100.0),
            Self::PARAM_TONE => {
                self.tone = value.clamp(0.0, 100.0);
                self.update_filters();
            }
            Self::PARAM_LEVEL => self.level = value.clamp(0.0, 100.0),
            Self::PARAM_MODE => self.mode = if value > 0.5 { 1.0 } else { 0.0 },
            _ => {}
        }
    }

    fn get_parameter(&self, param_id: u32) -> f32 {
        match param_id {
            Self::PARAM_DRIVE => self.drive,
            Self::PARAM_TONE => self.tone,
            Self::PARAM_LEVEL => self.level,
            Self::PARAM_MODE => self.mode,
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

        let out_level = (self.level / 100.0) * 1.5;

        for sample in buffer.iter_mut() {
            let input = *sample;

            // 1. Pre-shaping filter
            let pre = self.pre_filter.process_sample(input);

            // 2. 2x Oversampling: Linear interpolation for mid-point sample
            let mid_sample = (self.prev_sample + pre) * 0.5;
            self.prev_sample = pre;

            // Up-sample 1
            let up1 = self.up_filter.process_sample(pre * 2.0);
            let sat1 = self.waveshape(up1);

            // Up-sample 2
            let up2 = self.up_filter.process_sample(mid_sample * 2.0);
            let sat2 = self.waveshape(up2);

            // 3. Decimate / Down-sample with anti-aliasing filter
            let down1 = self.down_filter.process_sample(sat1);
            let _ = self.down_filter.process_sample(sat2);

            // 4. Tone stack and high-cut smoothing
            let tone_shaped = self.tone_filter.process_sample(down1);
            let post = self.post_filter.process_sample(tone_shaped);

            *sample = post * out_level;
        }
    }

    fn meta(&self) -> PedalMeta {
        PedalMeta {
            id: "vintage_drive".to_string(),
            name: "Vintage Overdrive".to_string(),
            category: "Distortion".to_string(),
            enabled: self.enabled,
            parameters: vec![
                ParameterMeta {
                    id: Self::PARAM_DRIVE,
                    name: "Drive".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 35.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_TONE,
                    name: "Tone".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 50.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_LEVEL,
                    name: "Level".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 70.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_MODE,
                    name: "Mode".to_string(),
                    min: 0.0,
                    max: 1.0,
                    default_value: 0.0,
                    unit: "type".to_string(),
                },
            ],
        }
    }
}
