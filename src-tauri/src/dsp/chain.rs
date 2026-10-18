use super::cab_ir::CabSim;
use super::delay::Delay;
use super::drive::Overdrive;
use super::gate::NoiseGate;
use super::reverb::Reverb;
use super::{AudioProcessor, PedalMeta};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedalState {
    pub id: String,
    pub name: String,
    pub category: String,
    pub enabled: bool,
    pub params: Vec<(u32, f32)>,
}

pub struct PedalChain {
    sample_rate: f32,
    processors: Vec<Box<dyn AudioProcessor>>,
    master_gain: f32,
    limiter_ceiling: f32,
}

impl Default for PedalChain {
    fn default() -> Self {
        Self::new()
    }
}

impl PedalChain {
    pub fn new() -> Self {
        let mut chain = Self {
            sample_rate: 48000.0,
            processors: Vec::new(),
            master_gain: 1.0,
            limiter_ceiling: 0.98,
        };

        // Initialize default standard pedalboard chain:
        // Gate -> Drive -> Delay -> Reverb -> Cab Sim
        chain.processors.push(Box::new(NoiseGate::new()));
        chain.processors.push(Box::new(Overdrive::new()));
        chain.processors.push(Box::new(Delay::new()));
        chain.processors.push(Box::new(Reverb::new()));
        chain.processors.push(Box::new(CabSim::new()));

        chain.prepare(48000.0, 512);
        chain
    }

    pub fn set_master_gain(&mut self, gain_db: f32) {
        self.master_gain = 10.0_f32.powf(gain_db / 20.0);
    }

    pub fn get_pedal_list(&self) -> Vec<PedalMeta> {
        self.processors.iter().map(|p| p.meta()).collect()
    }

    pub fn set_pedal_enabled(&mut self, pedal_id: &str, enabled: bool) -> bool {
        for proc in self.processors.iter_mut() {
            if proc.meta().id == pedal_id {
                proc.set_enabled(enabled);
                return true;
            }
        }
        false
    }

    pub fn set_pedal_param(&mut self, pedal_id: &str, param_id: u32, value: f32) -> bool {
        for proc in self.processors.iter_mut() {
            if proc.meta().id == pedal_id {
                proc.set_parameter(param_id, value);
                return true;
            }
        }
        false
    }

    pub fn reorder_pedals(&mut self, new_order: &[String]) {
        let mut new_procs: Vec<Box<dyn AudioProcessor>> = Vec::new();
        for id in new_order {
            if let Some(pos) = self.processors.iter().position(|p| &p.meta().id == id) {
                new_procs.push(self.processors.remove(pos));
            }
        }
        // Append any remaining processors
        new_procs.append(&mut self.processors);
        self.processors = new_procs;
    }

    pub fn load_cab_ir_samples(&mut self, _samples: &[f32]) {
        for proc in self.processors.iter_mut() {
            if proc.meta().id == "cab_sim" {
                // downcast or call cab sim helper
                proc.set_parameter(CabSim::PARAM_MODEL, 3.0);
            }
        }
    }
}

impl AudioProcessor for PedalChain {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.sample_rate = sample_rate.max(8000.0);
        for proc in self.processors.iter_mut() {
            proc.prepare(self.sample_rate, max_block_size);
        }
    }

    fn reset(&mut self) {
        for proc in self.processors.iter_mut() {
            proc.reset();
        }
    }

    fn set_parameter(&mut self, _param_id: u32, _value: f32) {}

    fn get_parameter(&self, _param_id: u32) -> f32 {
        0.0
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn set_enabled(&mut self, _enabled: bool) {}

    fn process(&mut self, buffer: &mut [f32]) {
        // Cascade process each pedal in the signal chain
        for proc in self.processors.iter_mut() {
            proc.process(buffer);
        }

        // Apply Master output gain & soft brickwall limiter
        let gain = self.master_gain;
        let ceiling = self.limiter_ceiling;

        for sample in buffer.iter_mut() {
            let amplified = *sample * gain;
            // Soft-knee brickwall limiter
            let limited = if amplified.abs() > ceiling {
                let sign = amplified.signum();
                sign * (ceiling + (amplified.abs() - ceiling).tanh() * (1.0 - ceiling))
            } else {
                amplified
            };
            *sample = limited.clamp(-1.0, 1.0);
        }
    }

    fn meta(&self) -> PedalMeta {
        PedalMeta {
            id: "pedal_chain".to_string(),
            name: "Pedalboard Signal Chain".to_string(),
            category: "Master".to_string(),
            enabled: true,
            parameters: Vec::new(),
        }
    }
}
