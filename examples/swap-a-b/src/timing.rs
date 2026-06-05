#[derive(Clone, Copy)]
pub struct SwapABTiming {
    pub(crate) intro: f32,
    pub(crate) swap: f32,
    pub(crate) done: f32,
}

impl Default for SwapABTiming {
    fn default() -> Self {
        Self {
            intro: 1.5,
            swap: 1.8,
            done: 1.4,
        }
    }
}
