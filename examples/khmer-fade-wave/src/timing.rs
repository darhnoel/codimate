#[derive(Clone, Copy)]
pub struct KhmerFadeWaveTiming {
    pub reveal: f32,
    pub wave: f32,
    pub wave_stagger: f32,
}

impl Default for KhmerFadeWaveTiming {
    fn default() -> Self {
        Self {
            reveal: 2.0,
            wave: 1.2,
            wave_stagger: 0.06,
        }
    }
}

impl KhmerFadeWaveTiming {
    pub fn wave_total(self, unit_count: usize) -> f32 {
        self.wave + (unit_count as f32 - 1.0).max(0.0) * self.wave_stagger
    }
}
