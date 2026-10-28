//! codimate-export — frame export boundary.
//!
//! PNG writing, raw RGBA streaming, and ffmpeg subprocess piping.

mod config;
mod error;
mod frames;
mod mp4;
mod still;

pub use config::*;
pub use error::*;
pub use frames::*;
pub use mp4::*;
pub use still::*;

// ---------------------------------------------------------------------------
// PNG (unchanged, from the previous slice)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use codimate_layout::Viewport;

    #[test]
    fn scale_pad_filter_preserves_aspect_ratio_for_1080p() {
        assert_eq!(
            scale_pad_filter(Viewport::new(1000.0, 640.0), Viewport::new(1920.0, 1080.0)),
            "scale=1688:1080:flags=lanczos,pad=1920:1080:116:0:color=black"
        );
    }
}
