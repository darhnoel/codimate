#[derive(Clone, Copy)]
pub struct NeuralNetTiming {
    pub show_inputs: f32,
    pub fire_group: f32,
    pub final_hold: f32,
}

impl Default for NeuralNetTiming {
    fn default() -> Self {
        Self {
            show_inputs: 0.8,
            fire_group: 0.42,
            final_hold: 1.0,
        }
    }
}
