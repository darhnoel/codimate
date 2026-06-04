#[derive(Clone, Copy)]
pub struct ThreeSumTiming {
    pub intro: f32,
    pub sort: f32,
    pub fix_anchor: f32,
    pub set_pointers: f32,
    pub compare: f32,
    pub move_pointer: f32,
    pub found: f32,
    pub skip_duplicate: f32,
    pub final_hold: f32,
}

impl Default for ThreeSumTiming {
    fn default() -> Self {
        Self {
            intro: 1.2,
            sort: 1.2,
            fix_anchor: 0.55,
            set_pointers: 0.55,
            compare: 0.72,
            move_pointer: 0.45,
            found: 1.0,
            skip_duplicate: 0.5,
            final_hold: 1.4,
        }
    }
}
