use super::{AudioProcessor, ParameterMeta, PedalMeta};

pub struct NoiseGate {
    sample_rate: f32,
    enabled: bool,
    threshold_db: f32,
    attack_ms: f32,
    hold_ms: f32,
    release_ms: f32,
    envelope: f32,
    gain: f32,
    hold_counter: usize,
}

impl Default for NoiseGate {
    fn default() -> Self {
        Self::new()
    }
}

impl NoiseGate {
    pub const PARAM_THRESHOLD: u32 = 0;
    pub const PARAM_ATTACK: u32 = 1;
    pub const PARAM_HOLD: u32 = 2;
    pub const PARAM_RELEASE: u32 = 3;

    pub fn new() -> Self {
        Self {
            sample_rate: 48000.0,
            enabled: true,
            threshold_db: -50.0,
            attack_ms: 1.5,
            hold_ms: 10.0,
            release_ms: 60.0,
            envelope: 0.0,
            gain: 1.0,
            hold_counter: 0,
        }
    }
}

impl AudioProcessor for NoiseGate {
    fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate.max(8000.0);
        self.reset();
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
        self.gain = 1.0;
        self.hold_counter = 0;
    }

    fn set_parameter(&mut self, param_id: u32, value: f32) {
        match param_id {
            Self::PARAM_THRESHOLD => self.threshold_db = value.clamp(-80.0, 0.0),
            Self::PARAM_ATTACK => self.attack_ms = value.clamp(0.1, 20.0),
            Self::PARAM_HOLD => self.hold_ms = value.clamp(0.0, 100.0),
            Self::PARAM_RELEASE => self.release_ms = value.clamp(5.0, 500.0),
            _ => {}
        }
    }

    fn get_parameter(&self, param_id: u32) -> f32 {
        match param_id {
            Self::PARAM_THRESHOLD => self.threshold_db,
            Self::PARAM_ATTACK => self.attack_ms,
            Self::PARAM_HOLD => self.hold_ms,
            Self::PARAM_RELEASE => self.release_ms,
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

        let thresh_lin = 10.0_f32.powf(self.threshold_db / 20.0);
        let attack_coeff = (-1.0 / (self.attack_ms * 0.001 * self.sample_rate)).exp();
        let release_coeff = (-1.0 / (self.release_ms * 0.001 * self.sample_rate)).exp();
        let hold_samples = (self.hold_ms * 0.001 * self.sample_rate) as usize;

        for sample in buffer.iter_mut() {
            let abs_in = sample.abs();
            // Envelope follower
            if abs_in > self.envelope {
                self.envelope = attack_coeff * self.envelope + (1.0 - attack_coeff) * abs_in;
            } else {
                self.envelope = release_coeff * self.envelope + (1.0 - release_coeff) * abs_in;
            }

            // Gate state evaluation
            let target_gain = if self.envelope >= thresh_lin {
                self.hold_counter = hold_samples;
                1.0
            } else if self.hold_counter > 0 {
                self.hold_counter -= 1;
                1.0
            } else {
                0.0
            };

            // Smooth gain transition
            let coeff = if target_gain > self.gain {
                attack_coeff
            } else {
                release_coeff
            };
            self.gain = coeff * self.gain + (1.0 - coeff) * target_gain;

            *sample *= self.gain;
        }
    }

    fn meta(&self) -> PedalMeta {
        PedalMeta {
            id: "noise_gate".to_string(),
            name: "Noise Gate".to_string(),
            category: "Dynamics".to_string(),
            enabled: self.enabled,
            parameters: vec![
                ParameterMeta {
                    id: Self::PARAM_THRESHOLD,
                    name: "Threshold".to_string(),
                    min: -80.0,
                    max: 0.0,
                    default_value: -50.0,
                    unit: "dB".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_ATTACK,
                    name: "Attack".to_string(),
                    min: 0.1,
                    max: 20.0,
                    default_value: 1.5,
                    unit: "ms".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_HOLD,
                    name: "Hold".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 10.0,
                    unit: "ms".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_RELEASE,
                    name: "Release".to_string(),
                    min: 5.0,
                    max: 500.0,
                    default_value: 60.0,
                    unit: "ms".to_string(),
                },
            ],
        }
    }
}
