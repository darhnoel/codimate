//! codimate-export — frame export boundary.
//!
//! PNG writing and ffmpeg piping belong here. This first slice only plans the
//! frame samples an exporter would encode.

use codimate_animation::Playable;
use codimate_core::ConcreteScene;

/// Export sampling configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExportConfig {
    pub fps: f32,
}

impl ExportConfig {
    pub fn new(fps: f32) -> Self {
        Self { fps }
    }
}

/// A concrete frame to be encoded by a future exporter.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportFrame {
    pub index: usize,
    pub elapsed_seconds: f32,
    pub scene: ConcreteScene,
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
        frames.push(sample(playable, frames.len(), elapsed, duration));
        elapsed += step;
    }
    frames.push(sample(playable, frames.len(), duration, duration));
    frames
}

fn sample(playable: &impl Playable, index: usize, elapsed: f32, duration: f32) -> ExportFrame {
    let t = if duration == 0.0 {
        1.0
    } else {
        (elapsed / duration).min(1.0)
    };
    ExportFrame {
        index,
        elapsed_seconds: elapsed,
        scene: playable.resolve(t),
    }
}
