#[derive(Clone, Copy)]
pub struct TransformerArchitecture {
    pub(crate) input: &'static [&'static str],
    pub(crate) output: &'static str,
}

impl TransformerArchitecture {
    pub fn new() -> Self {
        Self {
            input: &["I", "like", "cats"],
            output: "私は猫が好きです",
        }
    }
}

impl Default for TransformerArchitecture {
    fn default() -> Self {
        Self::new()
    }
}
