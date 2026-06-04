#[derive(Clone, Copy)]
pub struct Demo {
    pub(crate) circle_start_x: f32,
    pub(crate) circle_end_x: f32,
    pub(crate) circle_y: f32,
    pub(crate) circle_start_radius: f32,
    pub(crate) circle_end_radius: f32,
    pub(crate) rect_x: f32,
    pub(crate) rect_start_y: f32,
    pub(crate) rect_end_y: f32,
    pub(crate) rect_w: f32,
    pub(crate) rect_h: f32,
}

impl Demo {
    pub fn new() -> Self {
        Self {
            circle_start_x: 40.0,
            circle_end_x: 240.0,
            circle_y: 120.0,
            circle_start_radius: 16.0,
            circle_end_radius: 48.0,
            rect_x: 80.0,
            rect_start_y: 180.0,
            rect_end_y: 260.0,
            rect_w: 220.0,
            rect_h: 80.0,
        }
    }
}

impl Default for Demo {
    fn default() -> Self {
        Self::new()
    }
}
