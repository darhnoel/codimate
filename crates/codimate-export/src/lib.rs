//! codimate-export — frame export boundary.
//!
//! PNG writing, raw RGBA streaming, and ffmpeg subprocess piping.

use codimate_animation::Playable;
use codimate_layout::{layout_scene, Viewport};
use codimate_render::{rasterize, render_frame, RenderFrame};
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

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
}

impl ExportConfig {
    pub fn new(fps: f32, viewport: Viewport) -> Self {
        Self {
            fps,
            viewport,
            output_viewport: None,
            crf: 23,
        }
    }

    pub fn output_viewport(mut self, viewport: Viewport) -> Self {
        self.output_viewport = Some(viewport);
        self
    }

    pub fn crf(mut self, crf: u32) -> Self {
        self.crf = crf;
        self
    }
}

/// A render-ready frame with its sequence index.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportFrame {
    pub index: usize,
    pub frame: RenderFrame,
}

/// Errors from the export pipeline.
#[derive(Debug)]
pub enum ExportError {
    Io(io::Error),
    EncoderNotFound,
    EncoderFailed(std::process::ExitStatus),
}

impl From<io::Error> for ExportError {
    fn from(e: io::Error) -> Self {
        ExportError::Io(e)
    }
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Io(e) => write!(f, "I/O error: {}", e),
            ExportError::EncoderNotFound => {
                write!(f, "video encoder (ffmpeg) not found on PATH")
            }
            ExportError::EncoderFailed(status) => {
                write!(f, "video encoder failed with exit status: {}", status)
            }
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExportError::Io(e) => Some(e),
            ExportError::EncoderNotFound | ExportError::EncoderFailed(_) => None,
        }
    }
}

/// Lazy frame iterator over a `Playable` at a given `ExportConfig`.
///
/// Each call to `next` resolves exactly one frame. No frames are buffered.
/// The final frame always lands at `duration` seconds (the last sample).
///
/// ```
/// use codimate_animation::animation;
/// use codimate_core::{circle, scene};
/// use codimate_export::{playable_frames, ExportConfig};
/// use codimate_layout::Viewport;
///
/// let a = animation("demo", 1.0, scene().node(circle().radius(10.0)));
/// let frames: Vec<_> = playable_frames(&a, ExportConfig::new(2.0, Viewport::new(800.0, 600.0))).collect();
/// assert_eq!(frames.len(), 3);
/// ```
pub fn playable_frames<'a>(
    playable: &'a dyn Playable,
    config: ExportConfig,
) -> PlayableFrameIter<'a> {
    PlayableFrameIter {
        playable,
        config,
        elapsed: 0.0,
    }
}

/// Frame-by-frame iterator — see [`playable_frames`].
#[derive(Clone)]
pub struct PlayableFrameIter<'a> {
    playable: &'a dyn Playable,
    config: ExportConfig,
    elapsed: f32,
}

impl<'a> Iterator for PlayableFrameIter<'a> {
    type Item = RenderFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let duration = self.playable.duration();

        if self.elapsed > duration {
            return None;
        }

        let frame = sample_render_frame(self.playable, &self.config, self.elapsed);

        let step = if self.config.fps <= 0.0 {
            duration
        } else {
            1.0 / self.config.fps
        };

        if self.elapsed >= duration {
            self.elapsed = duration + 1.0;
        } else {
            self.elapsed += step;
            if self.elapsed > duration {
                self.elapsed = duration;
            }
        }

        Some(frame)
    }
}

fn sample_render_frame(
    playable: &dyn Playable,
    config: &ExportConfig,
    elapsed: f32,
) -> RenderFrame {
    let scene = playable.resolve_at(elapsed);
    let layout = layout_scene(scene, config.viewport);
    render_frame(playable.name(), elapsed, &layout)
}

/// Pure export frame planner. Collects every frame into memory.
///
/// For streaming (zero heap allocation per frame), use [`playable_frames`]
/// directly with [`write_raw_frames`].
pub fn export_frames(playable: &impl Playable, config: ExportConfig) -> Vec<ExportFrame> {
    playable_frames(playable, config)
        .enumerate()
        .map(|(i, frame)| ExportFrame { index: i, frame })
        .collect()
}

/// Pure streaming: resolve, rasterize, and write each frame's raw RGBA bytes
/// to `writer`. One frame in memory at a time. Deterministic — no I/O beyond
/// what the writer does.
///
/// ```
/// use codimate_animation::animation;
/// use codimate_core::{circle, scene};
/// use codimate_export::{playable_frames, write_raw_frames, ExportConfig};
/// use codimate_layout::Viewport;
///
/// let a = animation("demo", 1.0, scene().node(circle().radius(10.0)));
/// let config = ExportConfig::new(2.0, Viewport::new(800.0, 600.0));
/// let mut buf = Vec::new();
/// write_raw_frames(playable_frames(&a, config), &mut buf).unwrap();
/// // buf now holds width * height * 4 * 3 frames of raw RGBA
/// assert!(buf.len() > 0);
/// ```
pub fn write_raw_frames(
    frames: impl IntoIterator<Item = RenderFrame>,
    writer: &mut impl Write,
) -> Result<(), ExportError> {
    for frame in frames {
        let bitmap = rasterize(&frame);
        writer.write_all(&bitmap.rgba)?;
    }
    Ok(())
}

