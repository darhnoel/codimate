#[derive(Clone, Copy)]
pub struct WordAppearTiming {
    pub per_word: f32,
    pub stagger_offset: f32,
}

impl Default for WordAppearTiming {
    fn default() -> Self {
        Self {
            per_word: 0.6,
            stagger_offset: 0.15,
        }
    }
}
