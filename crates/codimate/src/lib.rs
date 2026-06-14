//! codimate — beginner facade crate.
//!
//! One dependency, one import surface:
//!
//! ```
//! use codimate::*;
//! ```

pub use codimate_animation::*;
pub use codimate_core::*;
pub use codimate_export::*;
pub use codimate_layout::*;

/// Beginner-friendly render helper with sensible defaults.
///
/// Uses 30 fps and native viewport output.
pub fn render(
    playable: &impl Playable,
    viewport: Viewport,
    output: impl AsRef<std::path::Path>,
) -> Result<(), ExportError> {
    let cfg = ExportConfig::new(30.0, viewport);
    export_mp4(playable, &cfg, output)
}

/// Render helper that accepts explicit export settings.
pub fn render_with(
    playable: &impl Playable,
    config: ExportConfig,
    output: impl AsRef<std::path::Path>,
) -> Result<(), ExportError> {
    export_mp4(playable, &config, output)
}
