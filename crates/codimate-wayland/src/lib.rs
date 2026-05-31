//! codimate-wayland — live preview boundary.
//!
//! The real Wayland window and frame callback loop belongs here. This first
//! slice only provides pure preview sampling helpers.

use codimate_animation::Playable;
use codimate_layout::{layout_scene, Viewport};
use codimate_render::{render_frame, RenderFrame};

/// Preview sampling configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewConfig {
    pub fps: f32,
    pub viewport: Viewport,
}

impl PreviewConfig {
    pub fn new(fps: f32, viewport: Viewport) -> Self {
        Self { fps, viewport }
    }
}

/// Pure frame sampler. The future Wayland loop will own the real clock.
pub fn preview_frames(playable: &impl Playable, config: PreviewConfig) -> Vec<RenderFrame> {
    let duration = playable.duration();
    let step = if config.fps <= 0.0 {
        duration
    } else {
        1.0 / config.fps
    };
    let mut frames = Vec::new();
    let mut elapsed = 0.0;

    while elapsed < duration {
        frames.push(sample(playable, config, elapsed));
        elapsed += step;
    }
    frames.push(sample(playable, config, duration));
    frames
}

fn sample(playable: &impl Playable, config: PreviewConfig, elapsed: f32) -> RenderFrame {
    let scene = playable.resolve_at(elapsed);
    let layout = layout_scene(scene, config.viewport);

    render_frame(playable.name(), elapsed, &layout)
}
