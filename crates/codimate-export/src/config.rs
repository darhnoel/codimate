use codimate_layout::Viewport;

/// Export sampling configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExportConfig {
    pub fps: f32,
    pub viewport: Viewport,
    /// Optional encoded output size. Frames are rendered at `viewport`, then
    /// scaled and padded by the encoder so the scene keeps its aspect ratio.
    pub output_viewport: Option<Viewport>,
    /// H.264 CRF (0–51). Lower = better quality, larger file.
    /// Default 23 is a good balance; use 10–15 for demo-quality exports.
    pub crf: u32,
    /// Render at this many times the viewport resolution before encoding.
    /// Use with `output_viewport` to produce crisp high-DPI output without
    /// pixel upscaling artifacts. Default 1.0 (native viewport resolution).
    pub pixel_scale: f32,
}

impl ExportConfig {
    #[must_use]
    pub fn new(fps: f32, viewport: Viewport) -> Self {
        Self {
            fps,
            viewport,
            output_viewport: None,
            crf: 23,
            pixel_scale: 1.0,
        }
    }

    #[must_use]
    pub fn output_viewport(mut self, viewport: Viewport) -> Self {
        self.output_viewport = Some(viewport);
        self
    }

    #[must_use]
    pub fn crf(mut self, crf: u32) -> Self {
        self.crf = crf;
        self
    }

    #[must_use]
    pub fn pixel_scale(mut self, scale: f32) -> Self {
        self.pixel_scale = scale;
        self
    }
}
