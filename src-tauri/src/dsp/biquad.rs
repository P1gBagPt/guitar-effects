use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BiquadFilterType {
    LowPass,
    HighPass,
    BandPass,
    Peaking,
    LowShelf,
    HighShelf,
}

#[derive(Debug, Clone)]
pub struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl BiquadFilter {
    pub fn new() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    pub fn configure(
        &mut self,
        filter_type: BiquadFilterType,
        sample_rate: f32,
        frequency: f32,
        q: f32,
        gain_db: f32,
    ) {
        let sample_rate = sample_rate.max(8000.0);
        let freq = frequency.clamp(10.0, sample_rate * 0.49);
        let q = q.max(0.01);
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        let a = 10.0_f32.powf(gain_db / 40.0);

        let (b0, b1, b2, a0, a1, a2) = match filter_type {
            BiquadFilterType::LowPass => {
                let b0 = (1.0 - cos_omega) / 2.0;
                let b1 = 1.0 - cos_omega;
                let b2 = (1.0 - cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadFilterType::HighPass => {
                let b0 = (1.0 + cos_omega) / 2.0;
                let b1 = -(1.0 + cos_omega);
                let b2 = (1.0 + cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadFilterType::BandPass => {
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadFilterType::Peaking => {
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha / a;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadFilterType::LowShelf => {
                let a_plus_1 = a + 1.0;
                let a_minus_1 = a - 1.0;
                let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
                let b0 = a * (a_plus_1 - a_minus_1 * cos_omega + two_sqrt_a_alpha);
                let b1 = 2.0 * a * (a_minus_1 - a_plus_1 * cos_omega);
                let b2 = a * (a_plus_1 - a_minus_1 * cos_omega - two_sqrt_a_alpha);
                let a0 = a_plus_1 + a_minus_1 * cos_omega + two_sqrt_a_alpha;
                let a1 = -2.0 * (a_minus_1 + a_plus_1 * cos_omega);
                let a2 = a_plus_1 + a_minus_1 * cos_omega - two_sqrt_a_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadFilterType::HighShelf => {
                let a_plus_1 = a + 1.0;
                let a_minus_1 = a - 1.0;
                let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
                let b0 = a * (a_plus_1 + a_minus_1 * cos_omega + two_sqrt_a_alpha);
                let b1 = -2.0 * a * (a_minus_1 + a_plus_1 * cos_omega);
                let b2 = a * (a_plus_1 + a_minus_1 * cos_omega - two_sqrt_a_alpha);
                let a0 = a_plus_1 - a_minus_1 * cos_omega + two_sqrt_a_alpha;
                let a1 = 2.0 * (a_minus_1 - a_plus_1 * cos_omega);
                let a2 = a_plus_1 - a_minus_1 * cos_omega - two_sqrt_a_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        let inv_a0 = 1.0 / a0;
        self.b0 = b0 * inv_a0;
        self.b1 = b1 * inv_a0;
        self.b2 = b2 * inv_a0;
        self.a1 = a1 * inv_a0;
        self.a2 = a2 * inv_a0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        // Prevent denormals
        if output.abs() < 1e-15 {
            0.0
        } else {
            output
        }
    }
}
