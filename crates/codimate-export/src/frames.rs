use crate::ExportConfig;
use codimate_animation::Playable;
use codimate_layout::layout_scene;
use codimate_render::{render_frame, RenderFrame};

/// A render-ready frame with its sequence index.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportFrame {
    pub index: usize,
    pub frame: RenderFrame,
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
        times: FrameTimes::new(playable.duration(), config.fps),
        playable,
        config,
    }
}

/// Frame-by-frame iterator — see [`playable_frames`].
#[derive(Clone)]
pub struct PlayableFrameIter<'a> {
    playable: &'a dyn Playable,
    config: ExportConfig,
    times: FrameTimes,
}

impl<'a> Iterator for PlayableFrameIter<'a> {
    type Item = RenderFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let at = self.times.next()?;
        Some(sample_render_frame(self.playable, &self.config, at))
    }
}

/// The instants a Playable is sampled at, without sampling it.
///
/// Split out so frames can be resolved in any order — the parallel exporter
/// needs a batch of instants up front, and a serial `Iterator<Item = f32>`
/// is the only honest way to get them.
///
/// Frame `i` is sampled at exactly `i / fps`, worked out in `f64`, which is
/// the instant the encoder shows it at. The last frame lands on `duration`.
/// The step used to be accumulated in `f32` (`elapsed += 1 / fps`), and the
/// rounding piled up: a ten-minute film at 60 fps drew frames up to 102 ms
/// away from when they were shown, out of step with its sound, and encoded
/// seven frames too many (ADR 0011, amended).
#[derive(Clone, Debug)]
pub struct FrameTimes {
    duration: f32,
    fps: f32,
    next: u64,
    last: u64,
}

/// How far `duration * fps` may sit above a whole number and still count as
/// that number: an `f32` duration such as 1.1 s is 33.0000007 frames at 30
/// fps, which is 33 frames, not a 34th a hair after the 33rd.
const SLACK: f64 = 1e-3;

impl FrameTimes {
    /// The times of every frame in a clip `duration` seconds long, at `fps` frames per second.
    #[must_use]
    pub fn new(duration: f32, fps: f32) -> Self {
        // The index of the last frame. A clip of no length has one frame;
        // any other has at least two, its start and its end, as it always had.
        let last = if duration <= 0.0 {
            0
        } else if fps <= 0.0 {
            1
        } else {
            (f64::from(duration) * f64::from(fps) - SLACK)
                .ceil()
                .max(1.0) as u64
        };
        Self {
            duration,
            fps,
            next: if duration < 0.0 { 1 } else { 0 },
            last,
        }
    }

    fn at(&self, i: u64) -> f32 {
        if i == self.last {
            return self.duration;
        }
        if self.fps <= 0.0 {
            return 0.0;
        }
        ((i as f64 / f64::from(self.fps)) as f32).min(self.duration)
    }
}

impl Iterator for FrameTimes {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.next > self.last {
            return None;
        }
        let at = self.at(self.next);
        self.next += 1;
        Some(at)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = (self.last + 1).saturating_sub(self.next) as usize;
        (left, Some(left))
    }
}

pub(crate) fn sample_render_frame(
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
/// directly with [`crate::write_raw_frames`].
pub fn export_frames(playable: &impl Playable, config: ExportConfig) -> Vec<ExportFrame> {
    playable_frames(playable, config)
        .enumerate()
        .map(|(i, frame)| ExportFrame { index: i, frame })
        .collect()
}
