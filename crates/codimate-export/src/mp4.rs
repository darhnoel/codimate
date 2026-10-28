use crate::{sample_render_frame, ExportConfig, ExportError, FrameTimes};
use codimate_animation::Playable;
use codimate_layout::Viewport;
use codimate_render::{rasterize, rasterize_scaled, RenderFrame};
use rayon::prelude::*;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

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

/// Which `ffmpeg` to run.
///
/// `CODIMATE_FFMPEG` wins if set, otherwise whatever is on `PATH`. The
/// override exists so a caller that has its own copy — a Python wheel with a
/// bundled static build, say — can point at it without the user having to
/// install anything system-wide.
fn encoder_binary() -> std::ffi::OsString {
    std::env::var_os("CODIMATE_FFMPEG").unwrap_or_else(|| "ffmpeg".into())
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
    // `Sync` because frames are rasterized across threads; see the loop below.
    playable: &(impl Playable + Sync),
    config: &ExportConfig,
    output: impl AsRef<Path>,
) -> Result<(), ExportError> {
    let pixel_scale = config.pixel_scale.max(1.0);
    let width = (config.viewport.width * pixel_scale).round().max(1.0) as u32;
    let height = (config.viewport.height * pixel_scale).round().max(1.0) as u32;
    let fps = config.fps.max(1.0);

    let mut command = Command::new(encoder_binary());
    command
        .arg("-y")
        // ffmpeg's default is 46 lines of banner, build configuration and
        // libx264 internals per render, which buries anything that actually
        // went wrong and is the first thing a new author sees. `error` keeps
        // the failures; `-stats` keeps the one rewriting progress line, so a
        // two-minute render still looks alive rather than hung.
        .arg("-loglevel")
        .arg("error")
        .arg("-stats")
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
        let render_viewport = Viewport::new(
            config.viewport.width * pixel_scale,
            config.viewport.height * pixel_scale,
        );
        command
            .arg("-vf")
            .arg(scale_pad_filter(render_viewport, output_viewport));
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

    // Frames are resolved and rasterized across cores, then written in order.
    //
    // This is sound for the reason the whole engine is built on: `f(t) -> Scene`
    // is pure (ADR 0008, Invariant 1), so no frame can observe another. The only
    // state any of them share is the formula glyph cache, which is already a
    // Mutex, and the font registry, which is immutable.
    //
    // Work is taken a batch at a time rather than all at once. Holding every
    // bitmap would be `frames * width * height * 4` — for a two-minute 1080p
    // render, tens of gigabytes. A batch bounds it at `BATCH * frame`, about
    // 66 MB at 1080p, which is the price of the parallelism and is paid back
    // several times over in wall time.
    const BATCH: usize = 8;

    let mut times = FrameTimes::new(playable.duration(), config.fps);
    let mut batch: Vec<f32> = Vec::with_capacity(BATCH);

    loop {
        batch.clear();
        batch.extend(times.by_ref().take(BATCH));
        if batch.is_empty() {
            break;
        }

        let bitmaps: Vec<_> = batch
            .par_iter()
            .map(|at| {
                let frame = sample_render_frame(playable, config, *at);
                if pixel_scale > 1.0 {
                    rasterize_scaled(&frame, pixel_scale)
                } else {
                    rasterize(&frame)
                }
            })
            .collect();

        for bitmap in &bitmaps {
            (&stdin).write_all(&bitmap.rgba).map_err(ExportError::Io)?;
        }
    }

    // Drop stdin so ffmpeg sees EOF
    drop(stdin);

    let status = child.wait().map_err(ExportError::Io)?;
    if !status.success() {
        return Err(ExportError::EncoderFailed(status));
    }
    Ok(())
}

pub(crate) fn scale_pad_filter(input: Viewport, output: Viewport) -> String {
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
