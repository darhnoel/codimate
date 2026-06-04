#[derive(Clone, Copy)]
pub struct SymSpellTiming {
    pub intro: f32,
    pub index_word: f32,
    pub index_built_hold: f32,
    pub generate: f32,
    pub lookup_miss: f32,
    pub lookup_hit: f32,
    pub verify: f32,
    pub rank: f32,
    pub answer: f32,
    pub final_hold: f32,
}

impl Default for SymSpellTiming {
    fn default() -> Self {
        Self {
            intro: 2.0,
            index_word: 1.6,
            index_built_hold: 1.0,
            generate: 1.5,
            lookup_miss: 0.5,
            lookup_hit: 1.6,
            verify: 0.8,
            rank: 1.2,
            answer: 1.5,
            final_hold: 2.0,
        }
    }
}
