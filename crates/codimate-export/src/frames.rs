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
/// The step is **accumulated**, not computed as `i / fps`. The two disagree in
/// the last bits of a float once `i` is large, and that is the difference
/// between a render that is byte-identical to the old one and a render that is
/// merely very close.
#[derive(Clone, Debug)]
pub struct FrameTimes {
    duration: f32,
    fps: f32,
    elapsed: f32,
}

impl FrameTimes {
    /// The times of every frame in a clip `duration` seconds long, at `fps` frames per second.
    #[must_use]
    pub fn new(duration: f32, fps: f32) -> Self {
        Self {
            duration,
            fps,
            elapsed: 0.0,
        }
    }
}

impl Iterator for FrameTimes {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.elapsed > self.duration {
            return None;
        }
        let at = self.elapsed;

        let step = if self.fps <= 0.0 {
            self.duration
        } else {
            1.0 / self.fps
        };

        if self.elapsed >= self.duration {
            self.elapsed = self.duration + 1.0;
        } else {
            self.elapsed += step;
            if self.elapsed > self.duration {
                self.elapsed = self.duration;
            }
        }

        Some(at)
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
