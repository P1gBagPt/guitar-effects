use super::biquad::{BiquadFilter, BiquadFilterType};
use super::{AudioProcessor, ParameterMeta, PedalMeta};
use std::f32::consts::PI;

const IR_LENGTH: usize = 512;

pub struct CabSim {
    sample_rate: f32,
    enabled: bool,
    model: f32, // 0 = 4x12 UK V30, 1 = 2x12 US Twin, 2 = 1x12 Tweed, 3 = Custom
    proximity: f32,
    low_cut: f32,
    high_cut: f32,
    level_db: f32,
    // Active Impulse Response samples
    ir_kernel: Vec<f32>,
    history: Vec<f32>,
    history_idx: usize,
    // Pre/post tone shaping filters
    low_cut_filter: BiquadFilter,
    high_cut_filter: BiquadFilter,
    resonance_filter: BiquadFilter,
}

impl Default for CabSim {
    fn default() -> Self {
        Self::new()
    }
}

impl CabSim {
    pub const PARAM_MODEL: u32 = 0;
    pub const PARAM_PROXIMITY: u32 = 1;
    pub const PARAM_LOW_CUT: u32 = 2;
    pub const PARAM_HIGH_CUT: u32 = 3;
    pub const PARAM_LEVEL: u32 = 4;

    pub fn new() -> Self {
        let mut cab = Self {
            sample_rate: 48000.0,
            enabled: true,
            model: 0.0,
            proximity: 50.0,
            low_cut: 80.0,
            high_cut: 6500.0,
            level_db: 0.0,
            ir_kernel: vec![0.0; IR_LENGTH],
            history: vec![0.0; IR_LENGTH],
            history_idx: 0,
            low_cut_filter: BiquadFilter::new(),
            high_cut_filter: BiquadFilter::new(),
            resonance_filter: BiquadFilter::new(),
        };
        cab.synthesize_cabinet_ir();
        cab.update_filters();
        cab
    }

    pub fn load_custom_ir(&mut self, samples: &[f32]) {
        self.ir_kernel.fill(0.0);
        let copy_len = samples.len().min(IR_LENGTH);
        self.ir_kernel[..copy_len].copy_from_slice(&samples[..copy_len]);
        // Normalize
        let mut max_abs = 1e-6_f32;
        for &s in &self.ir_kernel {
            if s.abs() > max_abs {
                max_abs = s.abs();
            }
        }
        for s in self.ir_kernel.iter_mut() {
            *s /= max_abs;
        }
        self.model = 3.0;
    }

    fn synthesize_cabinet_ir(&mut self) {
        // High quality physical modeling synthesis of loudspeaker acoustic impulse response
        self.ir_kernel.fill(0.0);
        let sr = self.sample_rate;

        let (res_freq, damp, cone_freq, cone_damp) = match self.model as usize {
            1 => (85.0, 0.025, 3400.0, 0.04),  // 2x12 US Twin (bright, scooped)
            2 => (110.0, 0.035, 2600.0, 0.05), // 1x12 Tweed (warm, vintage)
            _ => (98.0, 0.020, 2900.0, 0.03),  // 4x12 UK V30 (chunky, punchy)
        };

        for (i, sample) in self.ir_kernel.iter_mut().enumerate() {
            let t = (i as f32) / sr;
            // Primary cabinet enclosure resonance
            let enclosure = (-t * damp * 1000.0).exp() * (2.0 * PI * res_freq * t).sin();
            // Speaker cone breakup & upper midrange resonance
            let cone = (-t * cone_damp * 1200.0).exp() * (2.0 * PI * cone_freq * t).cos() * 0.45;
            // High frequency roll-off
            let air = (-t * 8000.0).exp() * 0.3;

            *sample = (enclosure + cone + air) * (1.0 - (-t * 3000.0).exp());
        }

        // Window and normalize
        let len = self.ir_kernel.len() as f32;
        let mut sum = 0.0;
        for (i, s) in self.ir_kernel.iter_mut().enumerate() {
            let win = 0.5 * (1.0 - (2.0 * PI * (i as f32) / len).cos());
            *s *= win;
            sum += s.abs();
        }
        if sum > 1e-6 {
            let scale = 1.0 / (sum * 0.25);
            for s in self.ir_kernel.iter_mut() {
                *s *= scale;
            }
        }
    }

