use crate::{sample_render_frame, ExportConfig, ExportError, FrameTimes};
use codimate_animation::Playable;
use codimate_layout::Viewport;
use codimate_render::{rasterize, rasterize_scaled, RenderFrame};
use rayon::prelude::*;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

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
/// to finish. ffmpeg writes a partial file beside `output`, and only a
/// finished, successful encode replaces `output`: a render that fails or is
/// interrupted leaves the previous video as it was, and no half-written file.
/// An `.mp4`, `.mov` or `.m4v` has its index at the front (`+faststart`), so a
/// browser can start playing it before it has all of it.
///
/// Returns `ExportError::EncoderNotFound` if ffmpeg is not on `PATH`, and
/// `ExportError::EncoderFailed` if ffmpeg exits with an error, even when that
/// first showed up here as a broken pipe. The error type is generic so the
/// caller never mentions "ffmpeg".
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
    // `Sync` because frames are rasterized across threads; see `write_frames`.
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

    command
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg(config.crf.to_string())
        .arg("-pix_fmt")
        .arg("yuv420p");

    let output = output.as_ref();
    if has_index_at_end(output) {
        command.arg("-movflags").arg("+faststart");
    }

    // Removed again unless the encode finishes; see `Partial`.
    let partial = Partial::new(output)?;
    let child = command
        .arg(&partial.path)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|_| ExportError::EncoderNotFound)?;
    let mut encoder = Encoder(Some(child));
    let stdin = encoder
        .child()
        .stdin
        .take()
        .ok_or(ExportError::EncoderNotFound)?;

    let written = write_frames(playable, config, pixel_scale, &stdin);

    // Drop stdin so ffmpeg sees EOF, then always wait for it: ffmpeg's own
    // verdict says more than the broken pipe a failed encode shows up as here.
    drop(stdin);
    let status = encoder.wait()?;
    if !status.success() {
        return Err(ExportError::EncoderFailed(status));
    }
    written?;
    partial.finish(output)
}

/// Resolve, rasterize and write every frame to ffmpeg's stdin, in order.
fn write_frames(
    playable: &(impl Playable + Sync),
    config: &ExportConfig,
    pixel_scale: f32,
    stdin: &std::process::ChildStdin,
) -> Result<(), ExportError> {
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
            (&*stdin).write_all(&bitmap.rgba).map_err(ExportError::Io)?;
        }
    }
    Ok(())
}

/// Whether `path` is a container that keeps its index at the end unless told
/// otherwise, which is what `+faststart` moves to the front.
fn has_index_at_end(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| ["mp4", "mov", "m4v"].contains(&e.to_ascii_lowercase().as_str()))
}

/// Where ffmpeg writes while it encodes: beside `output`, so the final rename
/// stays on one file system, hidden, and with `output`'s extension, which is
/// how ffmpeg picks the container. `None` when `output` names no file.
pub(crate) fn partial_path(output: &Path) -> Option<PathBuf> {
    let mut hidden = std::ffi::OsString::from(".partial-");
    hidden.push(output.file_name()?);
    Some(output.with_file_name(hidden))
}

/// A file being encoded. It is deleted when dropped, which is every way out
/// of `export_mp4` but one: [`Partial::finish`], which moves it into place.
struct Partial {
    path: PathBuf,
    finished: bool,
}

impl Partial {
    fn new(output: &Path) -> Result<Self, ExportError> {
        let path = partial_path(output).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("the video's path {} names no file", output.display()),
            )
        })?;
        Ok(Self {
            path,
            finished: false,
        })
    }

    fn finish(mut self, output: &Path) -> Result<(), ExportError> {
        std::fs::rename(&self.path, output)?;
        self.finished = true;
        Ok(())
    }
}

impl Drop for Partial {
    fn drop(&mut self) {
        if !self.finished {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// The running ffmpeg. Waited for on every way out, a panic included, so it
/// is never left behind still writing the partial file.
struct Encoder(Option<Child>);

impl Encoder {
    fn child(&mut self) -> &mut Child {
        self.0
            .as_mut()
            .expect("the encoder is running until waited for")
    }

    fn wait(&mut self) -> Result<std::process::ExitStatus, ExportError> {
        let mut child = self.0.take().expect("the encoder is waited for once");
        Ok(child.wait()?)
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_partial_file_sits_hidden_beside_the_output_with_its_extension() {
        assert_eq!(
            partial_path(Path::new("/films/salt.mp4")),
            Some(PathBuf::from("/films/.partial-salt.mp4"))
        );
        assert_eq!(partial_path(Path::new("/")), None);
    }

    #[test]
    fn only_containers_with_a_trailing_index_get_faststart() {
        assert!(has_index_at_end(Path::new("a.mp4")));
        assert!(has_index_at_end(Path::new("a.MOV")));
        assert!(!has_index_at_end(Path::new("a.webm")));
        assert!(!has_index_at_end(Path::new("a")));
    }

    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("codimate-unit-{}-{name}", std::process::id()))
    }

    #[test]
    fn an_unfinished_partial_is_removed_and_the_video_is_left_alone() {
        let output = temp("left-alone.mp4");
        std::fs::write(&output, b"previous").unwrap();
        let partial = Partial::new(&output).unwrap();
        std::fs::write(&partial.path, b"half written").unwrap();
        let path = partial.path.clone();

        drop(partial);

        assert!(!path.exists());
        assert_eq!(std::fs::read(&output).unwrap(), b"previous");
        std::fs::remove_file(&output).unwrap();
    }

    #[test]
    fn a_finished_partial_replaces_the_video() {
        let output = temp("replaced.mp4");
        std::fs::write(&output, b"previous").unwrap();
        let partial = Partial::new(&output).unwrap();
        std::fs::write(&partial.path, b"new").unwrap();
        let path = partial.path.clone();

        partial.finish(&output).unwrap();

        assert!(!path.exists());
        assert_eq!(std::fs::read(&output).unwrap(), b"new");
        std::fs::remove_file(&output).unwrap();
    }
}
