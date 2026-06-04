#[derive(Clone, Copy)]
pub struct HanoiTiming {
    pub intro: f32,
    pub move_disk: f32,
    pub final_hold: f32,
}

impl Default for HanoiTiming {
    fn default() -> Self {
        Self {
            intro: 1.2,
            move_disk: 0.68,
            final_hold: 1.4,
        }
    }
}
