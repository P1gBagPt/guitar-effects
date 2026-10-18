pub mod biquad;
pub mod cab_ir;
pub mod chain;
pub mod delay;
pub mod drive;
pub mod gate;
pub mod reverb;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterMeta {
    pub id: u32,
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub default_value: f32,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedalMeta {
    pub id: String,
    pub name: String,
    pub category: String,
    pub enabled: bool,
    pub parameters: Vec<ParameterMeta>,
}

pub trait AudioProcessor: Send {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize);
    fn process(&mut self, buffer: &mut [f32]);
    fn reset(&mut self);
    fn set_parameter(&mut self, param_id: u32, value: f32);
    fn get_parameter(&self, param_id: u32) -> f32;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn meta(&self) -> PedalMeta;
}
