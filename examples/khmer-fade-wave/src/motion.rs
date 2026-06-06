#[derive(Clone, Copy)]
pub struct KhmerFadeWaveMotion {
    pub contour_fraction: f32,
    pub wave_lift: f32,
}

impl Default for KhmerFadeWaveMotion {
    fn default() -> Self {
        Self {
            contour_fraction: 0.68,
            wave_lift: 18.0,
        }
    }
}

pub fn khmer_fade_wave_motion() -> KhmerFadeWaveMotion {
    KhmerFadeWaveMotion::default()
}

pub fn bounce(local_t: f32) -> f32 {
    if local_t < 0.5 {
        local_t * 2.0
    } else {
        2.0 - local_t * 2.0
    }
}
