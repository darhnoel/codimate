#[derive(Clone, Copy)]
pub struct SwapTiming {
    pub move_duration: f32,
    pub pause: f32,
}

impl Default for SwapTiming {
    fn default() -> Self {
        Self {
            move_duration: 0.85,
            pause: 0.25,
        }
    }
}
