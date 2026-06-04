//! The renderer-neutral command model and the `Renderer` seam.
//!
//! This is the backend-agnostic half of the crate: a `ConcreteScene` (already
//! laid out) is projected into a flat list of [`RenderCommand`]s packaged in a
//! [`RenderFrame`]. No pixels, no `tiny-skia` — any backend (the CPU
//! [`crate::raster`] adapter today, a GPU one tomorrow) consumes this behind
//! the [`Renderer`] seam (ADR 0001).

use codimate_core::{Color, ConcreteNode, Segment};
use codimate_layout::{LayoutFrame, Viewport};

/// Renderer-neutral drawing command produced from a laid-out frame.
#[derive(Clone, Debug, PartialEq)]
pub enum RenderCommand {
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        fill: Color,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fill: Color,
    },
    Path {
        segments: Vec<Segment>,
        closed: bool,
        fill: Color,
        stroke_width: f32,
        stroke_color: Color,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        font_size: f32,
        fill: Color,
    },
}

/// Convert a laid-out frame into renderer-neutral commands.
pub fn render_commands(frame: &LayoutFrame) -> Vec<RenderCommand> {
    frame
        .scene
        .children
        .iter()
        .map(|node| match node {
            ConcreteNode::Circle(circle) => RenderCommand::Circle {
                x: circle.x,
                y: circle.y,
                radius: circle.radius,
                fill: circle.fill,
            },
            ConcreteNode::Rect(rect) => RenderCommand::Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                fill: rect.fill,
            },
            ConcreteNode::Path(path) => RenderCommand::Path {
                segments: path.path.segments.clone(),
                closed: path.path.closed,
                fill: path.fill,
                stroke_width: path.stroke_width,
                stroke_color: path.stroke_color,
            },
            ConcreteNode::Text(text) => RenderCommand::Text {
                x: text.x,
                y: text.y,
                text: text.text.clone(),
                font_size: text.font_size,
                fill: text.fill,
            },
        })
        .collect()
}

/// Renderer-neutral frame ready for a backend or export pipeline.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderFrame {
    pub name: String,
    pub elapsed_seconds: f32,
    pub viewport: Viewport,
    pub commands: Vec<RenderCommand>,
}

/// Package a laid-out frame and metadata into renderer-neutral commands.
pub fn render_frame(
    name: impl Into<String>,
    elapsed_seconds: f32,
    frame: &LayoutFrame,
) -> RenderFrame {
    RenderFrame {
        name: name.into(),
        elapsed_seconds,
        viewport: frame.viewport,
        commands: render_commands(frame),
    }
}

/// Inject debug metadata into a rendered frame.
///
/// Appends a green overlay [`RenderCommand::Text`] with the animation name and
/// elapsed seconds so rendered frames can be traced back to their source during
/// testing or AI feedback.
pub fn inject_debug_metadata(frame: &mut RenderFrame) {
    let text = format!("{}  {:.2}s", frame.name, frame.elapsed_seconds);
    frame.commands.push(RenderCommand::Text {
        x: 8.0,
        y: 20.0,
        text,
        font_size: 12.0,
        fill: Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
    });
}

/// Backend trait implemented by concrete renderers — the seam where a CPU,
/// GPU, or Skia adapter plugs in (ADR 0001).
pub trait Renderer {
    type Error;

    fn render(&mut self, frame: &LayoutFrame) -> Result<(), Self::Error>;
}
