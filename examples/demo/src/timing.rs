#[derive(Clone, Copy)]
pub struct DemoTiming {
    pub grow_circle: f32,
    pub move_rect: f32,
    pub morph_path: f32,
}

impl Default for DemoTiming {
    fn default() -> Self {
        Self {
            grow_circle: 1.0,
            move_rect: 1.0,
            morph_path: 1.0,
        }
    }
}
