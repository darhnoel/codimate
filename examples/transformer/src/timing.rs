#[derive(Clone, Copy)]
pub struct TransformerTiming {
    pub encoder: f32,
    pub decoder: f32,
    pub cross: f32,
}

impl Default for TransformerTiming {
    fn default() -> Self {
        Self {
            encoder: 3.0,
            decoder: 3.0,
            cross: 1.5,
        }
    }
}
