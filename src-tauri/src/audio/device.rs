use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
    pub max_channels: u16,
    pub sample_rates: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioHostInfo {
    pub name: String,
    pub host_id: String,
    pub input_devices: Vec<AudioDeviceInfo>,
    pub output_devices: Vec<AudioDeviceInfo>,
}

pub struct DeviceManager {
    host: Host,
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    pub fn enumerate_hosts() -> Vec<String> {
        cpal::available_hosts()
            .into_iter()
            .map(|id| format!("{:?}", id))
            .collect()
    }

    pub fn get_host_info(&self) -> AudioHostInfo {
        let host_name = format!("{:?}", self.host.id());

        let default_in_name = self
            .host
            .default_input_device()
            .and_then(|d| d.name().ok());
        let default_out_name = self
            .host
            .default_output_device()
            .and_then(|d| d.name().ok());

        let mut input_devices = Vec::new();
        if let Ok(devs) = self.host.input_devices() {
            for dev in devs {
                if let Ok(name) = dev.name() {
                    let is_default = default_in_name.as_ref() == Some(&name);
                    let mut sample_rates = Vec::new();
                    let mut max_channels = 1;
                    if let Ok(configs) = dev.supported_input_configs() {
                        for cfg in configs {
                            max_channels = max_channels.max(cfg.channels());
                            let min_rate = cfg.min_sample_rate().0;
                            let max_rate = cfg.max_sample_rate().0;
                            for &std_rate in &[44100, 48000, 88200, 96000] {
                                if std_rate >= min_rate && std_rate <= max_rate && !sample_rates.contains(&std_rate) {
                                    sample_rates.push(std_rate);
                                }
                            }
                        }
                    }
                    if sample_rates.is_empty() {
                        sample_rates.push(48000);
                    }
                    input_devices.push(AudioDeviceInfo {
                        name,
                        is_default,
                        max_channels,
                        sample_rates,
                    });
                }
            }
        }

        let mut output_devices = Vec::new();
        if let Ok(devs) = self.host.output_devices() {
            for dev in devs {
                if let Ok(name) = dev.name() {
                    let is_default = default_out_name.as_ref() == Some(&name);
                    let mut sample_rates = Vec::new();
                    let mut max_channels = 2;
                    if let Ok(configs) = dev.supported_output_configs() {
                        for cfg in configs {
                            max_channels = max_channels.max(cfg.channels());
                            let min_rate = cfg.min_sample_rate().0;
                            let max_rate = cfg.max_sample_rate().0;
                            for &std_rate in &[44100, 48000, 88200, 96000] {
                                if std_rate >= min_rate && std_rate <= max_rate && !sample_rates.contains(&std_rate) {
                                    sample_rates.push(std_rate);
                                }
                            }
                        }
                    }
                    if sample_rates.is_empty() {
                        sample_rates.push(48000);
                    }
                    output_devices.push(AudioDeviceInfo {
                        name,
                        is_default,
                        max_channels,
                        sample_rates,
                    });
                }
            }
        }

        AudioHostInfo {
            name: host_name.clone(),
            host_id: host_name,
            input_devices,
            output_devices,
        }
    }

    pub fn find_input_device(&self, name: Option<&str>) -> Option<Device> {
        if let Some(target) = name {
            if let Ok(mut devs) = self.host.input_devices() {
                if let Some(found) = devs.find(|d| d.name().map(|n| n == target).unwrap_or(false)) {
                    return Some(found);
                }
            }
        }
        self.host.default_input_device()
    }

    pub fn find_output_device(&self, name: Option<&str>) -> Option<Device> {
        if let Some(target) = name {
            if let Ok(mut devs) = self.host.output_devices() {
                if let Some(found) = devs.find(|d| d.name().map(|n| n == target).unwrap_or(false)) {
                    return Some(found);
                }
            }
        }
        self.host.default_output_device()
    }
}
