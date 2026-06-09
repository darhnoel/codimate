#[derive(Clone, Copy)]
pub struct LogoAnimationTiming {
    pub(crate) frame: f32,
    pub(crate) source: f32,
    pub(crate) glyph: f32,
    pub(crate) keys: f32,
    pub(crate) settle: f32,
}

impl Default for LogoAnimationTiming {
    fn default() -> Self {
        Self {
            frame: 0.75,
            source: 0.45,
            glyph: 0.5,
            keys: 0.5,
            settle: 0.95,
        }
    }
}
