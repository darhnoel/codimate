#[derive(Clone, Copy)]
pub struct ConnectionPulse {
    pub(crate) left_x: f32,
    pub(crate) right_x: f32,
    pub(crate) box_y: f32,
    pub(crate) box_w: f32,
    pub(crate) box_h: f32,
}

impl ConnectionPulse {
    pub fn new() -> Self {
        Self {
            left_x: 50.0,
            right_x: 400.0,
            box_y: 150.0,
            box_w: 100.0,
            box_h: 60.0,
        }
    }
}

impl Default for ConnectionPulse {
    fn default() -> Self {
        Self::new()
    }
}
