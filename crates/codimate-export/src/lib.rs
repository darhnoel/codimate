//! codimate-export — frame export boundary.
//!
//! PNG writing and ffmpeg piping belong here. This first slice only plans the
//! frame samples an exporter would encode.

use codimate_animation::Playable;
use codimate_layout::{layout_scene, Viewport};
use codimate_render::{render_frame, RenderFrame};

/// Export sampling configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExportConfig {
    pub fps: f32,
    pub viewport: Viewport,
}

impl ExportConfig {
    pub fn new(fps: f32, viewport: Viewport) -> Self {
        Self { fps, viewport }
    }
}

/// A render-ready frame to be encoded by a future exporter.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportFrame {
    pub index: usize,
    pub frame: RenderFrame,
}

/// Pure export frame planner. The future implementation will encode these
/// frames to PNG and pipe them into ffmpeg.
pub fn export_frames(playable: &impl Playable, config: ExportConfig) -> Vec<ExportFrame> {
    let duration = playable.duration();
    let step = if config.fps <= 0.0 {
        duration
    } else {
        1.0 / config.fps
    };
    let mut frames = Vec::new();
    let mut elapsed = 0.0;

    while elapsed < duration {
        frames.push(sample(playable, config, frames.len(), elapsed));
        elapsed += step;
    }
    frames.push(sample(playable, config, frames.len(), duration));
    frames
}

fn sample(
    playable: &impl Playable,
    config: ExportConfig,
    index: usize,
    elapsed: f32,
) -> ExportFrame {
    let scene = playable.resolve_at(elapsed);
    let layout = layout_scene(scene, config.viewport);

    ExportFrame {
        index,
        frame: render_frame(playable.name(), elapsed, &layout),
    }
}