/// Encode a playable to mp4 via ffmpeg subprocess.
///
/// Spawns `ffmpeg`, pipes raw RGBA frames to its stdin, and waits for it
/// to finish. The output file is overwritten without asking (`-y`).
///
/// Returns `ExportError::EncoderNotFound` if ffmpeg is not on `PATH`.
/// The error type is generic so the caller never mentions "ffmpeg".
///
/// ```
/// use codimate_animation::animation;
/// use codimate_core::{circle, scene};
/// use codimate_export::{export_mp4, ExportConfig};
/// use codimate_layout::Viewport;
/// use std::path::Path;
///
/// let a = animation("demo", 1.0, scene().node(circle().radius(10.0)));
/// let config = ExportConfig::new(30.0, Viewport::new(64.0, 64.0));
/// let result = export_mp4(&a, &config, "/tmp/codimate-test.mp4");
/// // If ffmpeg is installed on this machine, this produces a real mp4.
/// ```
pub fn export_mp4(
    playable: &impl Playable,
    config: &ExportConfig,
    output: impl AsRef<Path>,
) -> Result<(), ExportError> {
    let width = config.viewport.width.round().max(1.0) as u32;
    let height = config.viewport.height.round().max(1.0) as u32;
    let fps = config.fps.max(1.0);

    let mut command = Command::new("ffmpeg");
    command
        .arg("-y")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("rgba")
        .arg("-s")
        .arg(format!("{}x{}", width, height))
        .arg("-r")
        .arg(fps.to_string())
        .arg("-i")
        .arg("-");

    if let Some(output_viewport) = config.output_viewport {
        command
            .arg("-vf")
            .arg(scale_pad_filter(config.viewport, output_viewport));
    }

    let mut child = command
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg(config.crf.to_string())
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(output.as_ref())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|_| ExportError::EncoderNotFound)?;

    let stdin = child.stdin.take().ok_or(ExportError::EncoderNotFound)?;

    let frames = playable_frames(playable, *config);
    write_raw_frames(frames, &mut &stdin)?;

    // Drop stdin so ffmpeg sees EOF
    drop(stdin);

    let status = child.wait().map_err(ExportError::Io)?;
    if !status.success() {
        return Err(ExportError::EncoderFailed(status));
    }
    Ok(())
}

fn scale_pad_filter(input: Viewport, output: Viewport) -> String {
    let input_width = input.width.round().max(1.0);
    let input_height = input.height.round().max(1.0);
    let output_width = even_dimension(output.width);
    let output_height = even_dimension(output.height);

    let scale = (output_width as f32 / input_width).min(output_height as f32 / input_height);
    let scaled_width = even_dimension((input_width * scale).min(output_width as f32));
    let scaled_height = even_dimension((input_height * scale).min(output_height as f32));
    let pad_x = (output_width - scaled_width) / 2;
    let pad_y = (output_height - scaled_height) / 2;

    format!(
        "scale={scaled_width}:{scaled_height}:flags=lanczos,pad={output_width}:{output_height}:{pad_x}:{pad_y}:color=black"
    )
}

fn even_dimension(value: f32) -> u32 {
    let mut dimension = value.round().max(2.0) as u32;
    if !dimension.is_multiple_of(2) {
        dimension += 1;
    }
    dimension
}

// ---------------------------------------------------------------------------
// PNG (unchanged, from the previous slice)
// ---------------------------------------------------------------------------

use codimate_render::Bitmap;

/// Pure: encode a `Bitmap` to PNG bytes. No I/O, deterministic.
pub fn encode_png(bitmap: &Bitmap) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, bitmap.width, bitmap.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("valid PNG header");
    writer
        .write_image_data(&bitmap.rgba)
        .expect("rgba matches width*height*4");
    writer.finish().expect("flush PNG");
    bytes
}

/// Encode `bitmap` and write the PNG to `path`. The only I/O here.
pub fn write_png(path: impl AsRef<Path>, bitmap: &Bitmap) -> io::Result<()> {
    std::fs::write(path, encode_png(bitmap))
}

/// Render one frame of `playable` at `seconds` and save it as a PNG — for
/// eyeballing a single moment without dumping a whole sequence to disk.
pub fn export_frame_png(
    playable: &impl Playable,
    seconds: f32,
    viewport: Viewport,
    path: impl AsRef<Path>,
) -> io::Result<()> {
    let scene = playable.resolve_at(seconds);
    let layout = layout_scene(scene, viewport);
    let frame = render_frame(playable.name(), seconds, &layout);
    let bitmap = rasterize(&frame);
    write_png(path, &bitmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_pad_filter_preserves_aspect_ratio_for_1080p() {
        assert_eq!(
            scale_pad_filter(Viewport::new(1000.0, 640.0), Viewport::new(1920.0, 1080.0)),
            "scale=1688:1080:flags=lanczos,pad=1920:1080:116:0:color=black"
        );
    }
}
