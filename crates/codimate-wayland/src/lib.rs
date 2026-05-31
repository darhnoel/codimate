//! codimate-wayland — live preview boundary.
//!
//! The real Wayland window and frame callback loop belongs here. This first
//! slice only provides pure preview sampling helpers.

use codimate_animation::Playable;
use codimate_core::ConcreteScene;

/// Preview sampling configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewConfig {
    pub fps: f32,
}

impl PreviewConfig {
    pub fn new(fps: f32) -> Self {
        Self { fps }
    }
}

/// A frame sampled for preview at elapsed seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewFrame {
    pub elapsed_seconds: f32,
    pub scene: ConcreteScene,
}

/// Pure frame sampler. The future Wayland loop will own the real clock.
pub fn preview_frames(playable: &impl Playable, config: PreviewConfig) -> Vec<PreviewFrame> {
    let duration = playable.duration();
    let step = if config.fps <= 0.0 {
        duration
    } else {
        1.0 / config.fps
    };
    let mut frames = Vec::new();
    let mut elapsed = 0.0;

    while elapsed < duration {
        frames.push(sample(playable, elapsed, duration));
        elapsed += step;
    }
    frames.push(sample(playable, duration, duration));
    frames
}

fn sample(playable: &impl Playable, elapsed: f32, duration: f32) -> PreviewFrame {
    let t = if duration == 0.0 {
        1.0
    } else {
        (elapsed / duration).min(1.0)
    };
    PreviewFrame {
        elapsed_seconds: elapsed,
        scene: playable.resolve(t),
    }
}
