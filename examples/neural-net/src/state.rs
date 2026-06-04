pub const INPUT_COUNT: usize = 3;
pub const HIDDEN_COUNT: usize = 4;
pub const OUTPUT_COUNT: usize = 2;

#[derive(Clone, Copy)]
pub struct NeuralNet {
    pub(crate) input_count: usize,
    pub(crate) hidden_count: usize,
    pub(crate) output_count: usize,
}

impl NeuralNet {
    pub fn new() -> Self {
        Self {
            input_count: INPUT_COUNT,
            hidden_count: HIDDEN_COUNT,
            output_count: OUTPUT_COUNT,
        }
    }
}

impl Default for NeuralNet {
    fn default() -> Self {
        Self::new()
    }
}