    fn update_filters(&mut self) {
        let sr = self.sample_rate;
        self.low_cut_filter.configure(
            BiquadFilterType::HighPass,
            sr,
            self.low_cut,
            0.707,
            0.0,
        );
        self.high_cut_filter.configure(
            BiquadFilterType::LowPass,
            sr,
            self.high_cut,
            0.707,
            0.0,
        );
        // Proximity effect bass boost (120 Hz peaking)
        let prox_gain = ((self.proximity - 50.0) / 50.0) * 5.0; // -5dB to +5dB
        self.resonance_filter.configure(
            BiquadFilterType::Peaking,
            sr,
            120.0,
            1.2,
            prox_gain,
        );
    }
}

impl AudioProcessor for CabSim {
    fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate.max(8000.0);
        self.synthesize_cabinet_ir();
        self.update_filters();
        self.reset();
    }

    fn reset(&mut self) {
        self.history.fill(0.0);
        self.history_idx = 0;
        self.low_cut_filter.reset();
        self.high_cut_filter.reset();
        self.resonance_filter.reset();
    }

    fn set_parameter(&mut self, param_id: u32, value: f32) {
        match param_id {
            Self::PARAM_MODEL => {
                self.model = value.clamp(0.0, 3.0);
                if self.model < 2.5 {
                    self.synthesize_cabinet_ir();
                }
            }
            Self::PARAM_PROXIMITY => {
                self.proximity = value.clamp(0.0, 100.0);
                self.update_filters();
            }
            Self::PARAM_LOW_CUT => {
                self.low_cut = value.clamp(20.0, 300.0);
                self.update_filters();
            }
            Self::PARAM_HIGH_CUT => {
                self.high_cut = value.clamp(2000.0, 16000.0);
                self.update_filters();
            }
            Self::PARAM_LEVEL => self.level_db = value.clamp(-18.0, 12.0),
            _ => {}
        }
    }

    fn get_parameter(&self, param_id: u32) -> f32 {
        match param_id {
            Self::PARAM_MODEL => self.model,
            Self::PARAM_PROXIMITY => self.proximity,
            Self::PARAM_LOW_CUT => self.low_cut,
            Self::PARAM_HIGH_CUT => self.high_cut,
            Self::PARAM_LEVEL => self.level_db,
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

        let gain = 10.0_f32.powf(self.level_db / 20.0);
        let ir_len = self.ir_kernel.len();

        for sample in buffer.iter_mut() {
            let input = *sample;

            // Save input into circular history buffer
            self.history[self.history_idx] = input;

            // Direct zero-latency FIR convolution with speaker IR
            let mut conv = 0.0;
            let mut read_idx = self.history_idx;
            for i in 0..ir_len {
                conv += self.history[read_idx] * self.ir_kernel[i];
                if read_idx == 0 {
                    read_idx = ir_len - 1;
                } else {
                    read_idx -= 1;
                }
            }

            self.history_idx = (self.history_idx + 1) % ir_len;

            // Apply acoustic post-filters (Low Cut, High Cut, Mic proximity)
            let filtered = self.low_cut_filter.process_sample(conv);
            let filtered2 = self.high_cut_filter.process_sample(filtered);
            let shaped = self.resonance_filter.process_sample(filtered2);

            *sample = shaped * gain;
        }
    }

    fn meta(&self) -> PedalMeta {
        PedalMeta {
            id: "cab_sim".to_string(),
            name: "Cabinet Simulator (IR)".to_string(),
            category: "Cabinet".to_string(),
            enabled: self.enabled,
            parameters: vec![
                ParameterMeta {
                    id: Self::PARAM_MODEL,
                    name: "Cabinet Model".to_string(),
                    min: 0.0,
                    max: 3.0,
                    default_value: 0.0,
                    unit: "type".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_PROXIMITY,
                    name: "Proximity".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 50.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_LOW_CUT,
                    name: "Low Cut".to_string(),
                    min: 20.0,
                    max: 300.0,
                    default_value: 80.0,
                    unit: "Hz".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_HIGH_CUT,
                    name: "High Cut".to_string(),
                    min: 2000.0,
                    max: 16000.0,
                    default_value: 6500.0,
                    unit: "Hz".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_LEVEL,
                    name: "Level".to_string(),
                    min: -18.0,
                    max: 12.0,
                    default_value: 0.0,
                    unit: "dB".to_string(),
                },
            ],
        }
    }
}
