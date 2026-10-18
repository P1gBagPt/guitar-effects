use super::{AudioProcessor, ParameterMeta, PedalMeta};

struct CombFilter {
    buffer: Vec<f32>,
    write_pos: usize,
    feedback: f32,
    damp: f32,
    filter_store: f32,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            write_pos: 0,
            feedback: 0.5,
            damp: 0.5,
            filter_store: 0.0,
        }
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.write_pos];
        self.filter_store = output * (1.0 - self.damp) + self.filter_store * self.damp;
        self.buffer[self.write_pos] = input + self.filter_store * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        output
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.filter_store = 0.0;
    }
}

struct AllPassFilter {
    buffer: Vec<f32>,
    write_pos: usize,
    feedback: f32,
}

impl AllPassFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            write_pos: 0,
            feedback: 0.5,
        }
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let buf_out = self.buffer[self.write_pos];
        let output = -input + buf_out;
        self.buffer[self.write_pos] = input + buf_out * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        output
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

const COMB_TUNING: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNING: [usize; 4] = [556, 441, 341, 225];

pub struct Reverb {
    sample_rate: f32,
    enabled: bool,
    room_size: f32,
    damp: f32,
    mix: f32,
    decay_time: f32,
    combs: Vec<CombFilter>,
    allpasses: Vec<AllPassFilter>,
}

impl Default for Reverb {
    fn default() -> Self {
        Self::new()
    }
}

impl Reverb {
    pub const PARAM_ROOM_SIZE: u32 = 0;
    pub const PARAM_DAMP: u32 = 1;
    pub const PARAM_MIX: u32 = 2;
    pub const PARAM_DECAY: u32 = 3;

    pub fn new() -> Self {
        let mut reverb = Self {
            sample_rate: 48000.0,
            enabled: true,
            room_size: 65.0,
            damp: 35.0,
            mix: 30.0,
            decay_time: 50.0,
            combs: Vec::new(),
            allpasses: Vec::new(),
        };
        reverb.rebuild_filters(48000.0);
        reverb
    }

    fn rebuild_filters(&mut self, sample_rate: f32) {
        let scale = sample_rate / 44100.0;
        self.combs.clear();
        for &tuning in &COMB_TUNING {
            let size = ((tuning as f32) * scale).round() as usize;
            self.combs.push(CombFilter::new(size.max(4)));
        }
        self.allpasses.clear();
        for &tuning in &ALLPASS_TUNING {
            let size = ((tuning as f32) * scale).round() as usize;
            self.allpasses.push(AllPassFilter::new(size.max(4)));
        }
        self.update_parameters();
    }

    fn update_parameters(&mut self) {
        let fb = 0.7 + (self.room_size / 100.0) * 0.28;
        let damp = (self.damp / 100.0) * 0.5;

        for comb in self.combs.iter_mut() {
            comb.feedback = fb;
            comb.damp = damp;
        }
        for allpass in self.allpasses.iter_mut() {
            allpass.feedback = 0.5;
        }
    }
}

impl AudioProcessor for Reverb {
    fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate.max(8000.0);
        self.rebuild_filters(self.sample_rate);
    }

    fn reset(&mut self) {
        for comb in self.combs.iter_mut() {
            comb.reset();
        }
        for allpass in self.allpasses.iter_mut() {
            allpass.reset();
        }
    }

    fn set_parameter(&mut self, param_id: u32, value: f32) {
        match param_id {
            Self::PARAM_ROOM_SIZE => {
                self.room_size = value.clamp(0.0, 100.0);
                self.update_parameters();
            }
            Self::PARAM_DAMP => {
                self.damp = value.clamp(0.0, 100.0);
                self.update_parameters();
            }
            Self::PARAM_MIX => self.mix = value.clamp(0.0, 100.0),
            Self::PARAM_DECAY => {
                self.decay_time = value.clamp(0.0, 100.0);
                self.update_parameters();
            }
            _ => {}
        }
    }

    fn get_parameter(&self, param_id: u32) -> f32 {
        match param_id {
            Self::PARAM_ROOM_SIZE => self.room_size,
            Self::PARAM_DAMP => self.damp,
            Self::PARAM_MIX => self.mix,
            Self::PARAM_DECAY => self.decay_time,
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
        let dry_mix = 1.0 - wet_mix * 0.4;

        for sample in buffer.iter_mut() {
            let input = *sample;

            // Parallel comb filters
            let mut comb_sum = 0.0;
            for comb in self.combs.iter_mut() {
                comb_sum += comb.process(input);
            }
            comb_sum *= 0.125; // Normalization for 8 combs

            // Serial all-pass diffusers
            let mut diffused = comb_sum;
            for allpass in self.allpasses.iter_mut() {
                diffused = allpass.process(diffused);
            }

            *sample = input * dry_mix + diffused * wet_mix;
        }
    }

    fn meta(&self) -> PedalMeta {
        PedalMeta {
            id: "hall_reverb".to_string(),
            name: "Ambient Reverb".to_string(),
            category: "Reverb".to_string(),
            enabled: self.enabled,
            parameters: vec![
                ParameterMeta {
                    id: Self::PARAM_ROOM_SIZE,
                    name: "Room Size".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 65.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_DAMP,
                    name: "Damp".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 35.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_MIX,
                    name: "Mix".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 30.0,
                    unit: "%".to_string(),
                },
                ParameterMeta {
                    id: Self::PARAM_DECAY,
                    name: "Decay".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default_value: 50.0,
                    unit: "%".to_string(),
                },
            ],
        }
    }
}
