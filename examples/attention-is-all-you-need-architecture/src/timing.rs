#[derive(Clone, Copy)]
pub struct TransformerArchitectureTiming {
    pub short: f32,
    pub normal: f32,
    pub final_reveal: f32,
}

impl Default for TransformerArchitectureTiming {
    fn default() -> Self {
        Self {
            short: 2.4,
            normal: 3.2,
            final_reveal: 4.5,
        }
    }
}
